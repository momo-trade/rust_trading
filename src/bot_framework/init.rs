use crate::hyperliquid::http::HttpClient;
use crate::hyperliquid::subscriptions::Subscription;
use crate::hyperliquid::websocket::WebSocketManager;
use anyhow::{Context, Result};
use ethers::signers::{LocalWallet, Signer};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::str::FromStr;
use std::sync::Arc;
use tokio_postgres::{Client, NoTls};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub wallet_secret: String,
    pub is_mainnet: bool,
    pub coin: String,
    pub interval: u64,
    pub database_url: Option<String>,
    #[serde(default)] // User empty object if bot_specific is missing
    pub bot_specific: Value, // Bot-specific configuration
}

pub struct InitResources {
    pub ws_manager: Arc<WebSocketManager>,
    pub http_client: HttpClient,
    pub wallet: LocalWallet,
    pub config: Config,
    pub db_client: Option<Arc<Client>>,
}

pub async fn initialize_bot(config_path: &str) -> Result<InitResources> {
    // dotenv::dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config_content = fs::read_to_string(config_path).context("Failed to read config file")?;
    let config: Config = toml::from_str(&config_content).context("Failed to parse config file")?;

    // let private_key = env::var("WALLET_SECRET").expect("WALLET_SECRET not set");
    let wallet = LocalWallet::from_str(&config.wallet_secret).context("Invalid wallet secret")?;

    // Initialize database connection if database_url is provided
    let db_client = if let Some(database_url) = &config.database_url {
        let (client, connection) = tokio_postgres::connect(database_url, NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });
        Some(Arc::new(client))
    } else {
        None
    };

    let http_client = HttpClient::new(config.is_mainnet, wallet.clone()).await?;
    let asset_info = http_client.get_asset_info(&config.coin).unwrap();

    let ws_manager = WebSocketManager::new(config.is_mainnet, db_client.clone()).await;

    // Subscribe to necessary data
    ws_manager.subscribe(Subscription::AllMids).await?;
    ws_manager
        .subscribe(Subscription::Trades {
            coin: asset_info.internal_name.clone(),
        })
        .await?;
    ws_manager
        .subscribe(Subscription::L2Book {
            coin: asset_info.internal_name.clone(),
        })
        .await?;
    ws_manager
        .subscribe(Subscription::UserFills {
            user: wallet.address(),
        })
        .await?;

    Ok(InitResources {
        ws_manager,
        http_client,
        wallet,
        config,
        db_client,
    })
}
