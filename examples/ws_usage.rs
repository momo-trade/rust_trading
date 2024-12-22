use dotenv::dotenv;
use ethers::signers::LocalWallet;
use hyperliquid_rust_sdk::BaseUrl;
use hyperliquid_rust_sdk::Subscription;
use log::{info, warn};
use rust_trading::hyperliquid::http::HttpClient;
use rust_trading::hyperliquid::websocket::WebSocketManager;
use rust_trading::utils::time::unix_time_to_jst;
use std::env;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger for logging purposes
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Load environment variables from a .env file
    dotenv().ok();

    // Retrieve the private key from the environment variables
    let private_key = env::var("WALLET_SECRET").expect("WALLET_SECRET not set");

    // Create a LocalWallet instance from the private key
    let wallet = LocalWallet::from_str(&private_key).expect("Invalid private key");

    // Initialize the WebSocketManager with the mainnet URL
    let ws_manager = WebSocketManager::new(BaseUrl::Mainnet).await;
    let http_manager = HttpClient::new(true, wallet).await.unwrap();
    // Set the maximum number of trades to store
    ws_manager.set_max_trades(200).await;

    let coin = "HYPE/USDC";
    let asset_info = http_manager.get_asset_info(coin).unwrap();
    let internal_name = asset_info.internal_name.clone();

    info!("Asset info: {:#?}", asset_info);

    ws_manager.subscribe(Subscription::AllMids).await?;
    ws_manager
        .subscribe(Subscription::Trades {
            coin: internal_name.clone(),
        })
        .await?;
    ws_manager
        .subscribe(Subscription::Candle {
            coin: internal_name.clone(),
            interval: "1m".to_string(),
        })
        .await?;

    // Main loop to periodically check and log WebSocket data
    loop {
        // Wait for 10 seconds before the next iteration
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        // Fetch the latest AllMids data
        let all_mids = ws_manager.get_all_mids().await;

        // Try to get the data for coin from AllMids
        let target = match all_mids.get(internal_name.as_str()) {
            Some(eth) => eth,
            None => {
                // Log if coin data is not found
                info!("{} not found in AllMids data", coin);
                continue; // Skip the rest of the loop
            }
        };
        // Log the mids data for coin
        info!("Mids data for {}: {}", coin, target);

        // Fetch the latest trades data
        let trades = ws_manager.get_trades().await;
        info!("Latest Trades data: {}", trades.len());

        // Log the most recent trade if available
        if let Some(latest_trade) = trades.last() {
            info!(
                "coin: {}, side: {}, price: {}, size: {}",
                latest_trade.coin, latest_trade.side, latest_trade.price, latest_trade.size
            );
        } else {
            info!("Latest Trades data is empty");
        }

        // Fetch the latest candle data
        let candles = ws_manager.get_candles().await;
        info!("Candles data {}", candles.len());

        // Log the second to last candle if at least two candles are available
        if candles.len() >= 2 {
            if let Some(previous_candle) = candles.get(candles.len() - 2) {
                info!(
                    "Previous Candle: open_time: {}, open: {}, high: {}, low: {}, close: {}",
                    unix_time_to_jst(previous_candle.time_open),
                    previous_candle.open,
                    previous_candle.high,
                    previous_candle.low,
                    previous_candle.close
                );
            }
        } else {
            // Warn if there aren't enough candles to display the second to last one
            warn!("Not enough candles to display the previous one.");
        }
    }
}
