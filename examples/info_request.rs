use dotenv::dotenv;
use ethers::signers::{LocalWallet, Signer};
use log::{error, info};
use rust_trading::hyperliquid::http::HttpClient;
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
    let address = wallet.address();

    // Initialize the Hyperliquid HTTP client in mainnet mode
    let client = HttpClient::new(true, wallet).await.unwrap();

    // Fetch and display open orders for the wallet address
    match client.fetch_open_orders(address).await {
        Ok(open_orders) => {
            info!("Open orders: {:#?}", open_orders);
        }
        Err(err) => {
            error!("Failed to fetch open orders: {}", err);
        }
    }

    // Fetch and display token balances for the wallet address
    match client.fetch_token_balances(address).await {
        Ok(token_balances) => {
            // info!("Token balances: {:#?}", token_balances);
            let usdc = match token_balances.iter().find(|balance| balance.coin == "USDC") {
                Some(balance) => balance,
                None => {
                    error!("USDC balance not found");
                    return;
                }
            };
            info!("USDC balance: {}", usdc.total);
        }
        Err(err) => {
            error!("Failed to fetch token balances: {}", err);
        }
    }

    // Query and display the status of a specific order using its order ID
    let oid = 57281516303;
    match client.query_order_status(address, oid).await {
        Ok(order_status) => {
            info!("Order status: {:#?}", order_status);
        }
        Err(err) => {
            error!("Failed to query order status: {}", err);
        }
    }
}
