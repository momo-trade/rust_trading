use hyperliquid_rust_sdk::Subscription as HyperliquidSubscription;

pub enum Subscription {
    AllMids,
    Trades { coin: String },
    Candle { coin: String, interval: String },
}

impl From<Subscription> for HyperliquidSubscription {
    fn from(sub: Subscription) -> Self {
        match sub {
            Subscription::AllMids => HyperliquidSubscription::AllMids,
            Subscription::Trades { coin } => HyperliquidSubscription::Trades { coin },
            Subscription::Candle { coin, interval } => {
                HyperliquidSubscription::Candle { coin, interval }
            }
        }
    }
}
