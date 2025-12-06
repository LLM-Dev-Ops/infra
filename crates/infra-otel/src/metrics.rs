//! Metrics utilities.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

/// Counter metric
pub struct Counter {
    value: AtomicU64,
    name: String,
    labels: HashMap<String, String>,
}

impl Counter {
    /// Create a new counter
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            value: AtomicU64::new(0),
            name: name.into(),
            labels: HashMap::new(),
        }
    }

    /// Create with labels
    pub fn with_labels(name: impl Into<String>, labels: HashMap<String, String>) -> Self {
        Self {
            value: AtomicU64::new(0),
            name: name.into(),
            labels,
        }
    }

    /// Increment by 1
    pub fn inc(&self) {
        self.add(1);
    }

    /// Add a value
    pub fn add(&self, value: u64) {
        self.value.fetch_add(value, Ordering::Relaxed);
    }

    /// Get current value
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    /// Get the metric name
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Gauge metric
pub struct Gauge {
    value: AtomicI64,
    name: String,
    labels: HashMap<String, String>,
}

impl Gauge {
    /// Create a new gauge
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            value: AtomicI64::new(0),
            name: name.into(),
            labels: HashMap::new(),
        }
    }

    /// Set the value
    pub fn set(&self, value: i64) {
        self.value.store(value, Ordering::Relaxed);
    }

    /// Increment by 1
    pub fn inc(&self) {
        self.add(1);
    }

    /// Decrement by 1
    pub fn dec(&self) {
        self.add(-1);
    }

    /// Add a value (can be negative)
    pub fn add(&self, value: i64) {
        self.value.fetch_add(value, Ordering::Relaxed);
    }

    /// Get current value
    pub fn get(&self) -> i64 {
        self.value.load(Ordering::Relaxed)
    }

    /// Get the metric name
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Histogram metric
pub struct Histogram {
    buckets: Vec<f64>,
    counts: Vec<AtomicU64>,
    sum: AtomicU64,
    count: AtomicU64,
    name: String,
}

impl Histogram {
    /// Create a new histogram with default buckets
    pub fn new(name: impl Into<String>) -> Self {
        Self::with_buckets(
            name,
            vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        )
    }

    /// Create with custom buckets
    pub fn with_buckets(name: impl Into<String>, buckets: Vec<f64>) -> Self {
        let counts = buckets.iter().map(|_| AtomicU64::new(0)).collect();
        Self {
            buckets,
            counts,
            sum: AtomicU64::new(0),
            count: AtomicU64::new(0),
            name: name.into(),
        }
    }

    /// Observe a value
    pub fn observe(&self, value: f64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.sum.fetch_add(value.to_bits(), Ordering::Relaxed);

        for (i, bucket) in self.buckets.iter().enumerate() {
            if value <= *bucket {
                self.counts[i].fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Get observation count
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    /// Get the metric name
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Metrics registry
pub struct MetricsRegistry {
    counters: RwLock<HashMap<String, Arc<Counter>>>,
    gauges: RwLock<HashMap<String, Arc<Gauge>>>,
    histograms: RwLock<HashMap<String, Arc<Histogram>>>,
}

impl MetricsRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            counters: RwLock::new(HashMap::new()),
            gauges: RwLock::new(HashMap::new()),
            histograms: RwLock::new(HashMap::new()),
        }
    }

    /// Get or create a counter
    pub fn counter(&self, name: &str) -> Arc<Counter> {
        let counters = self.counters.read().unwrap();
        if let Some(counter) = counters.get(name) {
            return Arc::clone(counter);
        }
        drop(counters);

        let mut counters = self.counters.write().unwrap();
        counters
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(Counter::new(name)))
            .clone()
    }

    /// Get or create a gauge
    pub fn gauge(&self, name: &str) -> Arc<Gauge> {
        let gauges = self.gauges.read().unwrap();
        if let Some(gauge) = gauges.get(name) {
            return Arc::clone(gauge);
        }
        drop(gauges);

        let mut gauges = self.gauges.write().unwrap();
        gauges
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(Gauge::new(name)))
            .clone()
    }

    /// Get or create a histogram
    pub fn histogram(&self, name: &str) -> Arc<Histogram> {
        let histograms = self.histograms.read().unwrap();
        if let Some(histogram) = histograms.get(name) {
            return Arc::clone(histogram);
        }
        drop(histograms);

        let mut histograms = self.histograms.write().unwrap();
        histograms
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(Histogram::new(name)))
            .clone()
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let counter = Counter::new("test_counter");
        counter.inc();
        counter.add(5);
        assert_eq!(counter.get(), 6);
    }

    #[test]
    fn test_gauge() {
        let gauge = Gauge::new("test_gauge");
        gauge.set(10);
        gauge.inc();
        gauge.dec();
        assert_eq!(gauge.get(), 10);
    }

    #[test]
    fn test_histogram() {
        let histogram = Histogram::new("test_histogram");
        histogram.observe(0.5);
        histogram.observe(1.5);
        histogram.observe(2.5);
        assert_eq!(histogram.count(), 3);
    }

    #[test]
    fn test_registry() {
        let registry = MetricsRegistry::new();
        let counter1 = registry.counter("test");
        let counter2 = registry.counter("test");
        counter1.inc();
        assert_eq!(counter2.get(), 1);
    }
}
