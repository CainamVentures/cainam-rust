use std::time::Duration;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::types::{
    api::{TokenSearchParams, WalletPortfolio, TokenOverview, LiquidityAnalysis, MarketImpact, PricePoint},
    error::BirdeyeError,
    TimeInterval,
};
use super::{
    rate_limiter::RateLimiter,
    pagination::{PaginationParams, PaginatedResponse, PaginatedIterator},
    cache::CachedClient,
};
use crate::{
    providers::{
        cache::TokenCache,
        pagination::{PaginatedRequest},
    },
    types::{
        api::{TokenInfo},
    },
};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::de::DeserializeOwned;

// Default cache TTL of 1 minute for frequently accessed data
const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(60);
const DEFAULT_RATE_LIMIT: f64 = 10.0;
const DEFAULT_CACHE_DURATION: Duration = Duration::from_secs(60);

// Birdeye API allows 10 requests per second
const RATE_LIMIT_CAPACITY: f64 = 10.0;
const RATE_LIMIT_REFILL_RATE: f64 = 10.0; // tokens per second

const API_BASE_URL: &str = "https://public-api.birdeye.so";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_RETRIES: u32 = 3;
const RETRY_DELAY: Duration = Duration::from_millis(1000);

#[derive(Clone)]
pub struct BirdeyeProvider {
    client: Client,
    api_key: String,
    rate_limiter: RateLimiter,
    token_cache: Arc<Mutex<TokenCache>>,
}

pub struct CachedBirdeyeProvider {
    inner: CachedClient<BirdeyeProvider>,
}

impl CachedBirdeyeProvider {
    pub fn new(api_key: &str) -> Self {
        let provider = BirdeyeProvider::new(api_key.to_string());
        Self {
            inner: CachedClient::new(provider, DEFAULT_CACHE_TTL),
        }
    }

    pub fn with_cache_ttl(api_key: &str, cache_ttl: Duration) -> Self {
        let provider = BirdeyeProvider::new(api_key.to_string());
        Self {
            inner: CachedClient::new(provider, cache_ttl),
        }
    }

    pub async fn get_token_overview(&self, address: &str) -> Result<TokenOverview, BirdeyeError> {
        self.inner
            .get_token_overview(address, |provider, addr| async move {
                provider.get_token_overview(addr).await
            })
            .await
    }
}

impl BirdeyeProvider {
    pub fn new(api_key: String) -> Self {
        Self::new_with_config(
            api_key,
            DEFAULT_RATE_LIMIT,
            DEFAULT_CACHE_DURATION,
        )
    }

