use crate::bot_framework::init::{initialize_bot, InitResources};
use anyhow::Result;
use async_trait::async_trait;
use log::info;

/// Trait for defining the lifecycle of a trading bot
#[async_trait]
pub trait BotFramework {
    /// Define required subscriptions
    async fn subscribe(&mut self, resources: &InitResources) -> Result<()>;

    /// Define the main logic of the bot
    async fn execute(&mut self, resources: &InitResources) -> Result<()>;
}

/// Main execution flow for running a bot
pub async fn run_bot<B: BotFramework + Send + Sync>(mut bot: B, config_path: &str) -> Result<()> {

    // Initialize resources using the configuration file
    let resources = initialize_bot(config_path).await?;

    info!("Subscribing to necessary data...");
    bot.subscribe(&resources).await?;

    let interval = resources.config.interval;
    info!("Loop interval: {} seconds", interval);

    info!("Starting the bot loop...");
    loop {
        bot.execute(&resources).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
    }
}
