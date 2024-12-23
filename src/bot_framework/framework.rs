use crate::bot_framework::init::{initialize_bot, InitOptions};
use crate::hyperliquid::websocket::WebSocketManager;
use anyhow::Result;
use async_trait::async_trait;
use log::info;

use super::init::InitResources;

/// Trait for defining the lifecycle of a trading bot
#[async_trait]
pub trait BotFramework {
    /// Define bot-specific initialization options
    fn init_options(&self) -> InitOptions;

    /// Define required subscriptions
    async fn subscribe(&mut self, ws_manager: &WebSocketManager) -> Result<()>;

    /// Define the main logic of the bot
    async fn execute(&mut self, resources: &InitResources) -> Result<()>;

    fn loop_interval(&self) -> u64 {
        5 // Default interval: 5 seconds
    }
}

/// Main execution flow for running a bot
pub async fn run_bot<B: BotFramework + Send + Sync>(mut bot: B) -> Result<()> {
    let options = bot.init_options();
    let resources = initialize_bot(options).await?;

    info!("Subscribing to necessary data...");
    bot.subscribe(&resources.ws_manager).await?;

    let interval = bot.loop_interval();
    info!("Loop interval: {} seconds", interval);

    info!("Starting the bot loop...");
    loop {
        bot.execute(&resources).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
    }
}
