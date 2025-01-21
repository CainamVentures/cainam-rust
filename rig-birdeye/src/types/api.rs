use serde::{Deserialize, Serialize};
use super::TimeInterval;
use std::fmt;
use crate::providers::pagination::PaginationParams;

// Token Search Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSearchParams {
    pub keyword: String,
    #[serde(rename = "sort_by")]
    pub sort_by: Option<TokenSortBy>,
    #[serde(rename = "sort_type")]
    pub sort_type: Option<SortType>,
    #[serde(flatten)]
    pub pagination: Option<PaginationParams>,
}

impl TokenSearchParams {
    pub fn new(keyword: String) -> Self {
        Self {
            keyword,
            sort_by: None,
            sort_type: None,
            pagination: None,
        }
    }

    pub fn with_sort(mut self, sort_by: TokenSortBy, sort_type: SortType) -> Self {
        self.sort_by = Some(sort_by);
        self.sort_type = Some(sort_type);
        self
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = Some(pagination);
        self
    }

    pub fn with_limit(mut self, limit: u32) -> Self {
        let pagination = self.pagination.get_or_insert_with(PaginationParams::default);
        pagination.limit = Some(limit);
        self
    }

    pub fn offset(&self) -> Option<u32> {
        self.pagination.as_ref().and_then(|p| p.offset)
    }

    pub fn limit(&self) -> Option<u32> {
        self.pagination.as_ref().and_then(|p| p.limit)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenSortBy {
    #[serde(rename = "price")]
    Price,
    #[serde(rename = "volume")]
    Volume,
    #[serde(rename = "liquidity")]
    Liquidity,
    #[serde(rename = "price_change")]
    PriceChange,
}

impl fmt::Display for TokenSortBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenSortBy::Price => write!(f, "price"),
            TokenSortBy::Volume => write!(f, "volume"),
            TokenSortBy::Liquidity => write!(f, "liquidity"),
            TokenSortBy::PriceChange => write!(f, "price_change"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortType {
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    Descending,
}

impl fmt::Display for SortType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortType::Ascending => write!(f, "asc"),
            SortType::Descending => write!(f, "desc"),
        }
    }
}

// Token Market Data Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMarketData {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub price: f64,
    pub price_change_24h: f64,
    pub volume_24h: f64,
    pub market_cap: Option<f64>,
    pub fully_diluted_valuation: Option<f64>,
    pub circulating_supply: Option<f64>,
    pub total_supply: Option<f64>,
}

// Wallet Portfolio Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletPortfolioParams {
    pub wallet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletPortfolio {
    pub wallet: String,
    pub total_usd: f64,
    pub items: Vec<WalletToken>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletToken {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub balance: String,
    pub ui_amount: f64,
    pub chain_id: String,
    pub logo_uri: Option<String>,
    pub price_usd: f64,
    pub value_usd: f64,
}

// Price History Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceHistoryParams {
    pub address: String,
    pub interval: TimeInterval,
    pub time_from: Option<i64>,
    pub time_to: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceHistoryPoint {
    pub timestamp: i64,
    pub price: f64,
    pub volume: f64,
}

// Token Security Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSecurity {
    pub address: String,
    pub total_supply: f64,
    pub mintable: bool,
    pub proxied: bool,
    pub owner_address: String,
    pub creator_address: String,
    pub security_checks: SecurityChecks,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityChecks {
    pub honeypot: bool,
    pub trading_cooldown: bool,
    pub transfer_pausable: bool,
    pub is_blacklisted: bool,
    pub is_whitelisted: bool,
    pub is_proxy: bool,
    pub is_mintable: bool,
    pub can_take_back_ownership: bool,
    pub hidden_owner: bool,
    pub anti_whale_modifiable: bool,
    pub is_anti_whale: bool,
    pub trading_pausable: bool,
    pub can_be_blacklisted: bool,
    pub is_true_token: bool,
    pub is_airdrop_scam: bool,
    pub slippage_modifiable: bool,
    pub is_honeypot: bool,
    pub transfer_pausable_time: bool,
    pub is_wrapped: bool,
}

// Token Overview Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenOverview {
    pub address: String,
    pub decimals: u8,
    pub symbol: String,
    pub name: String,
    pub extensions: Option<TokenExtensions>,
    pub logo_uri: Option<String>,
    pub liquidity: f64,
    pub price: f64,
    pub volume_24h: f64,
    pub price_change_24h: f64,
    pub holders: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenExtensions {
    pub coingecko_id: Option<String>,
    pub website: Option<String>,
    pub telegram: Option<String>,
    pub twitter: Option<String>,
    pub discord: Option<String>,
    pub medium: Option<String>,
    pub description: Option<String>,
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
