//! Chaos testing utilities.

use rand::Rng;
use std::time::Duration;

/// Chaos mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChaosMode {
    /// No chaos
    Disabled,
    /// Inject failures randomly
    Random,
    /// Always fail
    AlwaysFail,
    /// Fail with a specific probability
    Probabilistic,
}

/// Chaos injection configuration
#[derive(Debug, Clone)]
pub struct ChaosConfig {
    /// Chaos mode
    pub mode: ChaosMode,
    /// Failure probability (0.0 to 1.0)
    pub failure_probability: f64,
    /// Latency injection
    pub latency: Option<LatencyConfig>,
    /// Error message to return
    pub error_message: String,
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            mode: ChaosMode::Disabled,
            failure_probability: 0.1,
            latency: None,
            error_message: "Chaos failure".to_string(),
        }
    }
}

/// Latency injection configuration
#[derive(Debug, Clone)]
pub struct LatencyConfig {
    /// Minimum latency
    pub min: Duration,
    /// Maximum latency
    pub max: Duration,
    /// Probability of injecting latency
    pub probability: f64,
}

impl LatencyConfig {
    /// Create a new latency config
    pub fn new(min: Duration, max: Duration) -> Self {
        Self {
            min,
            max,
            probability: 1.0,
        }
    }

    /// Set probability
    pub fn probability(mut self, p: f64) -> Self {
        self.probability = p.clamp(0.0, 1.0);
        self
    }
}

/// Chaos injector
pub struct ChaosInjector {
    config: ChaosConfig,
}

impl ChaosInjector {
    /// Create a new chaos injector
    pub fn new(config: ChaosConfig) -> Self {
        Self { config }
    }

    /// Create with random failures
    pub fn random(failure_probability: f64) -> Self {
        Self::new(ChaosConfig {
            mode: ChaosMode::Probabilistic,
            failure_probability: failure_probability.clamp(0.0, 1.0),
            ..Default::default()
        })
    }

    /// Create with latency injection
    pub fn with_latency(min: Duration, max: Duration) -> Self {
        Self::new(ChaosConfig {
            mode: ChaosMode::Disabled,
            latency: Some(LatencyConfig::new(min, max)),
            ..Default::default()
        })
    }

    /// Check if a failure should be injected
    pub fn should_fail(&self) -> bool {
        match self.config.mode {
            ChaosMode::Disabled => false,
            ChaosMode::AlwaysFail => true,
            ChaosMode::Random | ChaosMode::Probabilistic => {
                let mut rng = rand::thread_rng();
                rng.gen::<f64>() < self.config.failure_probability
            }
        }
    }

    /// Get the latency to inject (if any)
    pub fn latency(&self) -> Option<Duration> {
        if let Some(ref latency_config) = self.config.latency {
            let mut rng = rand::thread_rng();
            if rng.gen::<f64>() < latency_config.probability {
                let range = latency_config.max.as_millis() - latency_config.min.as_millis();
                let delay = latency_config.min.as_millis()
                    + (rng.gen::<f64>() * range as f64) as u128;
                return Some(Duration::from_millis(delay as u64));
            }
        }
        None
    }

    /// Get the error message
    pub fn error_message(&self) -> &str {
        &self.config.error_message
    }

    /// Apply chaos (returns error if failure should be injected)
    pub fn apply<T>(&self, value: T) -> Result<T, String> {
        if self.should_fail() {
            Err(self.config.error_message.clone())
        } else {
            Ok(value)
        }
    }

    /// Apply chaos asynchronously with latency
    pub async fn apply_async<T>(&self, value: T) -> Result<T, String> {
        if let Some(latency) = self.latency() {
            tokio::time::sleep(latency).await;
        }

        self.apply(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chaos_disabled() {
        let injector = ChaosInjector::new(ChaosConfig::default());
        assert!(!injector.should_fail());
    }

    #[test]
    fn test_chaos_always_fail() {
        let injector = ChaosInjector::new(ChaosConfig {
            mode: ChaosMode::AlwaysFail,
            ..Default::default()
        });
        assert!(injector.should_fail());
    }

    #[test]
    fn test_chaos_apply() {
        let injector = ChaosInjector::new(ChaosConfig {
            mode: ChaosMode::AlwaysFail,
            error_message: "Test failure".to_string(),
            ..Default::default()
        });

        let result: Result<i32, String> = injector.apply(42);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Test failure");
    }

    #[test]
    fn test_latency_config() {
        let injector = ChaosInjector::with_latency(
            Duration::from_millis(10),
            Duration::from_millis(100),
        );

        // Latency should be in range
        if let Some(latency) = injector.latency() {
            assert!(latency >= Duration::from_millis(10));
            assert!(latency <= Duration::from_millis(100));
        }
    }
}
