use crate::hyperliquid::model::{CustomCandle, CustomTrade};
use hyperliquid_rust_sdk::{BaseUrl, InfoClient, Message, Subscription};
use log::{error, info};
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tokio::sync::{watch, RwLock};
use tokio::time::Duration;

#[derive(Clone, Debug)]
pub struct WsData {
    pub all_mids: HashMap<String, String>,
    pub trades: Vec<CustomTrade>,
    pub max_trades: usize,
    pub candles: Vec<CustomCandle>,
    pub max_candles: usize,
}

impl Default for WsData {
    fn default() -> Self {
        Self {
            all_mids: HashMap::new(),
            trades: Vec::new(),
            max_trades: 10000, // デフォルトの上限を設定
            candles: Vec::new(),
            max_candles: 10000,
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
            *existing_candle = new_candle; //同じ時刻のcandleを上書き
        } else {
            self.candles.push(new_candle);
        }

        if self.candles.len() > self.max_candles {
            let excess = self.candles.len() - self.max_candles;
            self.candles.drain(0..excess);
        }
    }
}

pub struct WebSocketManager {
    info_client: Arc<RwLock<InfoClient>>,
    ws_data: Arc<RwLock<WsData>>,
    ws_data_sender: watch::Sender<WsData>,
    ws_data_receiver: watch::Receiver<WsData>,
    subscriptions: Arc<RwLock<HashSet<String>>>,
}

impl WebSocketManager {
    pub async fn new(base_url: BaseUrl) -> Arc<Self> {
        let info_client = Arc::new(RwLock::new(
            InfoClient::new(None, Some(base_url))
                .await
                .expect("Failed to initialize InfoClient"),
        ));
        let (ws_data_sender, ws_data_receiver) = watch::channel(WsData::default());

        Arc::new(Self {
            info_client,
            ws_data: Arc::new(RwLock::new(WsData::default())),
            ws_data_sender,
            ws_data_receiver,
            subscriptions: Arc::new(RwLock::new(HashSet::new())),
        })
    }

    fn subscription_key(subscription: &Subscription) -> String {
        match subscription {
            Subscription::AllMids => "AllMids".to_string(),
            Subscription::Trades { coin } => format!("Trades:{}", coin),
            Subscription::Candle { coin, interval } => format!("Candle:{}:{}", coin, interval),
            _ => "UnknownSubscription".to_string(),
        }
    }

    fn parse_subscription_key(key: &str) -> Option<Subscription> {
        let parts: Vec<&str> = key.split(':').collect();
        match parts.as_slice() {
            ["AllMids"] => Some(Subscription::AllMids),
            ["Trades", coin] => Some(Subscription::Trades {
                coin: coin.to_string(),
            }),
            ["Candle", coin, interval] => Some(Subscription::Candle {
                coin: coin.to_string(),
                interval: interval.to_string(),
            }),
            // ["L2Book"] => Some(Subscription::L2Book { /* デフォルトのパラメータ */ }),
            // ["UserEvents"] => Some(Subscription::UserEvents { /* デフォルトのパラメータ */ }),
            // ["UserFills"] => Some(Subscription::UserFills { /* デフォルトのパラメータ */ }),
            // ["Positions"] => Some(Subscription::Positions),
            // ["Orders"] => Some(Subscription::Orders),
            // ["Balances"] => Some(Subscription::Balances),
            _ => None,
        }
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

    pub async fn add_subscription(&self, subscription: Subscription) {
        let key = Self::subscription_key(&subscription);
        let mut subscriptions = self.subscriptions.write().await;

        if subscriptions.insert(key.clone()) {
            info!("Add subscription key: {}", key);
        } else {
            info!("Subscription key already exists: {}", key);
        }
    }

    pub async fn list_subscription(&self) -> Vec<String> {
        self.subscriptions.read().await.iter().cloned().collect()
    }

    /// 再接続時に現在の`subscriptions`を再登録する
    async fn resubscribe_all(
        &self,
        sender: tokio::sync::mpsc::UnboundedSender<Message>,
    ) -> Result<(), String> {
        let subscriptions = self.subscriptions.read().await;

        for key in subscriptions.iter() {
            if let Some(subscription) = Self::parse_subscription_key(key) {
                if let Err(e) = self
                    .info_client
                    .write()
                    .await
                    .subscribe(subscription, sender.clone())
                    .await
                {
                    error!("Failed to resubscribe: {}", e);
                    return Err(format!("Failed to resubscribe to {}", key));
                }
            } else {
                error!("Invalid subscription key: {}", key);
            }
        }

        Ok(())
    }

    async fn process_message(
        message: Message,
        ws_data: Arc<RwLock<WsData>>,
        ws_data_sender: &watch::Sender<WsData>,
    ) {
        match message {
            Message::AllMids(all_mids) => {
                let mut data = ws_data.write().await;
                data.all_mids = all_mids.data.mids;
                if let Err(e) = ws_data_sender.send(data.clone()) {
                    error!("Failed to send all_mids data: {}", e);
                }
            }
            Message::Trades(trades) => {
                let custom_trades: Vec<CustomTrade> =
                    trades.data.into_iter().map(CustomTrade::from).collect();

                let mut data = ws_data.write().await;
                data.add_trade(custom_trades);

                if let Err(e) = ws_data_sender.send(data.clone()) {
                    error!("Failed to send trades data: {}", e);
                }
            }
            Message::L2Book(l2_book) => {
                info!("L2Book: {:#?}", l2_book);
            }
            Message::Candle(candle) => {
                let custom_candle: CustomCandle = CustomCandle::from(candle.data);

                let mut data = ws_data.write().await;
                data.add_candle(custom_candle);

                if let Err(e) = ws_data_sender.send(data.clone()) {
                    error!("Failed to send candles data: {}", e);
                }
            }
            Message::UserFills(user_fills) => {
                info!("UserFills: {:#?}", user_fills);
            }
            _ => {
                info!("Unhandled message: {:?}", message);
            }
        }
    }

    pub fn start(self: &Arc<Self>) {
        let manager = self.clone();
        tokio::spawn(async move {
            manager.run_loop().await;
        });
    }

    async fn run_loop(self: Arc<Self>) {
        info!("Starting WebSocketManager...");
        let mut reconnect_attempts = 0;
        loop {
            let (_, mut receiver) = {
                let (tx, rx) = unbounded_channel();
                match self.resubscribe_all(tx.clone()).await {
                    Ok(_) => {
                        info!("Subscribed successfully.");
                        reconnect_attempts = 0; // リセット
                        (tx, rx)
                    }
                    Err(e) => {
                        error!("Failed to resubscribe: {}", e);
                        reconnect_attempts += 1;
                        let sleep_duration = Duration::from_secs(5 * reconnect_attempts.min(6));
                        tokio::time::sleep(sleep_duration).await;
                        continue;
                    }
                }
            };

            let ws_data = self.ws_data.clone();
            let ws_data_sender = self.ws_data_sender.clone();
            let mut ws_data_receiver = self.ws_data_receiver.clone();
            let timeout_duration = Duration::from_secs(30);

            loop {
                tokio::select! {
                    Some(message) = receiver.recv() => {
                        Self::process_message(message, ws_data.clone(), &ws_data_sender).await;
                    }
                    Ok(_) = ws_data_receiver.changed() => {
                        // データ更新を検知（必要なら何か処理を追加）
                    }
                    _ = tokio::time::sleep(timeout_duration) => {
                        error!("WebSocketManager timeout. Reconnecting...");
                        break; // 再接続を試みる
                    }
                }
            }
        }
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
}