    pub fn new_with_config(
        api_key: String,
        rate_limit: f64,
        cache_duration: Duration,
    ) -> Self {
        let client = Client::builder()
            .timeout(DEFAULT_TIMEOUT)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_key,
            rate_limiter: RateLimiter::new(rate_limit, rate_limit),
            token_cache: Arc::new(Mutex::new(TokenCache::new(cache_duration))),
        }
    }

    /// Create a new BirdeyeProvider with custom rate limiting parameters
    pub fn with_rate_limit(api_key: &str, capacity: f64, refill_rate: f64) -> Self {
        let client = Client::builder()
            .timeout(DEFAULT_TIMEOUT)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_key: api_key.to_string(),
            rate_limiter: RateLimiter::new(capacity, refill_rate),
            token_cache: Arc::new(Mutex::new(TokenCache::new(DEFAULT_CACHE_TTL))),
        }
    }

    async fn request<T: DeserializeOwned>(&self, endpoint: &str, params: &[(&str, String)]) -> Result<T, BirdeyeError> {
        let url = format!("{}{}", API_BASE_URL, endpoint);
        
        let mut retries = 0;
        loop {
            // Wait for rate limit token
            self.rate_limiter.acquire().await;
            
            let response = self.client
                .get(&url)
                .header("X-API-KEY", &self.api_key)
                .query(params)
                .send()
                .await
                .map_err(BirdeyeError::RequestError)?;

            if !response.status().is_success() {
                if retries >= MAX_RETRIES {
                    return Err(BirdeyeError::InvalidResponse(format!(
                        "HTTP {} - {}",
                        response.status(),
                        response.text().await.unwrap_or_default()
                    )));
                }
                retries += 1;
                tokio::time::sleep(RETRY_DELAY).await;
                continue;
            }

            return response
                .json()
                .await
                .map_err(BirdeyeError::SerializationError);
        }
    }

    pub async fn search_tokens(&self, params: TokenSearchParams) -> Result<Vec<TokenInfo>, BirdeyeError> {
        let mut query_params = vec![
            ("query", params.query.clone()),
        ];

        if let Some(limit) = params.limit {
            query_params.push(("limit", limit.to_string()));
        }

        if let Some(offset) = params.offset {
            query_params.push(("offset", offset.to_string()));
        }

        self.request("/v1/token/search", &query_params).await
    }

    pub fn search_tokens_iter(
        &self,
        params: TokenSearchParams,
    ) -> PaginatedIterator<TokenInfo, impl PaginatedRequest<TokenInfo>> {
        let client = self.clone();
        let request = move |offset: u32, limit: u32| {
            let mut params = params.clone();
            params.offset = Some(offset);
            params.limit = Some(limit);
            Box::pin(async move { client.search_tokens(params).await })
        };

        PaginatedIterator::new(request, params.limit.unwrap_or(10))
    }

    pub async fn get_token_overview(&self, address: &str) -> Result<TokenOverview, BirdeyeError> {
        let query_params = vec![("address", address.to_string())];
        self.request("/v1/token/overview", &query_params).await
    }

    pub async fn analyze_liquidity(&self, address: &str) -> Result<LiquidityAnalysis, BirdeyeError> {
        let query_params = vec![("address", address.to_string())];
        self.request("/v1/token/liquidity", &query_params).await
    }

    pub async fn get_market_impact(&self, address: &str, size_usd: f64) -> Result<MarketImpact, BirdeyeError> {
        let query_params = vec![
            ("address", address.to_string()),
            ("size", size_usd.to_string()),
        ];
        self.request("/v1/token/impact", &query_params).await
    }

    pub async fn get_price_history(&self, address: &str, interval: TimeInterval) -> Result<Vec<PricePoint>, BirdeyeError> {
        let query_params = vec![
            ("address", address.to_string()),
            ("interval", interval.to_string()),
        ];
        self.request("/v1/token/price_history", &query_params).await
    }

    pub async fn get_wallet_portfolio(&self, wallet_address: &str) -> Result<WalletPortfolio, BirdeyeError> {
        let query_params = vec![("wallet", wallet_address.to_string())];
        self.request("/v1/wallet/tokens", &query_params).await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub price_usd: f64,
    pub volume_24h: f64,
    pub market_cap: f64,
}

