//! Simulation and testing utilities for LLM-Dev-Ops infrastructure.
//!
//! This crate provides mock implementations and simulation utilities
//! for testing infrastructure components.

mod clock;
mod mock;
mod scenario;
mod chaos;

pub use clock::{Clock, SimulatedClock, SystemClock};
pub use mock::{MockService, MockResponse, MockBuilder};
pub use scenario::{Scenario, ScenarioBuilder, Step};
pub use chaos::{ChaosConfig, ChaosMode, ChaosInjector};

use std::sync::Arc;
use tokio::sync::RwLock;

/// Global simulated clock for testing
static GLOBAL_CLOCK: RwLock<Option<Arc<dyn Clock>>> = RwLock::const_new(None);

/// Set the global clock
pub async fn set_clock(clock: Arc<dyn Clock>) {
    let mut global = GLOBAL_CLOCK.write().await;
    *global = Some(clock);
}

/// Get the current time from the global clock
pub async fn now() -> std::time::Instant {
    let global = GLOBAL_CLOCK.read().await;
    if let Some(clock) = global.as_ref() {
        clock.now()
    } else {
        std::time::Instant::now()
    }
}

/// Reset the global clock to use the system clock
pub async fn reset_clock() {
    let mut global = GLOBAL_CLOCK.write().await;
    *global = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulated_clock() {
        let clock = SimulatedClock::new();
        let t1 = clock.now();

        clock.advance(std::time::Duration::from_secs(10));
        let t2 = clock.now();

        assert!(t2 > t1);
    }
}
