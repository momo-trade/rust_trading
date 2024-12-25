use crate::bot_framework::init::{initialize_bot, InitResources};
use anyhow::Result;
use async_trait::async_trait;
use log::{error, info};
use tokio::signal;

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

    let loop_interval_secs = resources.config.interval;
    info!("Loop interval: {} seconds", loop_interval_secs);

    info!("Starting the bot loop...");
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(loop_interval_secs));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                if let Err(e) = bot.execute(&resources).await {
                    error!("Error executing bot: {:?}", e);
                }
            }
            _ = signal::ctrl_c() => {
                info!("Shutdown signal received");
                break;
            }
        }
    }

    info!("Bot stopped");
    Ok(())
}
