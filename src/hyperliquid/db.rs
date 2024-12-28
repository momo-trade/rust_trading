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

pub async fn load_fills_from_db(client: &Client, user: H160) -> Result<Vec<CustomUserFills>> {
    let rows = client.query(
        "SELECT closed_pnl, coin, crossed, dir, hash, order_id, price, side, start_position, size, timestamp, fee
         FROM user_fills WHERE user_address = $1",
        &[&user.to_string()],
    ).await?;

    let mut fills = Vec::new();
    for row in rows {
        fills.push(CustomUserFills {
            closed_pnl: row.get("closed_pnl"),
            coin: row.get("coin"),
            crossed: row.get("crossed"),
            dir: row.get("dir"),
            hash: row.get("hash"),
            order_id: row.get("order_id"),
            price: row.get("price"),
            side: row.get("side"),
            start_position: row.get("start_position"),
            size: row.get("size"),
            timestamp: row.get("timestamp"),
            fee: row.get("fee"),
        });
    }
    Ok(fills)
}

pub async fn load_fills_from_db_with_time_filter(
    client: &Client,
    user: H160,
    start_time: i64,
    end_time: i64,
) -> Result<Vec<CustomUserFills>> {
    let rows = client.query(
        "SELECT closed_pnl, coin, crossed, dir, hash, order_id, price, side, start_position, size, timestamp, fee
         FROM user_fills 
         WHERE user_address = $1 AND timestamp BETWEEN $2 AND $3",
        &[&user.to_string(), &start_time, &end_time],
    ).await?;

    let mut fills = Vec::new();
    for row in rows {
        fills.push(CustomUserFills {
            closed_pnl: row.get("closed_pnl"),
            coin: row.get("coin"),
            crossed: row.get("crossed"),
            dir: row.get("dir"),
            hash: row.get("hash"),
            order_id: row.get("order_id"),
            price: row.get("price"),
            side: row.get("side"),
            start_position: row.get("start_position"),
            size: row.get("size"),
            timestamp: row.get("timestamp"),
            fee: row.get("fee"),
        });
    }
    Ok(fills)
}
