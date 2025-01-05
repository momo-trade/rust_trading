use crate::hyperliquid::model::CustomUserFills;
use chrono::{DateTime, Local, TimeZone, Utc};
use plotters::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

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
        } else if (position.amount > 0.0 && fill_amount > 0.0)
            || (position.amount < 0.0 && fill_amount < 0.0)
        {
            // Same direction trade (increase position)
            let total_cost = position.amount * position.average_price + fill_amount * fill.price;
            position.amount += fill_amount;
            position.average_price = total_cost / position.amount.abs();
        } else {
            // Opposite direction trade (decrease or reverse position)
            let previous_amount = position.amount;
            position.amount += fill_amount;

            if position.amount == 0.0 {
                // ポジション完全解消
                position.average_price = 0.0;
            }

            // Realized PnL calculation
            position.pnl.realized +=
                (fill.price - position.average_price) * previous_amount.abs().min(fill.size);
        }

        // Update realized PnL and fees
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

    pub fn create_pnl_chart(
        &self,
        fills: &[CustomUserFills],
        output_path: &str,
    ) -> Result<(), Box<dyn Error>> {
        let root = BitMapBackend::new(output_path, (1024, 768)).into_drawing_area();
        root.fill(&RGBColor(30, 30, 30))?; // Set background to dark gray

        let mut pnl_by_coin: HashMap<String, Vec<(DateTime<Utc>, f64)>> = HashMap::new();

        // Calculate cumulative PnL for each coin
        for fill in fills {
            if let Some(dt) = Utc
                .timestamp_opt(
                    fill.timestamp / 1000,
                    ((fill.timestamp % 1000) * 1_000_000) as u32,
                )
                .single()
            {
                let entry = pnl_by_coin.entry(fill.coin.clone()).or_default();
                let last_pnl = entry.last().map(|&(_, pnl)| pnl).unwrap_or(0.0);
                entry.push((dt, last_pnl + fill.closed_pnl));
            }
        }

        let min_pnl = pnl_by_coin
            .values()
            .flat_map(|pnls| pnls.iter().map(|&(_, pnl)| pnl))
            .fold(f64::INFINITY, f64::min);

        let max_pnl = pnl_by_coin
            .values()
            .flat_map(|pnls| pnls.iter().map(|&(_, pnl)| pnl))
            .fold(f64::NEG_INFINITY, f64::max);

        let range_margin = (max_pnl - min_pnl) * 0.05;
        let y_min = min_pnl - range_margin;
        let y_max = max_pnl + range_margin;

        let min_date = pnl_by_coin
            .values()
            .flat_map(|pnls| pnls.iter().map(|&(date, _)| date))
            .min()
            .unwrap();

        let max_date = pnl_by_coin
            .values()
            .flat_map(|pnls| pnls.iter().map(|&(date, _)| date))
            .max()
            .unwrap();

        let total_duration = max_date.signed_duration_since(min_date);
        let date_format = if total_duration.num_days() == 0 {
            "%H:%M"
        } else if total_duration.num_days() < 30 {
            "%Y-%m-%d %H:%M"
        } else {
            "%Y-%m-%d"
        };

        let y_label_area_size = if y_max.abs() > 10_000.0 { 80 } else { 50 };

        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Cumulative PnL",
                ("sans-serif", 20).into_font().color(&WHITE),
            )
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(y_label_area_size)
            .build_cartesian_2d(
                min_date.with_timezone(&Local)..max_date.with_timezone(&Local),
                y_min..y_max,
            )?;

        chart
            .configure_mesh()
            // .disable_x_mesh() // Disable X-axis mesh lines
            .disable_y_mesh() // Disable Y-axis mesh lines
            .x_desc("Date")
            .y_desc("PnL")
            .x_label_formatter(&|date| date.with_timezone(&Local).format(date_format).to_string())
            .x_label_style(("sans-serif", 15).into_font().color(&WHITE))
            .y_label_style(("sans-serif", 15).into_font().color(&WHITE))
            .axis_style(WHITE) // Set axis lines color to white
            .draw()?;

        let colors = [
            RGBColor(0, 114, 178),   // Blue
            RGBColor(0, 158, 115),   // Green
            RGBColor(213, 94, 0),    // Orange
            RGBColor(204, 121, 167), // Purple
        ];

        for (i, (coin, pnls)) in pnl_by_coin.iter().enumerate() {
            let color = colors[i % colors.len()]; // Directly use color value
            chart
                .draw_series(LineSeries::new(
                    pnls.iter()
                        .map(|&(date, pnl)| (date.with_timezone(&Local), pnl)),
                    &color,
                ))?
                .label(coin)
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color));
        }

        chart
            .configure_series_labels()
            .background_style(RGBAColor(0, 0, 0, 0.0))
            .border_style(WHITE)
            .label_font(("sans-serif", 15).into_font().color(&WHITE))
            .draw()?;

        Ok(())
    }
}

impl Default for PortfolioManager {
    fn default() -> Self {
        Self::new()
    }
}
