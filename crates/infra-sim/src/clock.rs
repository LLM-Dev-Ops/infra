//! Clock abstractions for time simulation.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Clock trait for time abstraction
pub trait Clock: Send + Sync {
    /// Get the current time
    fn now(&self) -> Instant;

    /// Sleep for a duration
    fn sleep(&self, duration: Duration);
}

/// System clock (real time)
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }

    fn sleep(&self, duration: Duration) {
        std::thread::sleep(duration);
    }
}

/// Simulated clock for testing
pub struct SimulatedClock {
    base: Instant,
    offset_nanos: AtomicU64,
}

impl SimulatedClock {
    /// Create a new simulated clock
    pub fn new() -> Self {
        Self {
            base: Instant::now(),
            offset_nanos: AtomicU64::new(0),
        }
    }

    /// Advance the clock by a duration
    pub fn advance(&self, duration: Duration) {
        self.offset_nanos
            .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
    }

    /// Set the clock to a specific offset from the base
    pub fn set_offset(&self, duration: Duration) {
        self.offset_nanos
            .store(duration.as_nanos() as u64, Ordering::Relaxed);
    }

    /// Get the current offset
    pub fn offset(&self) -> Duration {
        Duration::from_nanos(self.offset_nanos.load(Ordering::Relaxed))
    }
}

impl Default for SimulatedClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SimulatedClock {
    fn now(&self) -> Instant {
        let offset = Duration::from_nanos(self.offset_nanos.load(Ordering::Relaxed));
        self.base + offset
    }

    fn sleep(&self, duration: Duration) {
        // In simulation, we just advance the clock
        self.advance(duration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_clock() {
        let clock = SystemClock;
        let t1 = clock.now();
        std::thread::sleep(Duration::from_millis(10));
        let t2 = clock.now();
        assert!(t2 > t1);
    }

    #[test]
    fn test_simulated_clock() {
        let clock = SimulatedClock::new();
        let t1 = clock.now();

        clock.advance(Duration::from_secs(60));
        let t2 = clock.now();

        assert!(t2 > t1);
        assert!(t2 - t1 >= Duration::from_secs(60));
    }

    #[test]
    fn test_simulated_clock_sleep() {
        let clock = SimulatedClock::new();
        let initial_offset = clock.offset();

        clock.sleep(Duration::from_secs(30));

        assert_eq!(clock.offset() - initial_offset, Duration::from_secs(30));
    }
}
