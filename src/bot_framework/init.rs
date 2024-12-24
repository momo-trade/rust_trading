use crate::hyperliquid::http::HttpClient;
use crate::hyperliquid::websocket::WebSocketManager;
use anyhow::{Context, Result};
use ethers::signers::LocalWallet;
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub wallet_secret: String,
    pub is_mainnet: bool,
    pub coin: String,
    pub interval: u64,
    #[serde(default)] // User empty object if bot_specific is missing
    pub bot_specific: Value, // Bot-specific configuration
}

pub struct InitResources {
    pub ws_manager: Arc<WebSocketManager>,
    pub http_client: HttpClient,
    pub wallet: LocalWallet,
    pub config: Config,
}

pub async fn initialize_bot(config_path: &str) -> Result<InitResources> {
    // dotenv::dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config_content = fs::read_to_string(config_path).context("Failed to read config file")?;
    let config: Config = toml::from_str(&config_content).context("Failed to parse config file")?;

    // let private_key = env::var("WALLET_SECRET").expect("WALLET_SECRET not set");
    let wallet = LocalWallet::from_str(&config.wallet_secret).context("Invalid wallet secret")?;

    let ws_manager = WebSocketManager::new(config.is_mainnet).await;
    let http_client = HttpClient::new(config.is_mainnet, wallet.clone()).await?;

    Ok(InitResources {
        ws_manager,
        http_client,
        wallet,
        config,
    })
}
