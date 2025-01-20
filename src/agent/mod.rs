use anyhow::Result;
use rig::{
    agent::Agent,
    providers::openai::{Client as OpenAIClient, CompletionModel},
    embeddings::EmbeddingModel,
};
use crate::{
    trading::TradingEngine,
    twitter::TwitterClient,
    vector_store::VectorStore,
};

#[allow(dead_code)]
pub struct TradingAgent<M: EmbeddingModel> {
    agent: Agent<CompletionModel>,
    trading_engine: TradingEngine,
    twitter_client: TwitterClient,
    vector_store: VectorStore<M>,
}

#[allow(dead_code)]
impl<M: EmbeddingModel> TradingAgent<M> {
    pub async fn new(
        openai_key: &str,
        twitter_email: String,
        twitter_username: String,
        twitter_password: String,
        embedding_model: M,
    ) -> Result<Self> {
        // Initialize OpenAI client
        let openai_client = OpenAIClient::new(openai_key);
        
        // Create agent with GPT-4o-mini
        let agent = openai_client
            .agent("gpt-4o-mini")
            .preamble(include_str!("../prompts/system.txt"))
            .build();

        // Initialize components
        let trading_engine = TradingEngine::new(0.7, 1000.0);
        let twitter_client = TwitterClient::new(twitter_email, twitter_username, twitter_password);
        let vector_store = VectorStore::new(embedding_model).await?;

        Ok(Self {
            agent,
            trading_engine,
            twitter_client,
            vector_store,
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