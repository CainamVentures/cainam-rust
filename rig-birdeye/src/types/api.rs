use serde::{Deserialize, Serialize};
use super::TimeInterval;

// Token Search Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSearchParams {
    pub keyword: String,
    #[serde(rename = "sort_by")]
    pub sort_by: Option<TokenSortBy>,
    #[serde(rename = "sort_type")]
    pub sort_type: Option<SortType>,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenSortBy {
    Fdv,
    Marketcap,
    Liquidity,
    Price,
    #[serde(rename = "price_change_24h_percent")]
    PriceChange24h,
    #[serde(rename = "volume_24h_usd")]
    Volume24h,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortType {
    Asc,
    Desc,
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