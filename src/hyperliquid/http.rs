use super::order::LimitOrderParams;
use crate::hyperliquid::model::{CustomOpenOrders, CustomOprderStatus, CustomUserTokenBalance};
use ethers::signers::LocalWallet;
use ethers::types::H160;
use hyperliquid_rust_sdk::{
    BaseUrl, ClientCancelRequest, ClientLimit, ClientOrder, ClientOrderRequest, ExchangeClient,
    ExchangeDataStatus, ExchangeResponseStatus, InfoClient,
};
use log::{error, info};

pub struct HttpClient {
    info: InfoClient,
    exchange: ExchangeClient,
}

impl HttpClient {
    pub async fn new(is_mainnet: bool, wallet: LocalWallet) -> Result<Self, String> {
        let base_url = if is_mainnet {
            BaseUrl::Mainnet
        } else {
            BaseUrl::Testnet
        };

        let info = InfoClient::new(None, Some(base_url)).await.map_err(|err| {
            error!("{}", err);
            format!("Failed to initialize InfoClient: {}", err)
        })?;

        let exchange = ExchangeClient::new(None, wallet, Some(base_url), None, None)
            .await
            .map_err(|err| {
                error!("{}", err);
                format!("Failed to initialize ExchangeClient: {}", err)
            })?;

        Ok(Self { info, exchange })
    }

    pub async fn limit_order(&self, params: LimitOrderParams) -> Result<u64, String> {
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

        let response_status = match self.exchange.order(order, None).await {
            Ok(status) => status,
            Err(err) => {
                let error_msg = format!("Failed to place limit order: {}", err);
                error!("{}", error_msg);
                return Err(error_msg);
            }
        };

        // レスポンス処理
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
                    .ok_or_else(|| {
                        "No valid statuses or unexpected status in exchange response.".to_string()
                    })?;

                info!("Order placed successfully with oid: {}", oid);
                Ok(oid)
            }
            ExchangeResponseStatus::Err(err) => Err(format!("Exchange returned an error: {}", err)),
        }
    }

    pub async fn market_order(&self) {
        todo!("market_order")
    }

    pub async fn cancel_order(&self, asset: String, oid: u64) -> Result<String, String> {
        let request = ClientCancelRequest { asset, oid };
        let response_status = self.exchange.cancel(request, None).await.map_err(|err| {
            let error_msg = format!("Failed to cancel order: {}", err);
            error!("{}", error_msg);
            error_msg
        })?;

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
                let error_msg = "Unexpected response format: No success status found.".to_string();
                error!("{}", error_msg);
                Err(error_msg)
            }
            ExchangeResponseStatus::Err(err) => {
                let error_msg = format!("Exchange returned an error: {}", err);
                error!("{}", error_msg);
                Err(error_msg)
            }
        }
    }

    pub async fn fetch_open_orders(&self, address: H160) -> Result<Vec<CustomOpenOrders>, String> {
        match self.info.open_orders(address).await {
            Ok(response) => {
                let open_orders: Vec<CustomOpenOrders> =
                    response.into_iter().map(CustomOpenOrders::from).collect();
                Ok(open_orders)
            }
            Err(err) => Err(format!("Failed to fetch open orders: {}", err)),
        }
    }

    pub async fn fetch_user_state(&self) {
        todo!("fetch_user_state")
    }

    pub async fn fetch_token_balances(
        &self,
        address: H160,
    ) -> Result<Vec<CustomUserTokenBalance>, String> {
        match self.info.user_token_balances(address).await {
            Ok(response) => {
                let token_balance: Vec<CustomUserTokenBalance> = response
                    .balances
                    .into_iter()
                    .map(CustomUserTokenBalance::from)
                    .collect();
                Ok(token_balance)
            }
            Err(err) => Err(format!("Failed to fetch token balances: {}", err)),
        }
    }

    pub async fn query_order_status(
        &self,
        address: H160,
        oid: u64,
    ) -> Result<CustomOprderStatus, String> {
        match self.info.query_order_by_oid(address, oid).await {
            Ok(response) => {
                let order_status: CustomOprderStatus = response.into();
                Ok(order_status)
            }
            Err(err) => Err(format!("Failed to query order status: {}", err)),
        }
    }
}
