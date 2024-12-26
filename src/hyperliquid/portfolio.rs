use crate::hyperliquid::model::CustomUserFills;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Position {
    pub coin: String,
    pub amount: f64,
    pub average_price: f64,
    pub pnl: Pnl,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pnl {
    pub realized: f64,
    pub unrealized: f64,
    pub fee_in_token: f64,
    pub fee_in_usdc: f64,
}

#[derive(Debug, Clone)]
pub struct PortfolioManager {
    positions: HashMap<String, Position>,
}

impl PortfolioManager {
    pub fn new() -> Self {
        PortfolioManager {
            positions: HashMap::new(),
        }
    }

    pub fn update_position(&mut self, fill: &CustomUserFills) {
        let position = self.positions.entry(fill.coin.clone()).or_insert(Position {
            coin: fill.coin.clone(),
            amount: 0.0,
            average_price: 0.0,
            pnl: Pnl {
                realized: 0.0,
                unrealized: 0.0,
                fee_in_token: 0.0,
                fee_in_usdc: 0.0,
            },
        });

        let is_buy = fill.side == "B";
        let fill_amount = if is_buy { fill.size } else { -fill.size };

        // Update position
        if position.amount == 0.0 {
            // New position
            position.amount = fill_amount;
            position.average_price = fill.price;
        } else {
            // Update existing position
            if (position.amount > 0.0 && fill_amount > 0.0)
                || (position.amount < 0.0 && fill_amount < 0.0)
            {
                // Same direction trade (increase position)
                let total_cost =
                    position.amount * position.average_price + fill_amount * fill.price;
                position.amount += fill_amount;
                position.average_price = total_cost / position.amount.abs();
            } else {
                // Opposite direction trade (decrease or reverse position)
                position.amount += fill_amount;
                if position.amount != 0.0 {
                    position.average_price = fill.price;
                }
            }
        }

        // Update realized PnL and fees
        position.pnl.realized += fill.closed_pnl;
        if is_buy {
            position.pnl.fee_in_token += fill.fee; // Fee in token for buy
            position.amount -= fill.fee; // Subtract fee from amount
        } else {
            position.pnl.fee_in_usdc += fill.fee; // Fee in USDC for sell
            position.pnl.realized -= fill.fee; // Subtract fee from realized PnL
        }
    }

    pub fn get_unrealized_pnl(&self, coin: &str, current_price: f64) -> f64 {
        if let Some(position) = self.positions.get(coin) {
            if position.amount != 0.0 {
                return (current_price - position.average_price) * position.amount;
            }
        }
        0.0
    }

    pub fn get_positions(&self) -> &HashMap<String, Position> {
        &self.positions
    }

    pub fn get_position(&self, coin: &str) -> Option<&Position> {
        self.positions.get(coin)
    }

    pub fn get_total_realized_pnl(&self) -> f64 {
        self.positions.values().map(|p| p.pnl.realized).sum()
    }

    pub fn get_total_and_individual_unrealized_pnl(
        &self,
        current_prices: &HashMap<String, f64>,
    ) -> (f64, HashMap<String, Pnl>) {
        let mut total_unrealized_pnl = 0.0;
        let mut individual_pnls = HashMap::new();

        for (coin, position) in &self.positions {
            if let Some(&current_price) = current_prices.get(coin) {
                let unrealized_pnl = if position.amount != 0.0 {
                    (current_price - position.average_price) * position.amount
                } else {
                    0.0
                };

                total_unrealized_pnl += unrealized_pnl;

                individual_pnls.insert(
                    coin.clone(),
                    Pnl {
                        realized: position.pnl.realized,
                        unrealized: unrealized_pnl,
                        fee_in_token: position.pnl.fee_in_token,
                        fee_in_usdc: position.pnl.fee_in_usdc,
                    },
                );
            }
        }

        (total_unrealized_pnl, individual_pnls)
    }
}

impl Default for PortfolioManager {
    fn default() -> Self {
        Self::new()
    }
}
