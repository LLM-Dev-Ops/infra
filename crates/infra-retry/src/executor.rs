//! Retry execution logic and traits.

use crate::policy::{RetryDecision, RetryPolicy};
use async_trait::async_trait;
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;

/// Trait for operations that can be retried.
///
/// Implementors define the operation to execute and how to handle errors.
#[async_trait]
pub trait Retryable {
    /// The success type returned by the operation.
    type Output;
    /// The error type that may be returned.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Executes the operation.
    ///
    /// This method is called for each retry attempt.
    async fn execute(&mut self) -> Result<Self::Output, Self::Error>;

    /// Determines if an error is retryable.
    ///
    /// By default, all errors are considered retryable.
    fn is_retryable(&self, _error: &Self::Error) -> bool {
        true
    }
}

/// Retries an async operation according to a retry policy.
///
/// # Arguments
///
/// * `operation` - A closure that returns a future producing the result.
/// * `policy` - The retry policy to use.
///
/// # Returns
///
/// The result of the operation if successful, or the last error encountered.
///
/// # Examples
///
/// ```no_run
/// use infra_retry::{retry_with_policy, ExponentialBackoff};
/// use std::io;
///
/// # async fn example() -> Result<(), io::Error> {
/// let policy = ExponentialBackoff::default();
/// let mut attempt = 0;
///
/// let result = retry_with_policy(
///     || async {
///         attempt += 1;
///         if attempt < 3 {
///             Err(io::Error::new(io::ErrorKind::Other, "temporary error"))
///         } else {
///             Ok("success")
///         }
///     },
///     &policy,
/// ).await?;
/// # Ok(())
/// # }
/// ```
pub async fn retry_with_policy<F, Fut, T, E>(
    mut operation: F,
    policy: &dyn RetryPolicy,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::error::Error + 'static,
{
    let mut attempt = 0;
    let max_attempts = policy.max_attempts();

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                if attempt >= max_attempts {
                    return Err(error);
                }

                let decision = policy.should_retry(attempt, &error);

                match decision {
                    RetryDecision::Retry(delay) => {
                        if delay > Duration::ZERO {
                            sleep(delay).await;
                        }
                        attempt += 1;
                    }
                    RetryDecision::Stop => {
                        return Err(error);
                    }
                }
            }
        }
    }
}

/// Retries a `Retryable` operation according to a retry policy.
///
/// # Arguments
///
/// * `retryable` - An implementation of the `Retryable` trait.
/// * `policy` - The retry policy to use.
///
/// # Returns
///
/// The result of the operation if successful, or the last error encountered.
pub async fn retry_retryable<R>(
    retryable: &mut R,
    policy: &dyn RetryPolicy,
) -> Result<R::Output, R::Error>
where
    R: Retryable,
{
    let mut attempt = 0;
    let max_attempts = policy.max_attempts();

    loop {
        match retryable.execute().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                if !retryable.is_retryable(&error) {
                    return Err(error);
                }

                if attempt >= max_attempts {
                    return Err(error);
                }

                let decision = policy.should_retry(attempt, &error);

                match decision {
                    RetryDecision::Retry(delay) => {
                        if delay > Duration::ZERO {
                            sleep(delay).await;
                        }
                        attempt += 1;
                    }
                    RetryDecision::Stop => {
                        return Err(error);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategies::FixedDelay;
    use std::io;

    #[tokio::test]
    async fn test_retry_with_policy_success() {
        let policy = FixedDelay::new(Duration::from_millis(10), 3);
        let mut attempts = 0;

        let result = retry_with_policy(
            || async {
                attempts += 1;
                if attempts < 2 {
                    Err(io::Error::new(io::ErrorKind::Other, "fail"))
                } else {
                    Ok("success")
                }
            },
            &policy,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempts, 2);
    }

    #[tokio::test]
    async fn test_retry_with_policy_exhausted() {
        let policy = FixedDelay::new(Duration::from_millis(10), 2);
        let mut attempts = 0;

        let result = retry_with_policy(
            || async {
                attempts += 1;
                Err(io::Error::new(io::ErrorKind::Other, "always fail"))
            },
            &policy,
        )
        .await;

        assert!(result.is_err());
        assert_eq!(attempts, 3); // Initial attempt + 2 retries
    }

    struct TestRetryable {
        attempts: u32,
        fail_until: u32,
    }

    #[async_trait]
    impl Retryable for TestRetryable {
        type Output = String;
        type Error = io::Error;

        async fn execute(&mut self) -> Result<Self::Output, Self::Error> {
            self.attempts += 1;
            if self.attempts < self.fail_until {
                Err(io::Error::new(io::ErrorKind::Other, "not yet"))
            } else {
                Ok("done".to_string())
            }
        }
    }

    #[tokio::test]
    async fn test_retry_retryable() {
        let policy = FixedDelay::new(Duration::from_millis(10), 5);
        let mut retryable = TestRetryable {
            attempts: 0,
            fail_until: 3,
        };

        let result = retry_retryable(&mut retryable, &policy).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "done");
        assert_eq!(retryable.attempts, 3);
    }
}
