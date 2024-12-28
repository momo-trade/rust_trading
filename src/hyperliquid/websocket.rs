use crate::hyperliquid::db::save_fills_to_db;
use crate::hyperliquid::model::{CustomCandle, CustomL2Book, CustomTrade, CustomUserFills};
use crate::hyperliquid::portfolio::{PortfolioManager, Position};
use crate::hyperliquid::subscriptions::Subscription;
use anyhow::{Context, Result};
use ethers::types::H160;
use hyperliquid_rust_sdk::{BaseUrl, InfoClient, Message, Subscription as HyperliquidSubscription};
use log::{error, info};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::{mpsc::UnboundedSender, RwLock};
use tokio_postgres::Client;

#[derive(Clone, Debug)]
pub struct WsData {
    pub all_mids: HashMap<String, String>,
    pub trades: Vec<CustomTrade>,
    pub max_trades: usize,
    pub candles: Vec<CustomCandle>,
    pub max_candles: usize,
    pub user_fills: Vec<CustomUserFills>,
    pub max_fills: usize,
    pub l2_books: Vec<CustomL2Book>,
    pub max_l2_book: usize,
    pub best_bid: f64,
    pub best_ask: f64,
    pub db_client: Option<Arc<Client>>,
    pub portfolio_manager: PortfolioManager,
}

impl Default for WsData {
    fn default() -> Self {
        Self {
            all_mids: HashMap::new(),
            trades: Vec::new(),
            max_trades: 10000, // Default limit for trades
            candles: Vec::new(),
            max_candles: 10000, // Default limit for candles
            user_fills: Vec::new(),
            max_fills: 10000,
            l2_books: Vec::new(),
            max_l2_book: 100,
            best_bid: 0.0,
            best_ask: 0.0,
            db_client: None,
            portfolio_manager: PortfolioManager::new(),
        }
    }
}

impl WsData {
    pub fn add_trade(&mut self, new_trades: Vec<CustomTrade>) {
        self.trades.extend(new_trades);
        if self.trades.len() > self.max_trades {
            let excess = self.trades.len() - self.max_trades;
            self.trades.drain(0..excess);
        }
    }

    pub fn add_candle(&mut self, new_candle: CustomCandle) {
        if let Some(existing_candle) = self
            .candles
            .iter_mut()
            .find(|candle| candle.time_open == new_candle.time_open)
        {
            *existing_candle = new_candle; // Overwrite the existing candle with the same time
        } else {
            self.candles.push(new_candle);
        }

        if self.candles.len() > self.max_candles {
            let excess = self.candles.len() - self.max_candles;
            self.candles.drain(0..excess);
        }
    }

    pub fn add_l2_book(&mut self, new_l2_book: CustomL2Book) {
        self.l2_books.push(new_l2_book);
        if self.l2_books.len() > self.max_l2_book {
            let excess = self.l2_books.len() - self.max_l2_book;
            self.l2_books.drain(0..excess);
        }

        if let Some(latest_l2_book) = self.l2_books.last() {
            self.best_bid = latest_l2_book
                .bid_levels
                .first()
                .map_or(0.0, |bid| bid.price);
            self.best_ask = latest_l2_book
                .ask_levels
                .first()
                .map_or(0.0, |ask| ask.price);
        }
    }

    pub async fn add_fills(&mut self, fills: Vec<CustomUserFills>, user: H160) {
        self.user_fills.extend(fills.clone());

        if self.user_fills.len() > self.max_fills {
            let excess = self.user_fills.len() - self.max_fills;
            self.user_fills.drain(0..excess);
        }

        for fill in &fills {
            self.portfolio_manager.update_position(fill);
        }

        if let Some(db_client) = &self.db_client {
            if let Err(e) = save_fills_to_db(db_client, &fills, user).await {
                error!("Failed to save fills to database: {}", e);
            }
        } else if let Err(e) = self.append_fills_to_file(fills, user) {
            error!("Failed to append fills to file: {}", e);
        }
    }

    fn append_fills_to_file(&self, fills: Vec<CustomUserFills>, user: H160) -> Result<()> {
        let file_name = format!("{:?}_fills.log", user);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_name)
            .context("Failed to open user fills file")?;

        for fill in fills {
            let json = serde_json::to_string(&fill).context("Failed to serialize fill")?;
            writeln!(file, "{}", json).context("Failed to write to user fills file")?;
        }

