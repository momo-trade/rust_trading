use super::order::LimitOrderParams;
use crate::hyperliquid::model::{
    CustomCandle, CustomOpenOrders, CustomOprderStatus, CustomTrade, CustomUserFills,
    CustomUserTokenBalance,
};
use anyhow::{anyhow, Context, Result};
use ethers::signers::LocalWallet;
use ethers::types::H160;
use hyperliquid_rust_sdk::{
    BaseUrl, ClientCancelRequest, ClientLimit, ClientOrder, ClientOrderRequest, ExchangeClient,
    ExchangeDataStatus, ExchangeResponseStatus, FundingHistoryResponse, InfoClient,
    UserFundingResponse, UserStateResponse,
};
use log::{error, info};
use std::collections::HashMap;

pub struct HttpClient {
    info: InfoClient,
    exchange: ExchangeClient,
}

impl HttpClient {
    pub async fn new(is_mainnet: bool, wallet: LocalWallet) -> Result<Self> {
        let base_url = if is_mainnet {
            BaseUrl::Mainnet
        } else {
            BaseUrl::Testnet
        };

        let info = InfoClient::new(None, Some(base_url))
            .await
            .context("Failed to initialize InfoClient")?;

        let exchange = ExchangeClient::new(None, wallet, Some(base_url), None, None)
            .await
            .context("Failed to initialize ExchangeClient")?;

        Ok(Self { info, exchange })
    }

    pub async fn limit_order(&self, params: LimitOrderParams) -> Result<u64> {
        let reduce_only = params.reduce_only.unwrap_or(false);
        let time_in_force = params.time_in_force.unwrap_or("Gtc".to_string());

        let order = ClientOrderRequest {
            asset: params.asset,
            is_buy: params.is_buy,
            reduce_only,
            limit_px: params.price,
            sz: params.size,
            cloid: params.cloid,
            order_type: ClientOrder::Limit(ClientLimit { tif: time_in_force }),
        };

        let response_status = self
            .exchange
            .order(order, None)
            .await
            .context("Failed to place limit order")?;

        match response_status {
            ExchangeResponseStatus::Ok(exchange_response) => {
                let oid = exchange_response
                    .data
                    .and_then(|data| {
                        data.statuses.first().map(|status| match status {
                            ExchangeDataStatus::Filled(order) => Some(order.oid),
                            ExchangeDataStatus::Resting(order) => Some(order.oid),
                            _ => None,
                        })
                    })
                    .flatten()
                    .context("No valid statuses or unexpected status in exchange response.")?;
                info!("Order placed successfully with oid: {}", oid);
                Ok(oid)
            }
            ExchangeResponseStatus::Err(err) => Err(anyhow!("Exchange returned an error: {}", err)),
        }
    }

    pub async fn market_order(&self) {
        todo!("market_order")
    }

    pub async fn cancel_order(&self, asset: String, oid: u64) -> Result<String> {
        let request = ClientCancelRequest { asset, oid };
        let response_status = self
            .exchange
            .cancel(request, None)
            .await
            .context("Failed to cancel order")?;

        match response_status {
            ExchangeResponseStatus::Ok(exchange_response) => {
                if let Some(data) = exchange_response.data {
                    let success = data
                        .statuses
                        .iter()
                        .any(|status| matches!(status, ExchangeDataStatus::Success));

                    if success {
                        let success_msg = "Order cancelled successfully".to_string();
                        info!("{}", success_msg);
                        return Ok(success_msg);
                    }
                }
                Err(anyhow!(
                    "Unexpected response format: No success status found."
                ))
            }
            ExchangeResponseStatus::Err(err) => Err(anyhow!("Exchange returned an error: {}", err)),
        }
    }

    pub async fn fetch_open_orders(&self, address: H160) -> Result<Vec<CustomOpenOrders>> {
        let response = self
            .info
            .open_orders(address)
            .await
            .context("Failed to fetch open orders")?;

        let open_orders: Vec<CustomOpenOrders> =
            response.into_iter().map(CustomOpenOrders::from).collect();
        Ok(open_orders)
    }

    pub async fn fetch_user_state(&self, address: H160) -> Result<UserStateResponse> {
        let response = self
            .info
            .user_state(address)
            .await
            .context("Failed to fetch user state")?;
        Ok(response)
    }

    pub async fn fetch_token_balances(&self, address: H160) -> Result<Vec<CustomUserTokenBalance>> {
        let response = self
            .info
            .user_token_balances(address)
            .await
            .context("Failed to fetch token balances")?;

        let token_balance: Vec<CustomUserTokenBalance> = response
            .balances
            .into_iter()
            .map(CustomUserTokenBalance::from)
            .collect();
        Ok(token_balance)
    }

    pub async fn query_order_status(&self, address: H160, oid: u64) -> Result<CustomOprderStatus> {
        let response = self
            .info
            .query_order_by_oid(address, oid)
            .await
            .context("Failed to query order status")?;

        let order_status: CustomOprderStatus = response.into();
        Ok(order_status)
    }

    pub async fn fetch_all_mids(&self) -> Result<HashMap<String, f64>> {
        let response = self
            .info
            .all_mids()
            .await
            .context("Failed to fetch all mids")?;

        let parsed_map: HashMap<String, f64> = response
            .into_iter()
            .filter_map(|(key, value)| match value.parse::<f64>() {
                Ok(parsed_value) => Some((key, parsed_value)),
                Err(err) => {
                    error!("Failed to parse value for key {}: {}", key, err);
                    None
                }
            })
            .collect();

        Ok(parsed_map)
    }

    pub async fn fetch_user_fills(&self, address: H160) -> Result<Vec<CustomUserFills>> {
        let response = self
            .info
            .user_fills(address)
            .await
            .context("Failed to fetch user fills")?;

        let user_fills: Vec<CustomUserFills> =
            response.into_iter().map(CustomUserFills::from).collect();
        Ok(user_fills)
    }

    pub async fn fetch_funding_history(
        &self,
        coin: &str,
        start_time: u64,
        end_time: Option<u64>,
    ) -> Result<Vec<FundingHistoryResponse>> {
        let response = self
            .info
            .funding_history(coin.to_string(), start_time, end_time)
            .await
            .context("Failed to fetch funding history")?;

        Ok(response)
    }

    pub async fn fetch_user_funding_history(
        &self,
        address: H160,
        start_time: u64,
        end_time: Option<u64>,
    ) -> Result<Vec<UserFundingResponse>> {
        let response = self
            .info
            .user_funding_history(address, start_time, end_time)
            .await
            .context("Failed to fetch user funding history")?;

        Ok(response)
    }

    pub async fn fetch_trades(&self, coin: &str) -> Result<Vec<CustomTrade>> {
        let response = self
            .info
            .recent_trades(coin.to_string())
            .await
            .context("Failed to fetch trades")?;

        let trades: Vec<CustomTrade> = response.into_iter().map(CustomTrade::from).collect();
        Ok(trades)
        // info!("Trades: {:#?}", resposne);
    }

    pub async fn fetch_candles(
        &self,
        coin: &str,
        interval: &str,
        start_time: u64,
        end_time: u64,
    ) -> Result<Vec<CustomCandle>> {
        let resposne = self
            .info
            .candles_snapshot(coin.to_string(), interval.to_string(), start_time, end_time)
            .await
            .context("Failed to fetch candles")?;

        let candles: Vec<CustomCandle> = resposne.into_iter().map(CustomCandle::from).collect();
        Ok(candles)
    }
}
