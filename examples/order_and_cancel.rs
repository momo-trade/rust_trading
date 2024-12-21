use dotenv::dotenv;
use ethers::signers::{LocalWallet, Signer};
use log::{error, info};
use rust_trading::hyperliquid::http::HttpClient;
use rust_trading::hyperliquid::order::LimitOrderParams;
use std::env;
use std::str::FromStr;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    dotenv().ok();

    let private_key = env::var("WALLET_SECRET").expect("WALLET_SECRET not set");
    let wallet = LocalWallet::from_str(&private_key).expect("Invalid private key");

    info!("Wallet address: {:?}", wallet.address());

    let client = HttpClient::new(true, wallet).await.unwrap();

    let order = LimitOrderParams::new("HYPE/USDC".to_string(), true, 8.0, 0.5);

    let order_id = match client.limit_order(order).await {
        Ok(id) => id,
        Err(err) => {
            error!("{}", err);
            return;
        }
    };

    match client.cancel_order("HYPE/USDC".to_string(), order_id).await {
        Ok(msg) => info!("Success: {}", msg),
        Err(err) => error!("{}", err),
    }
}
