use hyperliquid_rust_sdk::{
    CandleData, CandlesSnapshotResponse, OpenOrdersResponse, OrderStatusResponse,
    RecentTradesResponse, Trade, UserFillsResponse, UserTokenBalance,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomTrade {
    pub coin: String,
    pub side: String,
    pub price: f64,
    pub size: f64,
    pub timestamp: u64,
    pub hash: String,
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
        }
    }
}

impl From<RecentTradesResponse> for CustomTrade {
    fn from(recent_trade: RecentTradesResponse) -> Self {
        CustomTrade {
            coin: recent_trade.coin,
            side: recent_trade.side,
            price: recent_trade.px.parse().unwrap_or(0.0), // Convert "px" from String to f64
            size: recent_trade.sz.parse().unwrap_or(0.0),  // Convert "sz" from String to f64
            timestamp: recent_trade.time,
            hash: recent_trade.hash,
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

impl From<CandlesSnapshotResponse> for CustomCandle {
    fn from(candle: CandlesSnapshotResponse) -> Self {
        CustomCandle {
            coin: candle.coin,
            interval: candle.candle_interval,
            open: candle.open.parse().unwrap_or(0.0), // Convert the "open" field from string to f64
            high: candle.high.parse().unwrap_or(0.0), // Convert the "high" field from string to f64
            low: candle.low.parse().unwrap_or(0.0),   // Convert the "low" field from string to f64
            close: candle.close.parse().unwrap_or(0.0), // Convert the "close" field from string to f64
            volume: candle.vlm.parse().unwrap_or(0.0), // Convert the "volume" field from string to f64
            num_trades: candle.num_trades,
            time_close: candle.time_close,
            time_open: candle.time_open,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomOpenOrders {
    pub coin: String,
    pub price: f64,
    pub order_id: u64,
    pub side: String,
    pub size: f64,
    pub timestamp: u64,
}

impl From<OpenOrdersResponse> for CustomOpenOrders {
    fn from(order: OpenOrdersResponse) -> Self {
        CustomOpenOrders {
            coin: order.coin,
            price: order.limit_px.parse().unwrap_or(0.0), // Convert the "price" field from string to f64
            order_id: order.oid,
            side: order.side,
            size: order.sz.parse().unwrap_or(0.0), // Convert the "size" field from string to f64
            timestamp: order.timestamp,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomUserTokenBalance {
    pub coin: String,
    pub hold: f64,
    pub total: f64,
}

impl From<UserTokenBalance> for CustomUserTokenBalance {
    fn from(balance: UserTokenBalance) -> Self {
        CustomUserTokenBalance {
            coin: balance.coin,
            hold: balance.hold.parse().unwrap_or(0.0), // Convert the "hold" field from string to f64
            total: balance.total.parse().unwrap_or(0.0), // Convert the "total" field from string to f64
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomOrderStatus {
    pub coin: String,
    pub side: String,
    pub price: f64,
    pub size: f64,
    pub order_id: u64,
    pub timestamp: u64,
    pub status: String, // "filled" | "open" | "canceled" | "triggered" | "rejected" | "marginCanceled"
    pub reduce_only: bool,
    pub order_type: String,
    pub tif: String,
}

impl From<OrderStatusResponse> for CustomOrderStatus {
    fn from(response: OrderStatusResponse) -> Self {
        if let Some(order_info) = response.order {
            let order = order_info.order;
            CustomOrderStatus {
                coin: order.coin,
                side: order.side,
                price: order.limit_px.parse().unwrap_or(0.0), // Convert the "price" field from string to f64
                size: order.sz.parse().unwrap_or(0.0), // Convert the "size" field from string to f64
                order_id: order.oid,
                timestamp: order.timestamp,
                status: order_info.status, // Use the status field from OrderInfo
                reduce_only: order.reduce_only,
                order_type: order.order_type,
                tif: order.tif,
            }
        } else {
            // Handle case where `response.order` is `None`
            CustomOrderStatus {
                coin: "".to_string(),
                side: "".to_string(),
                price: 0.0,
                size: 0.0,
                order_id: 0,
                timestamp: 0,
                status: "unknown".to_string(),
                reduce_only: false,
                order_type: "".to_string(),
                tif: "".to_string(),
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomUserFills {
    pub closed_pnl: f64,
    pub coin: String,
    pub crossed: bool,
    pub dir: String,
    pub hash: String,
    pub order_id: u64,
    pub price: f64,
    pub side: String,
    pub start_position: f64,
    pub size: f64,
    pub timestamp: u64,
}

impl From<UserFillsResponse> for CustomUserFills {
    fn from(fills: UserFillsResponse) -> Self {
        CustomUserFills {
            closed_pnl: fills.closed_pnl.parse().unwrap_or(0.0), // Convert the "closed_pnl" field from string to f64
            coin: fills.coin,
            crossed: fills.crossed,
            dir: fills.dir,
            hash: fills.hash,
            order_id: fills.oid,
            price: fills.px.parse().unwrap_or(0.0), // Convert the "price" field from string to f64
            side: fills.side,
            start_position: fills.start_position.parse().unwrap_or(0.0), // Convert the "start_position" field from string to f64
            size: fills.sz.parse().unwrap_or(0.0), // Convert the "size" field from string to f64
            timestamp: fills.time,
        }
    }
}
