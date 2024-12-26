use anyhow::Result;
use ethers::signers::Signer;
use log::info;
use rust_trading::bot_framework::framework::{run_bot, BotFramework};
use rust_trading::bot_framework::init::InitResources;
use rust_trading::hyperliquid::subscriptions::Subscription;
use serde_json::Value;

pub struct SampleBot;

#[async_trait::async_trait]
impl BotFramework for SampleBot {
    async fn subscribe(&mut self, resources: &InitResources) -> Result<()> {
        let ws_manager = &resources.ws_manager;
        let config = &resources.config;
        ws_manager
            .subscribe(Subscription::L2Book {
                coin: config.coin.clone(),
            })
            .await?;
        ws_manager
            .subscribe(Subscription::UserFills {
                user: resources.wallet.address(),
            })
            .await?;
        info!("Subscribed to L2Book for {}", config.coin);
        Ok(())
    }

    async fn execute(&mut self, resources: &InitResources) -> Result<()> {
        let config = &resources.config;

        // Example of loading bot-specific settings
        let threshold: f64 = config
            .bot_specific
            .get("threshold")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        info!("custom bot config: {}", threshold);

        let ws_manager = &resources.ws_manager;
        let best_bid = ws_manager.get_best_bid().await;
        let best_ask = ws_manager.get_best_ask().await;

        let user_fills = ws_manager.get_user_fills().await;
        info!("User fills: {}", user_fills.len());

        info!("Best bid: {:.3}, best ask: {:.3}", best_bid, best_ask);

        match ws_manager.get_position(&config.coin).await {
            Some(position) => {
                info!(
                    "Position => coin: {}, amount: {}, avg_price: {}, realized_pnl: {}, unrealized_pnl: {}",
                    position.coin, position.amount, position.average_price, position.pnl.realized, position.pnl.unrealized
                );
            }
            None => {
                info!("No position for {}", config.coin);
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = "examples/config/sample_bot.toml";
    let bot = SampleBot;
    run_bot(bot, config_path).await
}
