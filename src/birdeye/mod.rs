use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json;

const BIRDEYE_API_BASE: &str = "https://public-api.birdeye.so";

// Common token addresses
const TOKEN_ADDRESSES: &[(&str, &str)] = &[
    ("SOL", "So11111111111111111111111111111111111111112"),
    ("USDC", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
    ("BONK", "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"),
];

#[derive(Debug, Clone)]
pub struct BirdeyeClient {
    client: Client,
    api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInfo {
    pub price: f64,
    pub volume24h: f64,
    #[serde(rename = "priceChange24h")]
    pub price_change_24h: f64,
    pub liquidity: f64,
    pub trade24h: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenMarketResponse {
    success: bool,
    data: TokenMarketData,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenMarketData {
    items: Vec<TokenMarketItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenMarketItem {
    price: Option<f64>,
    volume24h: Option<f64>,
    #[serde(rename = "priceChange24h")]
    price_change_24h: Option<f64>,
    liquidity: Option<f64>,
    trade24h: Option<i64>,
}

impl BirdeyeClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    fn get_token_address(symbol: &str) -> Result<&'static str> {
        let symbol = symbol.to_uppercase();
        TOKEN_ADDRESSES
            .iter()
            .find(|(s, _)| *s == symbol)
            .map(|(_, addr)| *addr)
            .ok_or_else(|| anyhow!("Unknown token symbol: {}", symbol))
    }

    pub async fn get_token_info(&self, symbol: &str) -> Result<TokenInfo> {
        let token_address = Self::get_token_address(symbol)?;
        let url = format!("{}/public/price?address={}", BIRDEYE_API_BASE, token_address);
        
        println!("Requesting: {}", url);
        
        let response = self.client
            .get(&url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await?;
            
        let status = response.status();
        let text = response.text().await?;
        
        println!("Response status: {}", status);
        println!("Response body: {}", text);
        
        if !status.is_success() {
            return Err(anyhow!("Birdeye API request failed with status {}: {}", status, text));
        }
        
        let response: TokenMarketResponse = serde_json::from_str(&text)
            .map_err(|e| anyhow!("Failed to parse response: {}\nResponse: {}", e, text))?;

        if !response.success {
            return Err(anyhow!("Birdeye API request failed"));
        }

        let market = response.data.items.first()
            .ok_or_else(|| anyhow!("No market data found for token"))?;

        Ok(TokenInfo {
            price: market.price.unwrap_or_default(),
            volume24h: market.volume24h.unwrap_or_default(),
            price_change_24h: market.price_change_24h.unwrap_or_default(),
            liquidity: market.liquidity.unwrap_or_default(),
            trade24h: market.trade24h.unwrap_or_default(),
        })
    }
} 