        Ok(())
    }

    pub fn calculate_thickness(&self) -> (f64, f64) {
        let mut bid_thickness = 0.0;
        let mut ask_thickness = 0.0;

        if let Some(latest_book) = self.l2_books.last() {
            for bid in &latest_book.bid_levels {
                bid_thickness += bid.size;
            }
            for ask in &latest_book.ask_levels {
                ask_thickness += ask.size;
            }
        }

        (bid_thickness, ask_thickness)
    }

    pub fn calculate_average_thickness(&self) -> (f64, f64) {
        let mut total_bid_thickness = 0.0;
        let mut total_ask_thickness = 0.0;
        let count = self.l2_books.len() as f64;

        for book in &self.l2_books {
            for bid in &book.bid_levels {
                total_bid_thickness += bid.size;
            }
            for ask in &book.ask_levels {
                total_ask_thickness += ask.size;
            }
        }

        let average_bid_thickness = if count > 0.0 {
            total_bid_thickness / count
        } else {
            0.0
        };

        let average_ask_thickness = if count > 0.0 {
            total_ask_thickness / count
        } else {
            0.0
        };

        (average_bid_thickness, average_ask_thickness)
    }

    pub fn calculate_thickness_near_best(&self, tick_size: f64, tick_range: usize) -> (f64, f64) {
        let mut bid_thickness = 0.0;
        let mut ask_thickness = 0.0;

        if let Some(latest_book) = self.l2_books.last() {
            if let Some(best_bid) = latest_book.bid_levels.first() {
                let bid_min = best_bid.price - (tick_size * tick_range as f64);
                let bid_max = best_bid.price;

                for bid in &latest_book.bid_levels {
                    if bid.price >= bid_min && bid.price <= bid_max {
                        bid_thickness += bid.size;
                    }
                }
            }

            if let Some(best_ask) = latest_book.ask_levels.first() {
                let ask_min = best_ask.price;
                let ask_max = best_ask.price + (tick_size * tick_range as f64);

                for ask in &latest_book.ask_levels {
                    if ask.price >= ask_min && ask.price <= ask_max {
                        ask_thickness += ask.size;
                    }
                }
            }
        }

        (bid_thickness, ask_thickness)
    }
}

pub struct WebSocketManager {
    info_client: Arc<RwLock<InfoClient>>,
    ws_data: Arc<RwLock<WsData>>,
    subscription: Arc<RwLock<HashMap<String, u32>>>,
}

impl WebSocketManager {
    pub async fn new(is_mainnet: bool, db_client: Option<Arc<Client>>) -> Arc<Self> {
        let base_url = if is_mainnet {
            BaseUrl::Mainnet
        } else {
            BaseUrl::Testnet
        };

        let info_client = Arc::new(RwLock::new(
            InfoClient::with_reconnect(None, Some(base_url))
                .await
                .expect("Failed to initialize InfoClient"),
        ));

        Arc::new(Self {
            info_client,
            subscription: Arc::new(RwLock::new(HashMap::new())),
            ws_data: Arc::new(RwLock::new(WsData {
                db_client,
                ..WsData::default()
            })),
        })
    }

    pub async fn subscribe(&self, subscription: Subscription) -> Result<()> {
        let internal_subscription: HyperliquidSubscription = subscription.into();
        let subscription_key = serde_json::to_string(&internal_subscription)
            .context("Failed to serialize subscription")?;

        let mut subscriptions = self.subscription.write().await;
        if subscriptions.contains_key(&subscription_key) {
            return Ok(());
        }

        let sender = self.create_subscription_channel().await?;
        let subscription_id = self
            .info_client
            .write()
            .await
            .subscribe(internal_subscription, sender)
            .await
            .context("Failed to subscribe")?;

        subscriptions.insert(subscription_key, subscription_id);

        Ok(())
    }

    pub async fn unsubscribe(&self, subscription: Subscription) -> Result<()> {
        let internal_subscription: HyperliquidSubscription = subscription.into();
        let subscription_key = serde_json::to_string(&internal_subscription)
            .context("Failed to serialize subscription")?;

        let mut subscriptions = self.subscription.write().await;
        if let Some(subscription_id) = subscriptions.remove(&subscription_key) {
            self.info_client
                .write()
                .await
                .unsubscribe(subscription_id)
                .await
                .context("Failed to unsubscribe")?;
        }
        Ok(())
    }

