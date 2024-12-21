use hyperliquid_rust_sdk::{CandleData, Trade};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomTrade {
    pub coin: String,
    pub side: String,
    pub price: f64,
    pub size: f64,
    pub timestamp: u64,
    pub hash: String,
    pub trade_id: u64,
}

impl From<Trade> for CustomTrade {
    fn from(trade: Trade) -> Self {
        CustomTrade {
            coin: trade.coin,
            side: trade.side,
            price: trade.px.parse().unwrap_or(0.0), // Convert the "px" field from string to f64
            size: trade.sz.parse().unwrap_or(0.0),  // Convert the "sz" field from string to f64
            timestamp: trade.time,
            hash: trade.hash,
            trade_id: trade.tid,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomCandle {
    pub coin: String,
    pub interval: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub num_trades: u64,
    pub time_close: u64,
    pub time_open: u64,
}

impl From<CandleData> for CustomCandle {
    fn from(candle: CandleData) -> Self {
        CustomCandle {
            coin: candle.coin,
            interval: candle.interval,
            open: candle.open.parse().unwrap_or(0.0), // Convert the "open" field from string to f64
            high: candle.high.parse().unwrap_or(0.0), // Convert the "high" field from string to f64
            low: candle.low.parse().unwrap_or(0.0),   // Convert the "low" field from string to f64
            close: candle.close.parse().unwrap_or(0.0), // Convert the "close" field from string to f64
            volume: candle.volume.parse().unwrap_or(0.0), // Convert the "volume" field from string to f64
            num_trades: candle.num_trades,
            time_close: candle.time_close,
            time_open: candle.time_open,
        }
    }
}
