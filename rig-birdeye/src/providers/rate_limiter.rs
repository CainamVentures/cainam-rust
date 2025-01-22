use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct RateLimiter {
    state: Arc<Mutex<RateLimiterState>>,
}

#[derive(Debug)]
struct RateLimiterState {
    tokens: f64,
    last_update: Instant,
    capacity: f64,
    refill_rate: f64,
}

impl RateLimiter {
    pub fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            state: Arc::new(Mutex::new(RateLimiterState {
                tokens: capacity,
                last_update: Instant::now(),
                capacity,
                refill_rate,
            })),
        }
    }

    pub async fn acquire(&self) {
        let mut state = self.state.lock().await;
        
        // Update token count based on time elapsed
        let now = Instant::now();
        let elapsed = now.duration_since(state.last_update);
        let new_tokens = elapsed.as_secs_f64() * state.refill_rate;
        state.tokens = (state.tokens + new_tokens).min(state.capacity);
        state.last_update = now;

        // Wait if we don't have enough tokens
        if state.tokens < 1.0 {
            let wait_time = Duration::from_secs_f64((1.0 - state.tokens) / state.refill_rate);
            drop(state); // Release lock while waiting
            tokio::time::sleep(wait_time).await;
            let mut state = self.state.lock().await;
            state.tokens = state.tokens + wait_time.as_secs_f64() * state.refill_rate;
        }

        state.tokens -= 1.0;
    }
}
