use anyhow::Result;
use qdrant_client::{
    client::{QdrantClient, QdrantClientConfig},
    qdrant::{
        vectors_config::Config,
        CreateCollection, Distance, PointStruct, SearchPoints, VectorParams, VectorsConfig,
        Value as QdrantValue, PointId,
    },
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

const COLLECTION_NAME: &str = "trade_memories";
const VECTOR_SIZE: u64 = 1536; // OpenAI embedding size

#[derive(Debug, Serialize, Deserialize)]
pub struct TradeMemory {
    pub timestamp: i64,
    pub symbol: String,
    pub action: String,
    pub amount: f64,
    pub price: f64,
    pub success: bool,
    pub notes: String,
}

pub struct VectorStore {
    client: QdrantClient,
}

impl VectorStore {
    pub async fn new() -> Result<Self> {
        let config = QdrantClientConfig::from_url("http://localhost:6334");
        let client = QdrantClient::new(Some(config))?;
        
        if !client.collection_exists(COLLECTION_NAME).await? {
            let config = CreateCollection {
                collection_name: COLLECTION_NAME.to_string(),
                vectors_config: Some(VectorsConfig {
                    config: Some(Config::Params(VectorParams {
                        size: VECTOR_SIZE,
                        distance: Distance::Cosine.into(),
                        ..Default::default()
                    })),
                }),
                ..Default::default()
            };
            
            client.create_collection(&config).await?;
        }

        Ok(Self { client })
    }

    pub async fn store_memory(&self, memory: TradeMemory, embedding: Vec<f32>) -> Result<()> {
        let mut payload_map = HashMap::new();
        let json_value = serde_json::to_value(&memory)?;
        if let serde_json::Value::Object(map) = json_value {
            for (k, v) in map {
                match v {
                    serde_json::Value::String(s) => { payload_map.insert(k, QdrantValue::from(s)); }
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            payload_map.insert(k, QdrantValue::from(i));
                        } else if let Some(f) = n.as_f64() {
                            payload_map.insert(k, QdrantValue::from(f));
                        }
                    }
                    serde_json::Value::Bool(b) => { payload_map.insert(k, QdrantValue::from(b)); }
                    _ => {}
                }
            }
        }

        let point_id = PointId {
            point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(
                Uuid::new_v4().to_string()
            )),
        };

        let point = PointStruct {
            id: Some(point_id),
            payload: payload_map,
            vectors: Some(embedding.into()),
        };
        
        self.client
            .upsert_points(COLLECTION_NAME, None, vec![point], None)
            .await?;

        Ok(())
    }

    pub async fn search_similar(&self, query_embedding: Vec<f32>, limit: u64) -> Result<Vec<TradeMemory>> {
        let search_points = SearchPoints {
            collection_name: COLLECTION_NAME.to_string(),
            vector: query_embedding,
            limit,
            with_payload: Some(true.into()),
            ..Default::default()
        };

        let response = self.client.search_points(&search_points).await?;
        
        let memories = response
            .result
            .into_iter()
            .filter_map(|point| {
                let payload = point.payload;
                let mut json_map = serde_json::Map::new();
                for (k, v) in payload {
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
                            } else if let Ok(b) = v.to_string().parse::<bool>() {
                                serde_json::Value::Bool(b)
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
            timestamp: Utc::now().timestamp(),
            symbol: "SOL".to_string(),
            action: "BUY".to_string(),
            amount: 100.0,
            price: 50.0,
            success: true,
            notes: "Test trade".to_string(),
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