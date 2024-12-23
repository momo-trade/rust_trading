use crate::hyperliquid::http::HttpClient;
use crate::hyperliquid::websocket::WebSocketManager;
use anyhow::Result;
use ethers::signers::LocalWallet;
use std::env;
use std::str::FromStr;
use std::sync::Arc;

pub struct InitOptions {
    pub is_mainnet: bool,
    pub coin: String,
}

pub struct InitResources {
    pub ws_manager: Arc<WebSocketManager>,
    pub http_client: HttpClient,
    pub wallet: LocalWallet,
}

pub async fn initialize_bot(options: InitOptions) -> Result<InitResources> {
    dotenv::dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let private_key = env::var("WALLET_SECRET").expect("WALLET_SECRET not set");
    let wallet = LocalWallet::from_str(&private_key)?;

    let ws_manager = WebSocketManager::new(options.is_mainnet).await;
    let http_client = HttpClient::new(options.is_mainnet, wallet.clone()).await?;

    Ok(InitResources {
        ws_manager,
        http_client,
        wallet,
    })
}
