use anyhow::{Result, anyhow};
use rig::{
    agent::Agent,
    providers::openai::{Client as OpenAIClient, CompletionModel, GPT_4_TURBO},
};
use crate::{
    trading::TradingEngine,
    twitter::TwitterClient,
    vector_store::VectorStore,
};

#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub openai_api_key: String,
    pub birdeye_api_key: String,
    pub twitter_email: String,
    pub twitter_username: String,
    pub twitter_password: String,
}

pub struct TradingAgent {
    agent: Agent<CompletionModel>,
    trading_engine: TradingEngine,
    twitter_client: TwitterClient,
    vector_store: VectorStore,
    config: AgentConfig,
}

impl TradingAgent {
    pub async fn new(config: AgentConfig) -> Result<Self> {
        // Initialize OpenAI client
        let openai_client = OpenAIClient::new(&config.openai_api_key);
        
        // Create agent with GPT-4
        let agent = Agent::builder()
            .with_model(openai_client.completion_model(GPT_4_TURBO))
            .with_system_message(include_str!("../prompts/system.txt"))
            .build()?;

        // Initialize components
        let trading_engine = TradingEngine::new(0.7, 1000.0);
        let twitter_client = TwitterClient::new(
            config.twitter_email.clone(),
            config.twitter_username.clone(),
            config.twitter_password.clone()
        );
        let vector_store = VectorStore::new().await?;

        Ok(Self {
            agent,
            trading_engine,
            twitter_client,
            vector_store,
            config,
        })
    }

    pub async fn analyze_market(&self, symbol: &str) -> Result<()> {
        // TODO: Implement market analysis using Birdeye API
        tracing::info!("Analyzing market for {}", symbol);
        Ok(())
    }

    pub async fn execute_trade(&self, symbol: &str, action: &str, amount: f64) -> Result<bool> {
        let decision = crate::trading::TradeDecision {
            action: action.to_string(),
            symbol: symbol.to_string(),
            amount,
            reason: "Market analysis indicates favorable conditions".to_string(),
            confidence: 0.8,
        };

        self.trading_engine.execute_trade(&decision).await
    }

    pub async fn post_trade_update(&self, symbol: &str, action: &str, amount: f64) -> Result<()> {
        let tweet = format!(
            "🤖 Trade Alert!\n{} {} {}\nReason: Market analysis indicates favorable conditions",
            action, amount, symbol
        );

        self.twitter_client.post_tweet(&tweet).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_trading_agent_creation() -> Result<()> {
        let config = AgentConfig {
            openai_api_key: "test_key".to_string(),
            birdeye_api_key: "test_key".to_string(),
            twitter_email: "test@example.com".to_string(),
            twitter_username: "test_user".to_string(),
            twitter_password: "test_pass".to_string(),
        };

        let agent = TradingAgent::new(config).await?;
        assert!(agent.config.openai_api_key == "test_key");
        Ok(())
    }

    #[tokio::test]
    async fn test_market_analysis() -> Result<()> {
        let config = AgentConfig {
            openai_api_key: "test_key".to_string(),
            birdeye_api_key: "test_key".to_string(),
            twitter_email: "test@example.com".to_string(),
            twitter_username: "test_user".to_string(),
            twitter_password: "test_pass".to_string(),
        };

        let agent = TradingAgent::new(config).await?;
        agent.analyze_market("SOL").await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_trade_execution() -> Result<()> {
        let config = AgentConfig {
            openai_api_key: "test_key".to_string(),
            birdeye_api_key: "test_key".to_string(),
            twitter_email: "test@example.com".to_string(),
            twitter_username: "test_user".to_string(),
            twitter_password: "test_pass".to_string(),
        };

        let agent = TradingAgent::new(config).await?;
        let result = agent.execute_trade("SOL", "BUY", 100.0).await?;
        assert!(result);
        Ok(())
    }
} 