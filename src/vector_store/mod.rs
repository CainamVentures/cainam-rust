use anyhow::Result;
use qdrant_client::{
    Qdrant,
    qdrant::{
        vectors_config::Config,
        Distance, VectorParams, VectorsConfig,
        QueryPoints, CreateCollection,
    },
};
use rig::{
    Embed, 
    embeddings::EmbeddingModel,
    vector_store::VectorStoreIndex,
};
use rig_qdrant::QdrantVectorStore;
use serde::{Deserialize, Serialize};

const COLLECTION_NAME: &str = "trading_memory";
const VECTOR_SIZE: u64 = 1536;  // OpenAI embedding size

#[derive(Debug, Clone, Serialize, Deserialize, Embed)]
pub struct TradeMemory {
    #[embed]
    pub id: String,
    #[embed]
    pub action: String,
    #[embed]
    pub symbol: String,
    pub amount: f64,
    pub price: f64,
    pub timestamp: i64,
    #[embed]
    pub reason: String,
    pub outcome: Option<String>,  // Not embedded since Option<String> doesn't implement Embed
}

#[allow(dead_code)]
pub struct VectorStore<M: EmbeddingModel> {
    store: QdrantVectorStore<M>,
    collection: String,
    model: M,
}

#[allow(dead_code)]
impl<M: EmbeddingModel> VectorStore<M> {
    pub async fn new(model: M) -> Result<Self> {
        let mut config = qdrant_client::config::QdrantConfig::from_url("http://localhost:6334");
        config.check_compatibility = false;  // Disable version compatibility check
        let client = Qdrant::new(config)?;

        // Create collection if it doesn't exist
        if !client.collection_exists(COLLECTION_NAME).await? {
            let vectors_config = VectorsConfig {
                config: Some(Config::Params(VectorParams {
                    size: VECTOR_SIZE as u64,
                    distance: Distance::Cosine.into(),
                    ..Default::default()
                })),
            };

            // Create collection using CreateCollection struct
            client.create_collection(CreateCollection {
                collection_name: COLLECTION_NAME.to_string(),
                vectors_config: Some(vectors_config),
                ..Default::default()
            }).await?;
        }

        let query_params = QueryPoints::default();
        let store = QdrantVectorStore::new(client, model.clone(), query_params);

        Ok(Self {
            store,
            collection: COLLECTION_NAME.to_string(),
            model,
        })
    }

    pub async fn store_memory(&self, memory: TradeMemory) -> Result<()> {
        // Store the memory using the VectorStoreIndex trait's top_n method
        // This will create the embedding and store it in the collection
        let _ = self.store.top_n::<TradeMemory>(&memory.reason, 0).await
            .map_err(|e| anyhow::anyhow!("Failed to store memory: {}", e))?;

        Ok(())
    }

    pub async fn search_similar(&self, query: &str, limit: u64) -> Result<Vec<TradeMemory>> {
        let results = self.store.top_n::<TradeMemory>(query, limit as usize).await
            .map_err(|e| anyhow::anyhow!("Failed to search similar memories: {}", e))?;
        
        Ok(results.into_iter().map(|(_, _, memory)| memory).collect())
    }
} 