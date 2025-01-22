use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};

#[derive(Debug)]
struct RateLimiterState {
    tokens: f64,
    last_update: Instant,
    max_tokens: f64,
    refill_rate: f64,
}

#[derive(Debug, Clone)]
pub struct RateLimiter {
    state: Arc<Mutex<RateLimiterState>>,
}

impl RateLimiter {
    pub fn new(max_tokens: f64, refill_rate: f64) -> Self {
        let state = RateLimiterState {
            tokens: max_tokens,
            last_update: Instant::now(),
            max_tokens,
            refill_rate,
        };

        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }

    pub async fn acquire(&self) {
        let mut state = self.state.lock().await;
        let now = Instant::now();
        let elapsed = now.duration_since(state.last_update);
        
        // Add tokens based on elapsed time
        state.tokens = (state.tokens + elapsed.as_secs_f64() * state.refill_rate)
            .min(state.max_tokens);
        state.last_update = now;

        // If we don't have enough tokens, calculate wait time
        if state.tokens < 1.0 {
            let wait_time = Duration::from_secs_f64((1.0 - state.tokens) / state.refill_rate);
            let state_clone = self.state.clone();
            
            // Release the lock before sleeping
            drop(state);
            
            tokio::time::sleep(wait_time).await;
            
            // Reacquire the lock and update state
            let mut state = state_clone.lock().await;
            state.tokens = state.tokens + wait_time.as_secs_f64() * state.refill_rate;
            state.last_update = Instant::now();
        }

        state.tokens -= 1.0;
    }
}
