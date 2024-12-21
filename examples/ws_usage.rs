use hyperliquid_rust_sdk::BaseUrl;
use hyperliquid_rust_sdk::Subscription;
use log::{info, warn};
use rust_trading::hyperliquid::websocket::WebSocketManager;
use rust_trading::utils::time::unix_time_to_jst;

#[tokio::main]
async fn main() {
    // Initialize the logger for logging purposes
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Initialize the WebSocketManager with the mainnet URL
    let ws_manager = WebSocketManager::new(BaseUrl::Mainnet).await;

    // Add subscriptions for AllMids, Trades (BTC), and Candle (BTC with a 1m interval)
    ws_manager.add_subscription(Subscription::AllMids).await;
    ws_manager
        .add_subscription(Subscription::Trades { coin: "BTC".into() })
        .await;
    ws_manager
        .add_subscription(Subscription::Candle {
            coin: "BTC".into(),
            interval: "1m".into(),
        })
        .await;

    // Set the maximum number of trades to store
    ws_manager.set_max_trades(200).await;

    // Start the WebSocket manager to begin receiving messages
    ws_manager.start();

    // Main loop to periodically check and log WebSocket data
    loop {
        // Wait for 10 seconds before the next iteration
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        // Fetch the latest AllMids data
        let all_mids = ws_manager.get_all_mids().await;

        // Try to get the data for ETH from AllMids
        let eth = match all_mids.get("ETH") {
            Some(eth) => eth,
            None => {
                // Log if ETH data is not found
                info!("ETH not found in AllMids data");
                continue; // Skip the rest of the loop
            }
        };
        // Log the mids data for ETH
        info!("Mids data for ETH: {}", eth);

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
