use serde::{Deserialize, Serialize};

mod api;
mod error;
mod shared;

// Re-export common types
pub use api::*;
pub use error::*;
pub use shared::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirdeyeResponse<T> {
    pub success: bool,
    pub data: T,
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
pub struct MarketDepth {
    pub bids: Vec<OrderBookEntry>,
    pub asks: Vec<OrderBookEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookEntry {
    pub price: f64,
    pub size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    pub timestamp: i64,
    pub price: f64,
    pub volume: f64,
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

// Time intervals supported by the API
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimeInterval {
    #[serde(rename = "1m")]
    OneMinute,
    #[serde(rename = "5m")]
    FiveMinutes,
    #[serde(rename = "15m")]
    FifteenMinutes,
    #[serde(rename = "1h")]
    OneHour,
    #[serde(rename = "4h")]
    FourHours,
    #[serde(rename = "1d")]
    OneDay,
    #[serde(rename = "1w")]
    OneWeek,
}

impl std::fmt::Display for TimeInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeInterval::OneMinute => write!(f, "1m"),
            TimeInterval::FiveMinutes => write!(f, "5m"),
            TimeInterval::FifteenMinutes => write!(f, "15m"),
            TimeInterval::OneHour => write!(f, "1h"),
            TimeInterval::FourHours => write!(f, "4h"),
            TimeInterval::OneDay => write!(f, "1d"),
            TimeInterval::OneWeek => write!(f, "1w"),
        }
    }
} 