use ethers::types::H160;
use hyperliquid_rust_sdk::Subscription as HyperliquidSubscription;

pub enum Subscription {
    AllMids,
    Trades { coin: String },
    Candle { coin: String, interval: String },
    UserFills { user: H160 },
}

impl From<Subscription> for HyperliquidSubscription {
    fn from(sub: Subscription) -> Self {
        match sub {
            Subscription::AllMids => HyperliquidSubscription::AllMids,
            Subscription::Trades { coin } => HyperliquidSubscription::Trades { coin },
            Subscription::Candle { coin, interval } => {
                HyperliquidSubscription::Candle { coin, interval }
            }
            Subscription::UserFills { user } => HyperliquidSubscription::UserFills { user },
        }
    }
}
