use hyperliquid_rust_sdk::{
    CandleData, CandlesSnapshotResponse, L2BookData, L2SnapshotResponse,
    OpenOrdersResponse, OrderStatusResponse, RecentTradesResponse, Trade, TradeInfo,
    UserFillsResponse, UserTokenBalance,
};
use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

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
    pub fee: f64,
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
            fee: fills.fee.parse().unwrap_or(0.0), // Convert the "fee" field from string to f64
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomL2Book {
    pub coin: String,
    // pub levels: [Vec<CustomWsLevel>; 2], // bids: levels[0], asks: levels[1]
    pub bid_levels: Vec<CustomLevel>,
    pub ask_levels: Vec<CustomLevel>,
    #[serde(rename = "time")]
    pub timestamp: u64,
}

impl From<L2SnapshotResponse> for CustomL2Book {
    fn from(response: L2SnapshotResponse) -> Self {
        let bid_levels = response
            .levels
            .first()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|level| CustomLevel {
                price: level.px.parse().unwrap_or(0.0),
                size: level.sz.parse().unwrap_or(0.0),
                num_orders: level.n,
            })
            .collect();

        let ask_levels = response
            .levels
            .get(1)
            .unwrap_or(&Vec::new())
            .iter()
            .map(|level| CustomLevel {
                price: level.px.parse().unwrap_or(0.0),
                size: level.sz.parse().unwrap_or(0.0),
                num_orders: level.n,
            })
            .collect();

        CustomL2Book {
            coin: response.coin,
            bid_levels,
            ask_levels,
            timestamp: response.time,
        }
    }
}

impl From<L2BookData> for CustomL2Book {
    fn from(response: L2BookData) -> Self {
        let bid_levels = response
            .levels
            .first()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|level| CustomLevel {
                price: level.px.parse().unwrap_or(0.0),
                size: level.sz.parse().unwrap_or(0.0),
                num_orders: level.n,
            })
            .collect();

        let ask_levels = response
            .levels
            .get(1)
            .unwrap_or(&Vec::new())
            .iter()
            .map(|level| CustomLevel {
                price: level.px.parse().unwrap_or(0.0),
                size: level.sz.parse().unwrap_or(0.0),
                num_orders: level.n,
            })
            .collect();

        CustomL2Book {
            coin: response.coin,
            bid_levels,
            ask_levels,
            timestamp: response.time,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomLevel {
    #[serde(rename = "px", deserialize_with = "string_to_f64")]
    pub price: f64,
    #[serde(rename = "sz", deserialize_with = "string_to_f64")]
    pub size: f64,
    #[serde(rename = "n")]
    pub num_orders: u64,
}

impl From<TradeInfo> for CustomUserFills {
    fn from(fills: TradeInfo) -> Self {
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
            fee: fills.fee.parse().unwrap_or(0.0), // Convert the "fee" field from string to f64
        }
    }
}

// カスタムデシリアライザ: String -> f64
fn string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}

// user_balances 用のカスタムデシリアライザ
fn deserialize_user_balances<'de, D>(deserializer: D) -> Result<Vec<(String, f64)>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(UserBalancesVisitor)
}

// Visitor を実装
struct UserBalancesVisitor;

impl<'de> Visitor<'de> for UserBalancesVisitor {
    type Value = Vec<(String, f64)>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a sequence of (String, String) where the second String is a number")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut balances = Vec::new();

        while let Some((address, balance_str)) = seq.next_element::<(String, String)>()? {
            let balance = balance_str.parse::<f64>().map_err(de::Error::custom)?;
            balances.push((address, balance));
        }

        Ok(balances)
    }
}

// JSONデシリアライズ対象の構造体
#[derive(Debug, Deserialize)]
pub struct Genesis {
    #[serde(
        rename = "userBalances",
        deserialize_with = "deserialize_user_balances"
    )]
    pub user_balances: Vec<(String, f64)>,

    #[serde(rename = "existingTokenBalances")]
    pub existing_token_balances: Option<Vec<(String, String)>>,
}

#[derive(Debug, Deserialize)]
pub struct TokenDetails {
    pub name: String,
    #[serde(rename = "maxSupply", deserialize_with = "string_to_f64")]
    pub max_supply: f64,
    #[serde(rename = "totalSupply", deserialize_with = "string_to_f64")]
    pub total_supply: f64,
    #[serde(rename = "circulatingSupply", deserialize_with = "string_to_f64")]
    pub circulating_supply: f64,
    #[serde(rename = "szDecimals")]
    pub size_decimals: u32,
    #[serde(rename = "weiDecimals")]
    pub wei_decimals: u32,
    #[serde(rename = "midPx", deserialize_with = "string_to_f64")]
    pub mid_price: f64,
    #[serde(rename = "markPx", deserialize_with = "string_to_f64")]
    pub mark_price: f64,
    #[serde(rename = "prevDayPx", deserialize_with = "string_to_f64")]
    pub prev_day_price: f64,
    pub genesis: Option<Genesis>,
    pub deployer: Option<String>,
    #[serde(rename = "deployGas", deserialize_with = "string_to_f64")]
    pub deploy_gas: f64,
    #[serde(rename = "deployTime")]
    pub deploy_time: Option<String>,
    #[serde(rename = "seededUsdc", deserialize_with = "string_to_f64")]
    pub seeded_usdc: f64,
    #[serde(rename = "nonCirculatingUserBalances")]
    pub non_circulating_user_balances: Vec<(String, String)>,
    #[serde(rename = "futureEmissions", deserialize_with = "string_to_f64")]
    pub future_emissions: f64,
}
