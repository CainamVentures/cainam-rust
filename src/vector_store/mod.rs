use anyhow::Result;
use qdrant_client::{
    client::QdrantClient,
    qdrant::{
        CreateCollection, Distance, PointStruct, SearchPoints, VectorParams, VectorsConfig,
        vectors_config::Config, Value as QdrantValue,
    },
};
use serde::{Deserialize, Serialize};
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
        let client = QdrantClient::from_url("http://localhost:6334").build()?;

        // Create collection if it doesn't exist
        if !client.collection_exists(COLLECTION_NAME).await? {
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
        let json_value = serde_json::to_value(&memory)?;
        let json_map = json_value.as_object().unwrap();
        let mut payload = HashMap::new();

        for (k, v) in json_map {
            match v {
                serde_json::Value::String(s) => payload.insert(k.clone(), QdrantValue::from(s.as_str())),
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        payload.insert(k.clone(), QdrantValue::from(i))
                    } else if let Some(f) = n.as_f64() {
                        payload.insert(k.clone(), QdrantValue::from(f))
                    } else {
                        None
                    }
                }
                _ => None,
            };
        }

        let point = PointStruct {
            id: Some(memory.id.into()),
            vectors: Some(embedding.into()),
            payload,
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
                limit,
                with_payload: Some(true.into()),
                ..Default::default()
            })
            .await?;

        let memories = search_result
            .result
            .into_iter()
            .filter_map(|point| {
                let mut json_map = serde_json::Map::new();
                
                for (k, v) in point.payload {
                    let json_value = match v {
                        v if v.to_string().starts_with('"') => {
                            let s = v.to_string().trim_matches('"').to_string();
                            serde_json::Value::String(s)
                        }
                        v => {
                            if let Ok(i) = v.to_string().parse::<i64>() {
                                serde_json::Value::Number(i.into())
                            } else if let Ok(f) = v.to_string().parse::<f64>() {
                                serde_json::Value::Number(serde_json::Number::from_f64(f).unwrap())
                            } else {
                                continue;
                            }
                        }
                    };
                    json_map.insert(k, json_value);
                }

                serde_json::from_value(serde_json::Value::Object(json_map)).ok()
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