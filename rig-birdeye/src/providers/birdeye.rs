use std::time::Duration;
use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;
use crate::types::*;

const API_BASE_URL: &str = "https://public-api.birdeye.so/v1";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_RETRIES: u32 = 3;
const RETRY_DELAY: Duration = Duration::from_millis(1000);

#[derive(Debug, Clone)]
pub struct BirdeyeProvider {
    client: Client,
    api_key: String,
}

impl BirdeyeProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(DEFAULT_TIMEOUT)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_key: api_key.into(),
        }
    }

    async fn request<T>(&self, endpoint: &str, params: Option<&[(&str, &str)]>) -> BirdeyeResult<T>
    where
        T: DeserializeOwned,
    {
        let mut url = format!("{}{}", API_BASE_URL, endpoint);
        
        let mut retries = 0;
        loop {
            let mut req = self.client
                .get(&url)
                .header("X-API-KEY", &self.api_key);

            if let Some(query_params) = params {
                req = req.query(query_params);
            }

            let response = req.send().await?;
            let status = response.status();

            match status {
                StatusCode::OK => {
                    let data = response.json::<T>().await?;
                    return Ok(data);
                }
                status if status.is_client_error() => {
                    let error_msg = response.text().await.unwrap_or_default();
                    return Err(BirdeyeError::from_status_and_message(
                        status.as_u16(),
                        error_msg,
                    ));
                }
                _ if retries < MAX_RETRIES => {
                    retries += 1;
                    tokio::time::sleep(RETRY_DELAY).await;
                    continue;
                }
                _ => {
                    let error_msg = response.text().await.unwrap_or_default();
                    return Err(BirdeyeError::from_status_and_message(
                        status.as_u16(),
                        error_msg,
                    ));
                }
            }
        }
    }

    pub async fn search_tokens(&self, params: TokenSearchParams) -> BirdeyeResult<Vec<TokenMarketData>> {
        let mut query_params = vec![
            ("keyword", params.keyword.as_str()),
        ];

        if let Some(sort_by) = params.sort_by {
            query_params.push(("sort_by", sort_by.to_string().as_str()));
        }

        if let Some(sort_type) = params.sort_type {
            query_params.push(("sort_type", sort_type.to_string().as_str()));
        }

        if let Some(offset) = params.offset {
            query_params.push(("offset", offset.to_string().as_str()));
        }

        if let Some(limit) = params.limit {
            query_params.push(("limit", limit.to_string().as_str()));
        }

        self.request("/token/search", Some(&query_params)).await
    }

    pub async fn get_token_overview(&self, address: &str) -> BirdeyeResult<TokenOverview> {
        let query_params = [("address", address)];
        self.request("/token/overview", Some(&query_params)).await
    }

    pub async fn get_token_security(&self, address: &str) -> BirdeyeResult<TokenSecurity> {
        let query_params = [("address", address)];
        self.request("/token/security", Some(&query_params)).await
    }

    pub async fn get_price_history(
        &self,
        params: PriceHistoryParams,
    ) -> BirdeyeResult<Vec<PriceHistoryPoint>> {
        let mut query_params = vec![
            ("address", params.address.as_str()),
            ("interval", params.interval.to_string().as_str()),
        ];

        if let Some(time_from) = params.time_from {
            query_params.push(("time_from", time_from.to_string().as_str()));
        }

        if let Some(time_to) = params.time_to {
            query_params.push(("time_to", time_to.to_string().as_str()));
        }

        self.request("/token/price-history", Some(&query_params)).await
    }

    pub async fn get_wallet_portfolio(&self, wallet: &str) -> BirdeyeResult<WalletPortfolio> {
        let query_params = [("wallet", wallet)];
        self.request("/wallet/portfolio", Some(&query_params)).await
    }
} 