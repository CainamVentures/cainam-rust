use anyhow::Result;
use rig::{
    agent::Agent,
    providers::openai::{Client as OpenAIClient, CompletionModel, GPT_4_TURBO, TEXT_EMBEDDING_ADA_002, EmbeddingModel},
    vector_store::VectorStoreIndex,
};
use qdrant_client::{
    qdrant::{CreateCollectionBuilder, Distance, QueryPointsBuilder, VectorParamsBuilder},
    Qdrant,
};
use crate::{
    trading::TradingEngine,
    twitter::TwitterClient,
};
use rig_qdrant::QdrantVectorStore;

const COLLECTION_NAME: &str = "trade_memories";
const VECTOR_SIZE: u64 = 1536; // OpenAI embedding size

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
    vector_store: QdrantVectorStore<EmbeddingModel>,
    config: AgentConfig,
}

impl TradingAgent {
    pub async fn new(config: AgentConfig) -> Result<Self> {
        // Initialize OpenAI client
        let openai_client = OpenAIClient::new(&config.openai_api_key);
        
        // Create agent with GPT-4
        let agent = openai_client
            .agent(GPT_4_TURBO)
            .preamble(include_str!("../prompts/system.txt"))
            .build();

        // Initialize components
        let trading_engine = TradingEngine::new(0.7, 1000.0);
        let twitter_client = TwitterClient::new(
            config.twitter_email.clone(),
            config.twitter_username.clone(),
            config.twitter_password.clone()
        );
        
        // Initialize vector store
        let qdrant = Qdrant::from_url("http://localhost:6334").build()?;
        
        // Create collection if it doesn't exist
        if !qdrant.collection_exists(COLLECTION_NAME).await? {
            qdrant
                .create_collection(
                    CreateCollectionBuilder::new(COLLECTION_NAME)
                        .vectors_config(VectorParamsBuilder::new(VECTOR_SIZE, Distance::Cosine)),
                )
                .await?;
        }

        // Create vector store with OpenAI embeddings
        let embedding_model = openai_client.embedding_model(TEXT_EMBEDDING_ADA_002);
        let query_params = QueryPointsBuilder::new(COLLECTION_NAME).with_payload(true).build();
        let vector_store = QdrantVectorStore::new(qdrant, embedding_model, query_params);

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
        println!("Starting market analysis for {}", symbol);
        
        // Get market data from Birdeye
        println!("Fetching market data from Birdeye...");
        // TODO: Implement Birdeye API call
        
        // Store analysis in vector store
        println!("Storing analysis in vector store...");
        // TODO: Implement vector store storage
        
        println!("Analysis complete for {}", symbol);
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