use serde::{Deserialize, Serialize};
use rig_core::plugin::{Action, ActionError};
use crate::{
    providers::birdeye::BirdeyeProvider,
    types::{
        api::{TokenSearchParams, TokenInfo, WalletPortfolio},
        error::BirdeyeError,
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenSearchAction {
    pub keyword: String,
    pub sort_by: Option<String>,
    pub sort_type: Option<String>,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

impl Action for TokenSearchAction {
    const NAME: &'static str = "birdeye.search_tokens";
    type Output = Vec<TokenInfo>;
    type Error = BirdeyeError;
    type Context = BirdeyeProvider;

    async fn execute(self, context: &Self::Context) -> Result<Self::Output, Self::Error> {
        let params = TokenSearchParams {
            keyword: self.keyword,
            sort_by: self.sort_by.map(|s| s.parse().unwrap()),
            sort_type: self.sort_type.map(|s| s.parse().unwrap()),
            offset: self.offset,
            limit: self.limit,
        };

        context.search_tokens(params).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletSearchAction {
    pub address: String,
}

impl Action for WalletSearchAction {
    const NAME: &'static str = "birdeye.search_wallet";
    type Output = WalletPortfolio;
    type Error = BirdeyeError;
    type Context = BirdeyeProvider;

    async fn execute(self, context: &Self::Context) -> Result<Self::Output, Self::Error> {
        context.search_wallet(&self.address).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_search_action() -> Result<(), BirdeyeError> {
        let provider = BirdeyeProvider::new("test_key");
        
        let action = TokenSearchAction {
            keyword: "SOL".to_string(),
            sort_by: None,
            sort_type: None,
            offset: None,
            limit: Some(10),
        };

        let result = action.execute(&provider).await?;
        assert!(!result.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_wallet_search_action() -> Result<(), BirdeyeError> {
        let provider = BirdeyeProvider::new("test_key");
        
        let action = WalletSearchAction {
            address: "DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK".to_string(),
        };

        let result = action.execute(&provider).await?;
        assert!(result.total_usd >= 0.0);
        Ok(())
    }
}
