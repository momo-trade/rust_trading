use anyhow::Result;
use log::info;
use rust_trading::bot_framework::framework::{run_bot, BotFramework};
use rust_trading::bot_framework::init::InitOptions;
use rust_trading::bot_framework::init::InitResources;
use rust_trading::hyperliquid::subscriptions::Subscription;
use rust_trading::hyperliquid::websocket::WebSocketManager;

pub struct SampleBot {
    coin: String,
    interval: u64,
}

impl SampleBot {
    pub fn new(coin: String, interval: u64) -> Self {
        Self { coin, interval }
    }
}

#[async_trait::async_trait]
impl BotFramework for SampleBot {
    fn init_options(&self) -> InitOptions {
        InitOptions {
            is_mainnet: true,
            coin: self.coin.clone(),
        }
    }

    async fn subscribe(&mut self, ws_manager: &WebSocketManager) -> Result<()> {
        ws_manager
            .subscribe(Subscription::L2Book {
                coin: self.coin.clone(),
            })
            .await?;
        info!("Subscribed to L2Book for {}", self.coin);
        Ok(())
    }

    fn loop_interval(&self) -> u64 {
        self.interval
    }

    async fn execute(&mut self, resources: &InitResources) -> Result<()> {
        let ws_manager = &resources.ws_manager;
        let best_bid = ws_manager.get_best_bid().await;
        let best_ask = ws_manager.get_best_ask().await;

        info!("Best bid: {}, best ask: {}", best_bid, best_ask);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let bot = SampleBot::new("@107".to_string(), 10); // HYPE/USDC(Spot)
    run_bot(bot).await
}
