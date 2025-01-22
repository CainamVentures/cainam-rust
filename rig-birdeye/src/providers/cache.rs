use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct CachedClient<T> {
    inner: T,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    ttl: Duration,
}

struct CacheEntry {
    value: String,
    expires_at: Instant,
}

impl<T> CachedClient<T> {
    pub fn new(inner: T, ttl: Duration) -> Self {
        Self {
            inner,
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    pub async fn get_token_overview<F, Fut>(
        &self,
        key: &str,
        fetch: F,
    ) -> Result<crate::types::api::TokenOverview, crate::types::error::BirdeyeError>
    where
        F: FnOnce(&T, &str) -> Fut,
        Fut: std::future::Future<Output = Result<crate::types::api::TokenOverview, crate::types::error::BirdeyeError>>,
    {
        let cache_key = format!("token_overview:{}", key);
        
        // Try to get from cache first
        if let Some(cached) = self.get_cached(&cache_key).await {
            return serde_json::from_str(&cached)
                .map_err(crate::types::error::BirdeyeError::SerializationError);
        }

        // If not in cache, fetch fresh data
        let result = fetch(&self.inner, key).await?;
        
        // Cache the result
        let serialized = serde_json::to_string(&result)
            .map_err(crate::types::error::BirdeyeError::SerializationError)?;
        self.set_cached(&cache_key, serialized).await;

        Ok(result)
    }

    async fn get_cached(&self, key: &str) -> Option<String> {
        let cache = self.cache.read().await;
        if let Some(entry) = cache.get(key) {
            if entry.expires_at > Instant::now() {
                return Some(entry.value.clone());
            }
        }
        None
    }

    async fn set_cached(&self, key: &str, value: String) {
        let mut cache = self.cache.write().await;
        cache.insert(
            key.to_string(),
            CacheEntry {
                value,
                expires_at: Instant::now() + self.ttl,
            },
        );
    }
}
