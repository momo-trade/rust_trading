use hyperliquid_rust_sdk::BaseUrl;
use hyperliquid_rust_sdk::Subscription;
use log::{info, warn};
use rust_trading::hyperliquid::websocket::WebSocketManager;
use rust_trading::utils::time::unix_time_to_jst;

#[tokio::main]
async fn main() {
    // ログの初期化
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // WebSocketManager の初期化
    let ws_manager = WebSocketManager::new(BaseUrl::Mainnet).await;

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

    ws_manager.set_max_trades(200).await;
    ws_manager.start();

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        let all_mids = ws_manager.get_all_mids().await;
        let eth = match all_mids.get("ETH") {
            Some(eth) => eth,
            None => {
                info!("ETH not found in AllMids data");
                continue;
            }
        };
        info!("Mids data for ETH: {}", eth);

        let trades = ws_manager.get_trades().await;
        info!("Latest Trades data: {}", trades.len());
        if let Some(latest_trade) = trades.last() {
            info!(
                "coin: {}, side: {}, prise: {}, size: {}",
                latest_trade.coin, latest_trade.side, latest_trade.price, latest_trade.size
            );
        } else {
            info!("Latest Trades data is empty");
        }

        let candles = ws_manager.get_candles().await;
        info!("Candles data {}", candles.len());
        if candles.len() >= 2 {
            if let Some(previous_candle) = candles.get(candles.len() - 2) {
                info!(
                    "Previous Candle: open_time: {}, open: {}, high: {}, low: {},close: {}",
                    unix_time_to_jst(previous_candle.time_open),
                    previous_candle.open,
                    previous_candle.high,
                    previous_candle.low,
                    previous_candle.close
                );
            }
        } else {
            warn!("Not enough candles to display the previous one.");
        }
    }
}
