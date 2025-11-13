use crate::utils::{LazyJiraError, Result};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Rate limiter for API requests
/// Implements token bucket algorithm
pub struct RateLimiter {
    tokens: Arc<Mutex<usize>>,
    max_tokens: usize,
    refill_interval: Duration,
    tokens_per_refill: usize,
    last_refill: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    /// 
    /// # Arguments
    /// * `max_tokens` - Maximum number of tokens (requests) allowed
    /// * `refill_interval` - How often tokens are refilled
    /// * `tokens_per_refill` - How many tokens to add per refill
    pub fn new(max_tokens: usize, refill_interval: Duration, tokens_per_refill: usize) -> Self {
        Self {
            tokens: Arc::new(Mutex::new(max_tokens)),
            max_tokens,
            refill_interval,
            tokens_per_refill,
            last_refill: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Create a rate limiter for Jira Cloud (100 requests per minute)
    pub fn jira_cloud() -> Self {
        Self::new(
            100,
            Duration::from_secs(60),
            100, // Refill all tokens every minute
        )
    }

    /// Wait until a token is available
    pub async fn wait_for_token(&self) -> Result<()> {
        loop {
            let mut tokens = self.tokens.lock().await;
            let mut last_refill = self.last_refill.lock().await;

            // Refill tokens if enough time has passed
            let now = Instant::now();
            let elapsed = now.duration_since(*last_refill);

            if elapsed >= self.refill_interval {
                let refills = (elapsed.as_secs_f64() / self.refill_interval.as_secs_f64()) as usize;
                *tokens = (*tokens + refills * self.tokens_per_refill).min(self.max_tokens);
                *last_refill = now;
            }

            // If tokens available, consume one and return
            if *tokens > 0 {
                *tokens -= 1;
                return Ok(());
            }

            // No tokens available, wait for refill
            let wait_time = self.refill_interval - elapsed;
            drop(tokens);
            drop(last_refill);
            tokio::time::sleep(wait_time).await;
        }
    }

    /// Try to acquire a token without waiting (non-blocking)
    pub async fn try_acquire_token(&self) -> bool {
        let mut tokens = self.tokens.lock().await;
        let mut last_refill = self.last_refill.lock().await;

        // Refill tokens if enough time has passed
        let now = Instant::now();
        let elapsed = now.duration_since(*last_refill);

        if elapsed >= self.refill_interval {
            let refills = (elapsed.as_secs_f64() / self.refill_interval.as_secs_f64()) as usize;
            *tokens = (*tokens + refills * self.tokens_per_refill).min(self.max_tokens);
            *last_refill = now;
        }

        if *tokens > 0 {
            *tokens -= 1;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_acquires_token() {
        let limiter = RateLimiter::new(10, Duration::from_millis(100), 10);
        assert!(limiter.try_acquire_token().await);
    }

    #[tokio::test]
    async fn test_rate_limiter_exhausts_tokens() {
        let limiter = RateLimiter::new(2, Duration::from_secs(1), 2);
        
        assert!(limiter.try_acquire_token().await);
        assert!(limiter.try_acquire_token().await);
        assert!(!limiter.try_acquire_token().await); // Should be exhausted
    }

    #[tokio::test]
    async fn test_rate_limiter_refills_tokens() {
        let limiter = RateLimiter::new(2, Duration::from_millis(100), 2);
        
        // Exhaust tokens
        assert!(limiter.try_acquire_token().await);
        assert!(limiter.try_acquire_token().await);
        assert!(!limiter.try_acquire_token().await);
        
        // Wait for refill
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Should have tokens again
        assert!(limiter.try_acquire_token().await);
    }

    #[tokio::test]
    async fn test_rate_limiter_wait_for_token() {
        let limiter = RateLimiter::new(1, Duration::from_millis(50), 1);
        
        // Acquire the only token
        assert!(limiter.try_acquire_token().await);
        
        // Wait for token should block until refill
        let start = Instant::now();
        limiter.wait_for_token().await.unwrap();
        let elapsed = start.elapsed();
        
        // Should have waited approximately the refill interval
        assert!(elapsed >= Duration::from_millis(50));
        assert!(elapsed < Duration::from_millis(100)); // But not too long
    }
}
