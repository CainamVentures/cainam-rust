use anyhow::Result;
use qdrant_client::{
    prelude::*,
    qdrant::{
        vectors_config::Config,
        VectorParams,
        VectorsConfig,
        CreateCollection,
        PointStruct,
        Value,
        PointId,
    },
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

const COLLECTION_NAME: &str = "trading_memory";
const VECTOR_SIZE: u64 = 1536; // OpenAI embedding size

#[derive(Debug, Serialize, Deserialize)]
pub struct TradeMemory {
    pub id: String,
    pub action: String,
    pub symbol: String,
    pub amount: f64,
    pub price: f64,
    pub reason: String,
    pub outcome: String,
    pub timestamp: i64,
}

pub struct VectorStore {
    client: QdrantClient,
}

impl VectorStore {
    pub async fn new() -> Result<Self> {
        let client = QdrantClient::new(Some(QdrantClientConfig::from_url("http://localhost:6334")))?;

        // Create collection if it doesn't exist
        let collection_info = client.get_collection(COLLECTION_NAME).await;
        if collection_info.is_err() {
            let vectors_config = VectorsConfig {
                config: Some(Config::Params(VectorParams {
                    size: VECTOR_SIZE,
                    distance: Distance::Cosine.into(),
                    ..Default::default()
                })),
            };

            client
                .create_collection(&CreateCollection {
                    collection_name: COLLECTION_NAME.to_string(),
                    vectors_config: Some(vectors_config),
                    ..Default::default()
                })
                .await?;
        }

        Ok(Self { client })
    }

    pub async fn store_memory(&self, memory: TradeMemory, embedding: Vec<f32>) -> Result<()> {
        let mut payload: HashMap<String, Value> = HashMap::new();
        payload.insert("action".to_string(), memory.action.into());
        payload.insert("symbol".to_string(), memory.symbol.into());
        payload.insert("amount".to_string(), memory.amount.into());
        payload.insert("price".to_string(), memory.price.into());
        payload.insert("reason".to_string(), memory.reason.into());
        payload.insert("outcome".to_string(), memory.outcome.into());
        payload.insert("timestamp".to_string(), memory.timestamp.into());

        let point = PointStruct {
            id: Some(PointId::from(memory.id)),
            vectors: Some(embedding.into()),
            payload: payload,
        };

        self.client
            .upsert_points(COLLECTION_NAME, None, vec![point], None)
            .await?;

        Ok(())
    }

    pub async fn search_similar(&self, query_embedding: Vec<f32>, limit: u64) -> Result<Vec<TradeMemory>> {
        let search_result = self.client
            .search_points(&SearchPoints {
                collection_name: COLLECTION_NAME.to_string(),
                vector: query_embedding,
                limit: limit,
                with_payload: Some(true.into()),
                ..Default::default()
            })
            .await?;

        let memories = search_result
            .result
            .into_iter()
            .filter_map(|point| {
                let payload = point.payload;
                
                Some(TradeMemory {
                    id: point.id.to_string(),
                    action: payload.get("action")?.try_into().ok()?,
                    symbol: payload.get("symbol")?.try_into().ok()?,
                    amount: payload.get("amount")?.try_into().ok()?,
                    price: payload.get("price")?.try_into().ok()?,
                    reason: payload.get("reason")?.try_into().ok()?,
                    outcome: payload.get("outcome")?.try_into().ok()?,
                    timestamp: payload.get("timestamp")?.try_into().ok()?,
                })
            })
            .collect();

        Ok(memories)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_vector_store() -> Result<()> {
        let store = VectorStore::new().await?;

        // Test storing memory
        let memory = TradeMemory {
            id: "test1".to_string(),
            action: "BUY".to_string(),
            symbol: "SOL".to_string(),
            amount: 100.0,
            price: 50.0,
            reason: "Test trade".to_string(),
            outcome: "SUCCESS".to_string(),
            timestamp: Utc::now().timestamp(),
        };

        // Create mock embedding
        let embedding = vec![0.1; VECTOR_SIZE as usize];
        store.store_memory(memory, embedding.clone()).await?;

        // Test searching similar memories
        let results = store.search_similar(embedding, 1).await?;
        assert!(!results.is_empty());
        
        let result = &results[0];
        assert_eq!(result.symbol, "SOL");
        assert_eq!(result.action, "BUY");

        Ok(())
    }
} 