impl Clone for BirdeyeProvider {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            api_key: self.api_key.clone(),
            rate_limiter: RateLimiter::new(RATE_LIMIT_CAPACITY, RATE_LIMIT_REFILL_RATE),
            token_cache: Arc::new(Mutex::new(TokenCache::new(DEFAULT_CACHE_TTL))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenOverview {
    pub price: f64,
    pub volume_24h: f64,
    pub price_change_24h: f64,
    pub liquidity: f64,
    pub holders: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityAnalysis {
    pub total_bid_liquidity: f64,
    pub total_ask_liquidity: f64,
    pub bid_ask_ratio: f64,
    pub depth_quality: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketImpact {
    pub price_impact: f64,
    pub executed_price: f64,
    pub size_usd: f64,
    pub size_tokens: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    pub timestamp: i64,
    pub price: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeInterval {
    FiveMinutes,
    FifteenMinutes,
    OneHour,
    FourHours,
    OneDay,
    OneWeek,
    OneMonth,
}

impl ToString for TimeInterval {
    fn to_string(&self) -> String {
        match self {
            TimeInterval::FiveMinutes => "5m".to_string(),
            TimeInterval::FifteenMinutes => "15m".to_string(),
            TimeInterval::OneHour => "1h".to_string(),
            TimeInterval::FourHours => "4h".to_string(),
            TimeInterval::OneDay => "1d".to_string(),
            TimeInterval::OneWeek => "1w".to_string(),
            TimeInterval::OneMonth => "1M".to_string(),
        }
    }
}

impl std::str::FromStr for TimeInterval {
    type Err = BirdeyeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "5m" => Ok(TimeInterval::FiveMinutes),
            "15m" => Ok(TimeInterval::FifteenMinutes),
            "1h" => Ok(TimeInterval::OneHour),
            "4h" => Ok(TimeInterval::FourHours),
            "1d" => Ok(TimeInterval::OneDay),
            "1w" => Ok(TimeInterval::OneWeek),
            "1m" => Ok(TimeInterval::OneMonth),
            _ => Err(BirdeyeError::InvalidParameters(format!(
                "Invalid time interval: {}. Valid intervals are: 5m, 15m, 1h, 4h, 1d, 1w, 1M",
                s
            ))),
        }
    }
}

#[cfg(test)]
mod time_interval_tests {
    use super::*;

    #[test]
    fn test_time_interval_to_string() {
        assert_eq!(TimeInterval::FiveMinutes.to_string(), "5m");
        assert_eq!(TimeInterval::FifteenMinutes.to_string(), "15m");
        assert_eq!(TimeInterval::OneHour.to_string(), "1h");
        assert_eq!(TimeInterval::FourHours.to_string(), "4h");
        assert_eq!(TimeInterval::OneDay.to_string(), "1d");
        assert_eq!(TimeInterval::OneWeek.to_string(), "1w");
        assert_eq!(TimeInterval::OneMonth.to_string(), "1M");
    }

    #[test]
    fn test_time_interval_from_str() {
        assert_eq!("5m".parse::<TimeInterval>().unwrap(), TimeInterval::FiveMinutes);
        assert_eq!("15m".parse::<TimeInterval>().unwrap(), TimeInterval::FifteenMinutes);
        assert_eq!("1h".parse::<TimeInterval>().unwrap(), TimeInterval::OneHour);
        assert_eq!("4h".parse::<TimeInterval>().unwrap(), TimeInterval::FourHours);
        assert_eq!("1d".parse::<TimeInterval>().unwrap(), TimeInterval::OneDay);
        assert_eq!("1w".parse::<TimeInterval>().unwrap(), TimeInterval::OneWeek);
        assert_eq!("1M".parse::<TimeInterval>().unwrap(), TimeInterval::OneMonth);
    }

    #[test]
    fn test_invalid_time_interval() {
        assert!(matches!(
            "invalid".parse::<TimeInterval>(),
            Err(BirdeyeError::InvalidParameters(_))
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_tokens() -> Result<(), BirdeyeError> {
        let provider = CachedBirdeyeProvider::new("test_key");
        let params = TokenSearchParams {
            keyword: "SOL".to_string(),
            sort_by: None,
            sort_type: None,
            offset: None,
            limit: Some(10),
        };

        let response = provider.search_tokens(params).await?;
        assert!(!response.is_empty());
        assert!(response.len() == 10);
        Ok(())
    }

    #[tokio::test]
    async fn test_search_tokens_iter() -> Result<(), BirdeyeError> {
        let provider = BirdeyeProvider::new("test_key");
        let params = TokenSearchParams {
            keyword: "SOL".to_string(),
            sort_by: None,
            sort_type: None,
            offset: None,
            limit: Some(10),
        };

        let mut total_tokens = 0;
        let mut iterator = provider.search_tokens_iter(params);
        
        while let Some(result) = iterator.next_page().await {
            let tokens = result?;
            total_tokens += tokens.len();
            if total_tokens >= 20 {
                break;
            }
        }

        assert!(total_tokens > 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_token_overview() -> Result<(), BirdeyeError> {
        let provider = CachedBirdeyeProvider::new("test_key");
        let overview = provider.get_token_overview("mock_address").await?;
        assert!(overview.price > 0.0);
        Ok(())
    }

    #[tokio::test]
    async fn test_analyze_liquidity() -> Result<(), BirdeyeError> {
        let provider = CachedBirdeyeProvider::new("test_key");
        let liquidity = provider.analyze_liquidity("mock_address").await?;
        assert!(liquidity.bid_ask_ratio > 0.0);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_market_impact() -> Result<(), BirdeyeError> {
        let provider = CachedBirdeyeProvider::new("test_key");
        let impact = provider.get_market_impact("mock_address", 1000.0).await?;
        assert!(impact.price_impact > 0.0);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_price_history() -> Result<(), BirdeyeError> {
        let provider = CachedBirdeyeProvider::new("test_key");
        let history = provider.get_price_history("mock_address", TimeInterval::OneHour).await?;
        assert!(!history.is_empty());
        Ok(())
    }
}
