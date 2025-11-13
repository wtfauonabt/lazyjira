use crate::utils::{LazyJiraError, Result};
use std::time::Duration;

/// Retry configuration
pub struct RetryConfig {
    pub max_retries: usize,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
        }
    }
}

/// Execute a function with retry logic and exponential backoff
pub async fn retry_with_backoff<F, Fut, T>(
    config: &RetryConfig,
    mut f: F,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut delay = config.initial_delay;
    let mut last_error = None;

    for attempt in 0..=config.max_retries {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);

                // Don't retry on authentication errors
                if matches!(
                    last_error.as_ref().unwrap(),
                    LazyJiraError::Authentication(_)
                ) {
                    return Err(last_error.unwrap());
                }

                // Don't retry on validation errors
                if matches!(
                    last_error.as_ref().unwrap(),
                    LazyJiraError::Validation(_)
                ) {
                    return Err(last_error.unwrap());
                }

                // Don't retry on 4xx errors (except 429 which is handled separately)
                if let LazyJiraError::Api(msg) = last_error.as_ref().unwrap() {
                    if msg.contains("400") || msg.contains("401") || msg.contains("403") 
                        || msg.contains("404") || msg.contains("422") {
                        return Err(last_error.unwrap());
                    }
                }

                // If this was the last attempt, return the error
                if attempt == config.max_retries {
                    break;
                }

                // Wait before retrying
                tokio::time::sleep(delay).await;

                // Calculate next delay with exponential backoff
                delay = Duration::from_secs_f64(
                    (delay.as_secs_f64() * config.backoff_multiplier)
                        .min(config.max_delay.as_secs_f64()),
                );
            }
        }
    }

    Err(last_error.unwrap())
}

/// Check if an error is retryable
pub fn is_retryable_error(error: &LazyJiraError) -> bool {
    match error {
        LazyJiraError::Network(_) => true,
        LazyJiraError::Api(msg) => {
            // Retry on 429 (Too Many Requests) and 5xx errors
            msg.contains("429") || msg.contains("500") || msg.contains("502") || msg.contains("503")
        }
        LazyJiraError::Authentication(_) => false,
        LazyJiraError::Validation(_) => false,
        LazyJiraError::Config(_) => false,
        LazyJiraError::Parse(_) => false,
        LazyJiraError::Internal(_) => false,
        LazyJiraError::Io(_) => true, // IO errors might be transient
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::LazyJiraError;

    #[tokio::test]
    async fn test_retry_succeeds_on_first_attempt() {
        let config = RetryConfig::default();
        let mut attempts = 0;

        let result = retry_with_backoff(&config, || {
            attempts += 1;
            async move { Ok(42) }
        })
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts, 1);
    }

    #[tokio::test]
    async fn test_retry_succeeds_after_retries() {
        let config = RetryConfig {
            max_retries: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_secs(1),
            backoff_multiplier: 2.0,
        };
        let attempts = std::sync::Arc::new(std::sync::Mutex::new(0));

        let attempts_clone = attempts.clone();
        let result: Result<i32> = retry_with_backoff(&config, move || {
            let attempts = attempts_clone.clone();
            async move {
                let mut count = attempts.lock().unwrap();
                *count += 1;
                let current = *count;
                drop(count);
                
                if current < 3 {
                    // Create a network error
                    Err(LazyJiraError::Api("Network error".to_string()))
                } else {
                    Ok(42)
                }
            }
        })
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(*attempts.lock().unwrap(), 3);
    }

    #[tokio::test]
    async fn test_retry_fails_after_max_retries() {
        let config = RetryConfig {
            max_retries: 2,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_secs(1),
            backoff_multiplier: 2.0,
        };
        let attempts = std::sync::Arc::new(std::sync::Mutex::new(0));

        let attempts_clone = attempts.clone();
        let result: Result<i32> = retry_with_backoff(&config, move || {
            let attempts = attempts_clone.clone();
            async move {
                let mut count = attempts.lock().unwrap();
                *count += 1;
                drop(count);
                Err(LazyJiraError::Api("Network error".to_string()))
            }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(*attempts.lock().unwrap(), 3); // Initial + 2 retries
    }

    #[tokio::test]
    async fn test_retry_does_not_retry_auth_errors() {
        let config = RetryConfig::default();
        let mut attempts = 0;

        let result: Result<i32> = retry_with_backoff(&config, || {
            attempts += 1;
            async move {
                Err(LazyJiraError::Authentication("Invalid credentials".to_string()))
            }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(attempts, 1); // Should not retry
    }

    #[test]
    fn test_is_retryable_error() {
        // Test API errors (which are retryable)
        assert!(is_retryable_error(&LazyJiraError::Api("429 Too Many Requests".to_string())));
        assert!(is_retryable_error(&LazyJiraError::Api("500 Internal Server Error".to_string())));
        assert!(is_retryable_error(&LazyJiraError::Api("502 Bad Gateway".to_string())));
        
        // Test non-retryable errors
        assert!(!is_retryable_error(&LazyJiraError::Authentication("Invalid".to_string())));
        assert!(!is_retryable_error(&LazyJiraError::Validation("Invalid".to_string())));
    }
}
