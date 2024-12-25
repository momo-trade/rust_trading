use crate::hyperliquid::model::CustomUserFills;
use anyhow::Result;
use ethers::types::H160;
use tokio_postgres::Client;

pub async fn save_fills_to_db(
    client: &Client,
    fills: &[CustomUserFills],
    user: H160,
) -> Result<()> {
    for fill in fills {
        client.execute(
            "INSERT INTO user_fills (user_address, closed_pnl, coin, crossed, dir, hash, order_id, price, side, start_position, size, timestamp, fee)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
            &[
                &user.to_string(),
                &fill.closed_pnl,
                &fill.coin,
                &fill.crossed,
                &fill.dir,
                &fill.hash,
                &fill.order_id,
                &fill.price,
                &fill.side,
                &fill.start_position,
                &fill.size,
                &fill.timestamp,
                &fill.fee,
            ],
        ).await?;
    }
    Ok(())
}
