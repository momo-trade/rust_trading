use dotenv::dotenv;
use ethers::signers::{LocalWallet, Signer};
use log::{error, info};
use rust_trading::hyperliquid::http::HttpClient;
use rust_trading::hyperliquid::order::LimitOrderParams;
use std::env;
use std::str::FromStr;

#[tokio::main]
async fn main() {
    // Initialize the logger with environment variables
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Load environment variables from a .env file
    dotenv().ok();

    // Retrieve the private key from the environment variables
    let private_key = env::var("WALLET_SECRET").expect("WALLET_SECRET not set");

    // Create a LocalWallet instance from the private key
    let wallet = LocalWallet::from_str(&private_key).expect("Invalid private key");
    info!("Wallet address: {:?}", wallet.address());

    // Initialize the Hyperliquid HTTP client in mainnet mode
    let client = HttpClient::new(true, wallet).await.unwrap();

    // Define a limit order with the desired parameters
    let order = LimitOrderParams::new("HYPE/USDC".to_string(), true, 8.0, 0.5);

    // Submit the limit order and retrieve the order ID
    let order_id = match client.limit_order(order).await {
        Ok(id) => id, // Successfully retrieved the order ID
        Err(err) => {
            // Log any errors encountered while submitting the order
            error!("{}", err);
            return;
        }
    };

    // Attempt to cancel the previously created order
    match client.cancel_order("HYPE/USDC".to_string(), order_id).await {
        Ok(msg) => info!("Success: {}", msg), // Log success message
        Err(err) => error!("{}", err),        // Log any errors encountered
    }
}
