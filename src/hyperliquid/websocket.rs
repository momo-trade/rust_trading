use crate::hyperliquid::model::{CustomCandle, CustomTrade, CustomUserFills};
use crate::hyperliquid::subscriptions::Subscription;
use anyhow::{Context, Ok, Result};
use ethers::types::H160;
use hyperliquid_rust_sdk::{BaseUrl, InfoClient, Message, Subscription as HyperliquidSubscription};
use log::{error, info};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::{mpsc::UnboundedSender, RwLock};

#[derive(Clone, Debug)]
pub struct WsData {
    pub all_mids: HashMap<String, String>,
    pub trades: Vec<CustomTrade>,
    pub max_trades: usize,
    pub candles: Vec<CustomCandle>,
    pub max_candles: usize,
    pub user_fills: Vec<CustomUserFills>,
    pub max_fills: usize,
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

    pub fn add_fills(&mut self, fills: Vec<CustomUserFills>, user: H160) {
        self.user_fills.extend(fills.clone());

        if self.user_fills.len() > self.max_fills {
            let excess = self.user_fills.len() - self.max_fills;
            self.user_fills.drain(0..excess);
        }

        if let Err(e) = self.append_fills_to_file(fills, user) {
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
}

pub struct WebSocketManager {
    info_client: Arc<RwLock<InfoClient>>,
    ws_data: Arc<RwLock<WsData>>,
    subscription: Arc<RwLock<HashMap<String, u32>>>,
}

impl WebSocketManager {
    pub async fn new(is_mainnet: bool) -> Arc<Self> {
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
            ws_data: Arc::new(RwLock::new(WsData::default())),
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
                            data.add_fills(custom_fills, user_fills.data.user);
                        }
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
}
