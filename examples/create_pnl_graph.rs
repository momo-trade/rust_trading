use chrono::{FixedOffset, NaiveDate, TimeZone};
use ethers::types::H160;
use rust_trading::hyperliquid::db::load_fills_from_db;
use rust_trading::hyperliquid::db::load_fills_from_db_with_time_filter;
use rust_trading::hyperliquid::portfolio::PortfolioManager;
use std::error::Error;
use std::str::FromStr;
use tokio_postgres::NoTls;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let user = H160::from_str("your wallet pub key").unwrap();
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres password=password dbname=db_name",
        NoTls,
    )
    .await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let jst = FixedOffset::east_opt(9 * 3600).unwrap(); // JST (+09:00)
    let date = NaiveDate::from_ymd_opt(2024, 12, 27).unwrap();
    let start_time = jst
        .from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
        .unwrap()
        .timestamp_millis();
    let end_time = jst
        .from_local_datetime(&date.and_hms_opt(23, 59, 59).unwrap())
        .unwrap()
        .timestamp_millis();

    let fills_with_time_filter =
        load_fills_from_db_with_time_filter(&client, user, start_time, end_time)
            .await
            .unwrap();
    println!(
        "Loaded {} fills from the database",
        fills_with_time_filter.len()
    );

    let fills = load_fills_from_db(&client, user).await.unwrap();
    println!("Loaded {} fills from the database", fills.len());

    let portfolio = PortfolioManager::new();

    portfolio.create_pnl_chart(&fills, "pnl_chart.png")?;
    Ok(())
}
