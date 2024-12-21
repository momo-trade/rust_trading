use dotenv::dotenv;
use ethers::signers::{LocalWallet, Signer};
use log::{error, info};
use rust_trading::hyperliquid::http::HttpClient;
use rust_trading::utils::time::{calculate_time_range, unix_time_to_jst};
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

    // Fetch the user's state and log the result or an error message
    match client.fetch_user_state(address).await {
        Ok(user_state) => {
            info!("User state: {:#?}", user_state);
        }
        Err(err) => {
            error!("Failed to fetch user state: {}", err);
        }
    }

    // Fetch the mids data (midpoint prices) for all coins and log the result or an error message
    match client.fetch_all_mids().await {
        Ok(mids) => {
            info!("All mids: {:#?}", mids);
        }
        Err(err) => {
            error!("Failed to fetch all mids: {}", err);
        }
    }

    // Fetch the user's trade fills and log the first fill or an error message
    match client.fetch_user_fills(address).await {
        Ok(user_fills) => {
            info!("User fills: {:#?}", user_fills.first().unwrap());
        }
        Err(err) => {
            error!("Failed to fetch user fills: {}", err);
        }
    }

    // Fetch recent trades for a specific coin and log the first trade or an error message
    let coin = "BTC";
    match client.fetch_trades(coin).await {
        Ok(trades) => {
            info!("Trades: {:#?}", trades.first().unwrap());
        }
        Err(err) => {
            error!("Failed to fetch trades: {}", err);
        }
    }

    // Calculate the time range for fetching candle data (last 24 hours in this case)
    let (start_time, end_time) = calculate_time_range(24);
    info!("start_time: {}, end_time: {}", start_time, end_time);
    match client.fetch_candles(coin, "1m", start_time, end_time).await {
        Ok(candles) => match candles.first() {
            // If candles are returned, log the details of the first candle
            Some(candle) => {
                info!(
                    "timestamp: {}, open: {}, high: {}, low: {}, close: {}",
                    unix_time_to_jst(candle.time_open),
                    candle.open,
                    candle.high,
                    candle.low,
                    candle.close
                );
            }
            None => {
                error!("No candles found");
            }
        },
        Err(err) => {
            error!("Failed to fetch candles: {}", err);
        }
    }

    match client.fetch_funding_history(coin, start_time, None).await {
        Ok(funding_history) => {
            info!("Funding history: {:#?}", funding_history);
        }
        Err(err) => {
            error!("Failed to fetch funding history: {}", err);
        }
    }

    match client
        .fetch_user_funding_history(address, start_time, None)
        .await
    {
        Ok(user_funding_history) => {
            info!("User funding history: {:#?}", user_funding_history);
        }
        Err(err) => {
            error!("Failed to fetch user funding history: {}", err);
        }
    }
}
