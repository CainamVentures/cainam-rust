use std::fmt;
use serde::{Deserialize, Serialize};
use rig_core::plugin::{Plugin, PluginRegistrar};

pub mod providers;
pub mod types;
pub mod actions;

pub use types::*;
pub use actions::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenSortBy {
    Price,
    Volume,
    MarketCap,
}

impl fmt::Display for TokenSortBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenSortBy::Price => write!(f, "price"),
            TokenSortBy::Volume => write!(f, "volume"),
            TokenSortBy::MarketCap => write!(f, "market_cap"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortType {
    Asc,
    Desc,
}

impl fmt::Display for SortType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortType::Asc => write!(f, "asc"),
            SortType::Desc => write!(f, "desc"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenSearchParams {
    pub keyword: String,
    pub sort_by: Option<String>,
    pub sort_type: Option<String>,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

pub struct BirdeyeProvider {
    api_key: String,
}

impl BirdeyeProvider {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }

    pub async fn search_tokens(&self, params: TokenSearchParams) -> anyhow::Result<Vec<TokenInfo>> {
        // Mock implementation for now
        Ok(vec![TokenInfo {
            address: "mock_address".to_string(),
            symbol: "MOCK".to_string(),
            name: "Mock Token".to_string(),
            decimals: 9,
            price_usd: 1.0,
            volume_24h: 1000000.0,
            market_cap: 10000000.0,
        }])
    }

    pub async fn get_token_overview(&self, _address: &str) -> anyhow::Result<TokenOverview> {
        Ok(TokenOverview {
            price: 1.0,
            volume_24h: 1000000.0,
            price_change_24h: 5.0,
            liquidity: 500000.0,
            holders: 1000,
        })
    }

    pub async fn analyze_liquidity(&self, _address: &str) -> anyhow::Result<LiquidityAnalysis> {
        Ok(LiquidityAnalysis {
            total_bid_liquidity: 250000.0,
            total_ask_liquidity: 250000.0,
            bid_ask_ratio: 1.0,
            depth_quality: 0.8,
        })
    }

    pub async fn get_market_impact(&self, _address: &str, size_usd: f64) -> anyhow::Result<MarketImpact> {
        Ok(MarketImpact {
            price_impact: 0.01,
            executed_price: 1.01,
            size_usd,
            size_tokens: size_usd,
        })
    }

    pub async fn get_price_history(&self, _address: &str, _interval: &str) -> anyhow::Result<Vec<PricePoint>> {
        Ok(vec![PricePoint {
            timestamp: chrono::Utc::now().timestamp(),
            price: 1.0,
            volume: 100000.0,
        }])
    }
}  

#[derive(Default)]
pub struct BirdeyePlugin;

impl Plugin for BirdeyePlugin {
    fn name(&self) -> &'static str {
        "birdeye"
    }

    fn description(&self) -> &'static str {
        "Birdeye plugin for token and wallet analytics on Solana"
    }

    fn register(&self, registrar: &mut dyn PluginRegistrar) {
        registrar.register_action::<TokenSearchAction>();
        registrar.register_action::<WalletSearchAction>();
    }
}

// Export plugin creation function
#[no_mangle]
pub fn create_plugin() -> Box<dyn Plugin> {
    Box::new(BirdeyePlugin::default())
} 