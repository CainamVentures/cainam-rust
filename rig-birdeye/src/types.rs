use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInfo {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub price_usd: f64,
    pub volume_24h: f64,
    pub market_cap: f64,
}

#[derive(Debug)]
pub struct TokenOverview {
    pub price: f64,
    pub volume_24h: f64,
    pub price_change_24h: f64,
    pub liquidity: f64,
    pub holders: u64,
}

#[derive(Debug)]
pub struct LiquidityAnalysis {
    pub total_bid_liquidity: f64,
    pub total_ask_liquidity: f64,
    pub bid_ask_ratio: f64,
    pub depth_quality: f64,
}

#[derive(Debug)]
pub struct MarketImpact {
    pub price_impact: f64,
    pub executed_price: f64,
    pub size_usd: f64,
    pub size_tokens: f64,
}

#[derive(Debug)]
pub struct PricePoint {
    pub timestamp: i64,
    pub price: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum TimeInterval {
    OneHour,
}

impl ToString for TimeInterval {
    fn to_string(&self) -> String {
        match self {
            TimeInterval::OneHour => "1h".to_string(),
        }
    }
} 