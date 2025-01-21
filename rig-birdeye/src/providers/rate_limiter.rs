use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

/// A token bucket rate limiter implementation
pub struct RateLimiter {
    tokens: Arc<Mutex<f64>>,
    capacity: f64,
    refill_rate: f64,
    last_update: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    /// * `capacity` - Maximum number of tokens the bucket can hold
    /// * `refill_rate` - Number of tokens added per second
    pub fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            tokens: Arc::new(Mutex::new(capacity)),
            capacity,
            refill_rate,
            last_update: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Attempt to acquire a token
    /// Returns true if a token was acquired, false if no tokens are available
    pub async fn try_acquire(&self) -> bool {
        let mut tokens = self.tokens.lock().await;
        let mut last_update = self.last_update.lock().await;

        // Calculate tokens to add based on time elapsed
        let now = Instant::now();
        let elapsed = now.duration_since(*last_update).as_secs_f64();
        *tokens = (*tokens + (elapsed * self.refill_rate)).min(self.capacity);
        *last_update = now;

        if *tokens >= 1.0 {
            *tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Acquire a token, waiting if necessary
    pub async fn acquire(&self) {
        loop {
            if self.try_acquire().await {
                return;
            }
            // Wait for approximately the time it takes to refill one token
            tokio::time::sleep(Duration::from_secs_f64(1.0 / self.refill_rate)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(5.0, 1.0); // 5 tokens, 1 per second
        
        // Should be able to acquire 5 tokens immediately
        for _ in 0..5 {
            assert!(limiter.try_acquire().await);
        }
        
        // Should not be able to acquire more tokens
        assert!(!limiter.try_acquire().await);
    }

    #[tokio::test]
    async fn test_rate_limiter_refill() {
        let limiter = RateLimiter::new(2.0, 1.0); // 2 tokens, 1 per second
        
        // Use up both tokens
        assert!(limiter.try_acquire().await);
        assert!(limiter.try_acquire().await);
        assert!(!limiter.try_acquire().await);
        
        // Wait for a token to be refilled
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // Should be able to acquire one token
        assert!(limiter.try_acquire().await);
    }

    #[tokio::test]
    async fn test_rate_limiter_acquire() {
        let limiter = RateLimiter::new(1.0, 1.0); // 1 token, 1 per second
        
        // First acquire should be immediate
        let start = Instant::now();
        limiter.acquire().await;
        assert!(start.elapsed() < Duration::from_millis(50));
        
        // Second acquire should take about 1 second
        let start = Instant::now();
        limiter.acquire().await;
        assert!(start.elapsed() >= Duration::from_millis(900));
    }
}
