use super::order::{LimitOrderParams, MarketOrderParams};
use crate::hyperliquid::model::{
    CustomCandle, CustomL2Book, CustomOpenOrders, CustomOrderStatus, CustomTrade, CustomUserFills,
    CustomUserTokenBalance, TokenDetails,
};
use anyhow::{anyhow, Context, Result};
use ethers::signers::LocalWallet;
use ethers::types::H160;
use hyperliquid_rust_sdk::{
    BaseUrl, ClientCancelRequest, ClientCancelRequestCloid, ClientLimit, ClientOrder,
    ClientOrderRequest, ExchangeClient, ExchangeDataStatus, ExchangeResponseStatus,
    FundingHistoryResponse, InfoClient, UserFundingResponse, UserStateResponse,
};
use log::{debug, error};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AssetInfo {
    pub internal_name: String,
    pub index: usize,
    pub sz_decimals: u8,
}

pub struct HttpClient {
    info: InfoClient,
    exchange: ExchangeClient,
    token_info: HashMap<String, AssetInfo>,
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

        let mut token_info = HashMap::new();
        token_info.extend(Self::build_spot_asset_map(&info).await?);
        token_info.extend(Self::build_perp_asset_map(&info).await?);

        Ok(Self {
            info,
            exchange,
            token_info,
        })
    }

    async fn build_spot_asset_map(info: &InfoClient) -> Result<HashMap<String, AssetInfo>> {
        let spot_meta = info
            .spot_meta()
            .await
            .context("Failed to fetch Spot meta")?;
        let mut asset_map = HashMap::new();

        for meta in spot_meta.universe.iter() {
            if meta.tokens.len() == 2 {
                let token_0_index = meta.tokens[0];
                let token_1_index = meta.tokens[1];

                if let (Some(token_0), Some(token_1)) = (
                    spot_meta.tokens.iter().find(|t| t.index == token_0_index),
                    spot_meta.tokens.iter().find(|t| t.index == token_1_index),
                ) {
                    let key = format!("{}/{}", token_0.name, token_1.name);
                    asset_map.insert(
                        key,
                        AssetInfo {
                            internal_name: meta.name.clone(),
                            index: meta.index + 10000,
                            sz_decimals: token_0.sz_decimals,
                        },
                    );
                }
            }
        }

        Ok(asset_map)
    }

    async fn build_perp_asset_map(info: &InfoClient) -> Result<HashMap<String, AssetInfo>> {
        let perp_meta = info.meta().await.context("Failed to fetch Perp meta")?;
        let mut asset_map = HashMap::new();

        for (index, meta) in perp_meta.universe.iter().enumerate() {
            asset_map.insert(
                meta.name.clone(),
                AssetInfo {
                    internal_name: meta.name.clone(),
                    index,
                    sz_decimals: meta.sz_decimals as u8,
                },
            );
        }

        Ok(asset_map)
    }

    pub fn get_asset_info(&self, symbol: &str) -> Option<&AssetInfo> {
        self.token_info.get(symbol)
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
                Ok(oid)
            }
            ExchangeResponseStatus::Err(err) => Err(anyhow!("Exchange returned an error: {}", err)),
        }
    }

    pub async fn market_order(&self, params: MarketOrderParams) -> Result<u64> {
        let (adjusted_price, sz_decimals) = self
            .calculate_slippage_price(&params.asset, params.is_buy, 0.01)
            .await
            .unwrap();

        let order = ClientOrderRequest {
            asset: params.asset,
            is_buy: params.is_buy,
            reduce_only: false,
            limit_px: adjusted_price,
            sz: round_to_decimals(params.size, sz_decimals),
            cloid: params.cloid,
            order_type: ClientOrder::Limit(ClientLimit {
                tif: "Ioc".to_string(),
            }),
        };

        let response_status = self
            .exchange
            .order(order, None)
            .await
            .context("Failed to place market order")?;

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
                Ok(oid)
            }
            ExchangeResponseStatus::Err(err) => Err(anyhow!("Exchange returned an error: {}", err)),
        }
    }

    async fn calculate_slippage_price(
        &self,
        asset: &str,
        is_buy: bool,
        slippage: f64,
    ) -> Result<(f64, u32)> {
        let asset_info = self
            .get_asset_info(asset)
            .context(format!("sz_decimals not found for asset {}", asset))?;

        let sz_decimals = asset_info.sz_decimals;
        let max_decimals: u32 = if asset_info.index < 10000 { 6 } else { 8 };

        let price_decimals = max_decimals.saturating_sub(sz_decimals as u32);
        let all_mids = self
            .fetch_all_mids()
            .await
            .context("Failed to fetch all mids")?;

        let current_price = all_mids
            .get(asset_info.internal_name.as_str())
            .context("Failed to fetch current price")?;

        let slippage_factor = if is_buy {
            1.0 + slippage
        } else {
            1.0 - slippage
        };

        let adjusted_price = current_price * slippage_factor;
        let adjusted_price = round_to_significant_and_decimal(adjusted_price, 5, price_decimals);

        Ok((adjusted_price, sz_decimals.into()))
    }

    pub async fn close_position(&self) {
        todo!("close_position")
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

    pub async fn cancel_by_cloid(&self, asset: String, cloid: Uuid) -> Result<String> {
        let request = ClientCancelRequestCloid { asset, cloid };
        let response_status = self
            .exchange
            .cancel_by_cloid(request, None)
            .await
            .context("Failed to cancel order by cloid")?;

        match response_status {
            ExchangeResponseStatus::Ok(exchange_response) => {
                if let Some(data) = exchange_response.data {
                    let success = data
                        .statuses
                        .iter()
                        .any(|status| matches!(status, ExchangeDataStatus::Success));

                    if success {
                        let success_msg = "Order cancelled successfully".to_string();
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

    // Perp positinos
    pub async fn fetch_user_state(&self, address: H160) -> Result<UserStateResponse> {
        let response = self
            .info
            .user_state(address)
            .await
            .context("Failed to fetch user state")?;
        Ok(response)
    }

    // Spot positions
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

    pub async fn query_order_status(&self, address: H160, oid: u64) -> Result<CustomOrderStatus> {
        let response = self
            .info
            .query_order_by_oid(address, oid)
            .await
            .context("Failed to query order status")?;

        let order_status: CustomOrderStatus = response.into();
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

    pub async fn fetch_l2_book(&self, coin: &str) -> Result<CustomL2Book> {
        let response = self
            .info
            .l2_snapshot(coin.to_string())
            .await
            .context("Failed to fetch l2 book")?;

        let l2_book: CustomL2Book = response.into();

        Ok(l2_book)
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

    pub async fn fetch_token_details(&self, token_id: String) -> Result<TokenDetails> {
        let request = serde_json::json!({"type": "tokenDetails", "tokenId": token_id});
        let data = serde_json::to_string(&request).context("Failed to serialize request")?;
        let response = self
            .info
            .http_client
            .post("/info", data)
            .await
            .context("Failed to fetch token details")?;
        debug!("Token details: {:#?}", response);
        serde_json::from_str(&response).context("Failed to deserialize response")
    }
}

fn round_to_decimals(value: f64, decimals: u32) -> f64 {
    let factor = 10f64.powi(decimals as i32);
    (value * factor).round() / factor
}

fn round_to_significant_and_decimal(value: f64, sig_figs: u32, max_decimals: u32) -> f64 {
    let abs_value = value.abs();
    let magnitude = abs_value.log10().floor() as i32;
    let scale = 10f64.powi(sig_figs as i32 - magnitude - 1);
    let rounded = (abs_value * scale).round() / scale;
    round_to_decimals(rounded.copysign(value), max_decimals)
}
