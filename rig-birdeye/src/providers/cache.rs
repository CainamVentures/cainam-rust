use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use serde::{de::DeserializeOwned, Serialize};

/// Cache entry with TTL
#[derive(Clone)]
struct CacheEntry<T> {
    data: T,
    expires_at: Instant,
}

/// Thread-safe cache implementation with TTL support
pub struct Cache<K, V> {
    entries: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    ttl: Duration,
}

impl<K, V> Cache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    /// Create a new cache with the specified TTL
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    /// Get a value from the cache if it exists and hasn't expired
    pub async fn get(&self, key: &K) -> Option<V> {
        let entries = self.entries.read().await;
        if let Some(entry) = entries.get(key) {
            if Instant::now() < entry.expires_at {
                return Some(entry.data.clone());
            }
        }
        None
    }

    /// Insert a value into the cache
    pub async fn set(&self, key: K, value: V) {
        let mut entries = self.entries.write().await;
        entries.insert(
            key,
            CacheEntry {
                data: value,
                expires_at: Instant::now() + self.ttl,
            },
        );
    }

    /// Remove expired entries from the cache
    pub async fn cleanup(&self) {
        let mut entries = self.entries.write().await;
        let now = Instant::now();
        entries.retain(|_, entry| entry.expires_at > now);
    }
}

/// A cached API client that wraps another client implementation
pub struct CachedClient<T> {
    inner: T,
    token_cache: Cache<String, TokenInfo>,
    overview_cache: Cache<String, TokenOverview>,
}

use crate::types::{
    api::{TokenInfo, TokenOverview},
    error::BirdeyeError,
};

impl<T> CachedClient<T>
where
    T: Clone,
{
    /// Create a new cached client with the specified TTL for cache entries
    pub fn new(inner: T, cache_ttl: Duration) -> Self {
        Self {
            inner,
            token_cache: Cache::new(cache_ttl),
            overview_cache: Cache::new(cache_ttl),
        }
    }

    /// Get token info from cache or fetch from API
    pub async fn get_token_info(
        &self,
        address: &str,
        fetch: impl Fn(&T, &str) -> impl std::future::Future<Output = Result<TokenInfo, BirdeyeError>>,
    ) -> Result<TokenInfo, BirdeyeError> {
        if let Some(cached) = self.token_cache.get(&address.to_string()).await {
            return Ok(cached);
        }

        let info = fetch(&self.inner, address).await?;
        self.token_cache.set(address.to_string(), info.clone()).await;
        Ok(info)
    }

    /// Get token overview from cache or fetch from API
    pub async fn get_token_overview(
        &self,
        address: &str,
        fetch: impl Fn(&T, &str) -> impl std::future::Future<Output = Result<TokenOverview, BirdeyeError>>,
    ) -> Result<TokenOverview, BirdeyeError> {
        if let Some(cached) = self.overview_cache.get(&address.to_string()).await {
            return Ok(cached);
        }

        let overview = fetch(&self.inner, address).await?;
        self.overview_cache
            .set(address.to_string(), overview.clone())
            .await;
        Ok(overview)
    }

    /// Start background cache cleanup task
    pub fn start_cleanup(cache: Cache<String, TokenInfo>, interval: Duration) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            loop {
                interval.tick().await;
                cache.cleanup().await;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_cache_basic() {
        let cache: Cache<String, i32> = Cache::new(Duration::from_secs(1));
        
        cache.set("key1".to_string(), 42).await;
        assert_eq!(cache.get(&"key1".to_string()).await, Some(42));
        assert_eq!(cache.get(&"key2".to_string()).await, None);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache: Cache<String, i32> = Cache::new(Duration::from_millis(100));
        
        cache.set("key1".to_string(), 42).await;
        assert_eq!(cache.get(&"key1".to_string()).await, Some(42));
        
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert_eq!(cache.get(&"key1".to_string()).await, None);
    }

    #[tokio::test]
    async fn test_cache_cleanup() {
        let cache: Cache<String, i32> = Cache::new(Duration::from_millis(100));
        
        cache.set("key1".to_string(), 42).await;
        cache.set("key2".to_string(), 43).await;
        
        tokio::time::sleep(Duration::from_millis(200)).await;
        cache.cleanup().await;
        
        assert_eq!(cache.entries.read().await.len(), 0);
    }
}
