use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Position {
    pub coin: String,
    pub amount: f64,
    pub average_price: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pnl {
    pub coin: String,
    pub realized: f64,
    pub unrealized: f64,
}

pub struct PortfolioManager {
    positions: HashMap<String, Position>,
    realized_pnl: f64, // Cumulative Profit and Loss(PnL)
}

impl PortfolioManager {
    pub fn new() -> Self {
        PortfolioManager {
            positions: HashMap::new(),
            realized_pnl: 0.0,
        }
    }

    pub fn update_position(&mut self) {
        todo!("update position");
    }

    pub fn calculate_unrealized_pnl(&self) -> Option<Pnl> {
        todo!("calculate unrealized pnl");
    }

    pub fn get_positions(&self) -> &HashMap<String, Position> {
        &self.positions
    }

    pub fn get_position(&self, coin: &str) -> Option<&Position> {
        self.positions.get(coin)
    }

    pub fn get_realized_pnl(&self) -> f64 {
        self.realized_pnl
    }
}

impl Default for PortfolioManager {
    fn default() -> Self {
        Self::new()
    }
}