    async fn create_subscription_channel(&self) -> Result<UnboundedSender<Message>> {
        let (sender, mut receiver) = unbounded_channel();

        let ws_data = self.ws_data.clone();
        tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                match message {
                    Message::AllMids(all_mids) => {
                        let mut data = ws_data.write().await;
                        data.all_mids = all_mids.data.mids;
                    }
                    Message::Trades(trades) => {
                        let custom_trades: Vec<CustomTrade> =
                            trades.data.into_iter().map(CustomTrade::from).collect();

                        let mut data = ws_data.write().await;
                        data.add_trade(custom_trades);
                    }
                    Message::Candle(candle) => {
                        let custom_candle: CustomCandle = CustomCandle::from(candle.data);
                        let mut data = ws_data.write().await;
                        data.add_candle(custom_candle);
                    }
                    Message::UserFills(user_fills) => {
                        if user_fills.data.is_snapshot != Some(true) {
                            let custom_fills: Vec<CustomUserFills> = user_fills
                                .data
                                .fills
                                .into_iter()
                                .map(CustomUserFills::from)
                                .collect();

                            let mut data = ws_data.write().await;
                            data.add_fills(custom_fills, user_fills.data.user).await;
                        }
                    }
                    Message::L2Book(l2_book) => {
                        let custom_l2_book: CustomL2Book = CustomL2Book::from(l2_book.data);
                        let mut data = ws_data.write().await;
                        data.add_l2_book(custom_l2_book);
                    }
                    Message::NoData => {
                        error!("Disconnected from websocket");
                    }
                    _ => {
                        info!("Unhandled message: {:#?}", message);
                    }
                }
            }
        });
        Ok(sender)
    }

    pub async fn set_max_trades(&self, max_trades: usize) {
        let mut ws_data = self.ws_data.write().await;
        ws_data.max_trades = max_trades;
        info!("Updated max trades to {}", max_trades);
    }

    pub async fn set_max_candles(&self, max_candles: usize) {
        let mut ws_data = self.ws_data.write().await;
        ws_data.max_candles = max_candles;
        info!("Updated max candles to {}", max_candles);
    }

    pub async fn set_max_fills(&self, max_fills: usize) {
        let mut ws_data = self.ws_data.write().await;
        ws_data.max_fills = max_fills;
        info!("Updated max user fills to {}", max_fills);
    }

    pub async fn set_max_l2_book(&self, max_l2_book: usize) {
        let mut ws_data = self.ws_data.write().await;
        ws_data.max_l2_book = max_l2_book;
        info!("Updated max l2 book to {}", max_l2_book);
    }

    pub async fn get_all_mids(&self) -> HashMap<String, String> {
        self.ws_data.read().await.all_mids.clone()
    }

    pub async fn get_trades(&self) -> Vec<CustomTrade> {
        self.ws_data.read().await.trades.clone()
    }

    pub async fn get_candles(&self) -> Vec<CustomCandle> {
        self.ws_data.read().await.candles.clone()
    }

    pub async fn get_user_fills(&self) -> Vec<CustomUserFills> {
        self.ws_data.read().await.user_fills.clone()
    }

    pub async fn get_l2_books(&self) -> Vec<CustomL2Book> {
        self.ws_data.read().await.l2_books.clone()
    }

    pub async fn get_best_bid(&self) -> f64 {
        self.ws_data.read().await.best_bid
    }

    pub async fn get_best_ask(&self) -> f64 {
        self.ws_data.read().await.best_ask
    }

    pub async fn get_position(&self, coin: &str) -> Option<Position> {
        self.ws_data
            .read()
            .await
            .portfolio_manager
            .get_position(coin)
            .cloned()
    }

    pub async fn get_unrealized_pnl(&self, coin: &str) -> f64 {
        let current_price = self
            .get_all_mids()
            .await
            .get(coin)
            .map_or(0.0, |price| price.parse::<f64>().unwrap_or(0.0));
        self.ws_data
            .read()
            .await
            .portfolio_manager
            .get_unrealized_pnl(coin, current_price)
    }

    pub async fn get_thickness(&self) -> (f64, f64) {
        let ws_data = self.ws_data.read().await;
        ws_data.calculate_thickness()
    }

    pub async fn get_average_thickness(&self) -> (f64, f64) {
        let ws_data = self.ws_data.read().await;
        ws_data.calculate_average_thickness()
    }

    pub async fn get_thickness_near_best(&self, tick_size: f64, tick_range: usize) -> (f64, f64) {
        let ws_data = self.ws_data.read().await;
        ws_data.calculate_thickness_near_best(tick_size, tick_range)
    }
}
