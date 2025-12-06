# SPARC Phase 4: Refinement

## LLM-Dev-Ops Unified Infrastructure Layer

**Document Version:** 1.0
**Date:** 2025-12-06
**Status:** Draft - Pending User Approval
**Previous Phase:** [03-architecture.md](./03-architecture.md)

---

## 1. Overview

This phase refines the architecture through:
- Error handling optimization and recovery strategies
- API ergonomics improvements
- Edge case handling and failure modes
- Performance optimization guidelines
- Security hardening measures
- Testing strategies
- Migration paths from RuvNet ecosystem
- Troubleshooting and debugging support

---

## 2. Error Handling Refinement

### 2.1 Error Context Enhancement

The base `InfraError` is enhanced with rich context for debugging:

```rust
// ============================================================
// Enhanced Error Context System
// ============================================================

/// Context that can be attached to any InfraError
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Unique error instance ID for correlation
    pub error_id: String,

    /// Timestamp when error occurred
    pub timestamp: DateTime<Utc>,

    /// Source location (file, line, column)
    pub location: Option<SourceLocation>,

    /// Call chain leading to the error
    pub backtrace: Vec<FrameInfo>,

    /// Related span IDs for distributed tracing
    pub trace_ids: TraceIds,

    /// Key-value pairs for additional context
    pub attributes: HashMap<String, String>,

    /// Suggested remediation steps
    pub remediation: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
    pub function: &'static str,
}

#[derive(Debug, Clone)]
pub struct TraceIds {
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub parent_span_id: Option<String>,
}

/// Macro for capturing source location
#[macro_export]
macro_rules! infra_error {
    ($kind:expr) => {
        $kind.with_context(infra_errors::ErrorContext {
            error_id: infra_id::generate_error_id(),
            timestamp: chrono::Utc::now(),
            location: Some(infra_errors::SourceLocation {
                file: file!(),
                line: line!(),
                column: column!(),
                function: infra_errors::function_name!(),
            }),
            backtrace: Vec::new(),
            trace_ids: infra_errors::TraceIds::from_current_span(),
            attributes: std::collections::HashMap::new(),
            remediation: None,
        })
    };

    ($kind:expr, $($key:expr => $value:expr),+ $(,)?) => {{
        let mut err = infra_error!($kind);
        $(
            err.context_mut().attributes.insert($key.to_string(), $value.to_string());
        )+
        err
    }};
}
```

### 2.2 Error Recovery Strategies

```rust
// ============================================================
// Recovery Strategy Framework
// ============================================================

/// Defines how to recover from specific error types
pub trait RecoveryStrategy: Send + Sync {
    /// Check if this strategy can handle the error
    fn can_handle(&self, error: &InfraError) -> bool;

    /// Attempt recovery, returning Ok(T) on success or the original error
    async fn recover<T, F, Fut>(&self, error: InfraError, retry_fn: F) -> InfraResult<T>
    where
        F: Fn() -> Fut + Send,
        Fut: Future<Output = InfraResult<T>> + Send;

    /// Get the maximum number of recovery attempts
    fn max_attempts(&self) -> usize { 3 }
}

/// Exponential backoff recovery for transient errors
pub struct ExponentialBackoffRecovery {
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub jitter: bool,
}

impl RecoveryStrategy for ExponentialBackoffRecovery {
    fn can_handle(&self, error: &InfraError) -> bool {
        error.is_retryable()
    }

    async fn recover<T, F, Fut>(&self, error: InfraError, retry_fn: F) -> InfraResult<T>
    where
        F: Fn() -> Fut + Send,
        Fut: Future<Output = InfraResult<T>> + Send,
    {
        let mut attempts = 0;
        let mut last_error = error;

        while attempts < self.max_attempts() {
            let delay = self.calculate_delay(attempts);

            tracing::warn!(
                error = %last_error,
                attempt = attempts + 1,
                delay_ms = delay.as_millis(),
                "Retrying after transient error"
            );

            tokio::time::sleep(delay).await;

            match retry_fn().await {
                Ok(result) => return Ok(result),
                Err(e) if e.is_retryable() => {
                    last_error = e;
                    attempts += 1;
                }
                Err(e) => return Err(e),
            }
        }

        Err(last_error)
    }
}

impl ExponentialBackoffRecovery {
    fn calculate_delay(&self, attempt: usize) -> Duration {
        let base = self.base_delay.as_millis() as u64;
        let exponential = base * 2u64.pow(attempt as u32);
        let capped = exponential.min(self.max_delay.as_millis() as u64);

        let delay = if self.jitter {
            let jitter_range = capped / 4;
            let jitter = rand::thread_rng().gen_range(0..=jitter_range);
            capped + jitter
        } else {
            capped
        };

        Duration::from_millis(delay)
    }
}

/// Circuit breaker for preventing cascade failures
pub struct CircuitBreakerRecovery {
    state: Arc<RwLock<CircuitState>>,
    failure_threshold: usize,
    success_threshold: usize,
    timeout: Duration,
}

#[derive(Debug, Clone)]
enum CircuitState {
    Closed { failures: usize },
    Open { opened_at: Instant },
    HalfOpen { successes: usize },
}

impl RecoveryStrategy for CircuitBreakerRecovery {
    fn can_handle(&self, _error: &InfraError) -> bool {
        true // Applies to all errors
    }

    async fn recover<T, F, Fut>(&self, error: InfraError, retry_fn: F) -> InfraResult<T>
    where
        F: Fn() -> Fut + Send,
        Fut: Future<Output = InfraResult<T>> + Send,
    {
        let state = self.state.read().await.clone();

        match state {
            CircuitState::Open { opened_at } => {
                if opened_at.elapsed() >= self.timeout {
                    // Transition to half-open
                    *self.state.write().await = CircuitState::HalfOpen { successes: 0 };
                    self.try_request(retry_fn).await
                } else {
                    Err(InfraError::External {
                        service: "circuit_breaker".to_string(),
                        operation: "request".to_string(),
                        message: "Circuit breaker is open".to_string(),
                        retry_after: Some(self.timeout - opened_at.elapsed()),
                    })
                }
            }
            CircuitState::Closed { .. } | CircuitState::HalfOpen { .. } => {
                self.try_request(retry_fn).await
            }
        }
    }
}
```

### 2.3 Error Aggregation for Batch Operations

```rust
// ============================================================
// Batch Error Handling
// ============================================================

/// Aggregates multiple errors from batch operations
#[derive(Debug)]
pub struct BatchError {
    /// Total items in the batch
    pub total: usize,

    /// Successfully processed items
    pub succeeded: usize,

    /// Failed items with their errors
    pub failures: Vec<BatchFailure>,

    /// Whether the batch should be considered failed overall
    pub is_partial_success: bool,
}

#[derive(Debug)]
pub struct BatchFailure {
    /// Index or ID of the failed item
    pub item_id: String,

    /// The error that occurred
    pub error: InfraError,

    /// Whether this item can be retried
    pub retryable: bool,
}

impl BatchError {
    pub fn new(total: usize) -> Self {
        Self {
            total,
            succeeded: 0,
            failures: Vec::new(),
            is_partial_success: false,
        }
    }

    pub fn record_success(&mut self) {
        self.succeeded += 1;
    }

    pub fn record_failure(&mut self, item_id: impl Into<String>, error: InfraError) {
        let retryable = error.is_retryable();
        self.failures.push(BatchFailure {
            item_id: item_id.into(),
            error,
            retryable,
        });
    }

    pub fn finalize(mut self) -> InfraResult<BatchResult> {
        self.is_partial_success = self.succeeded > 0 && !self.failures.is_empty();

        if self.failures.is_empty() {
            Ok(BatchResult::Success { count: self.succeeded })
        } else if self.succeeded == 0 {
            Err(InfraError::Batch(self))
        } else {
            Ok(BatchResult::PartialSuccess {
                succeeded: self.succeeded,
                failed: self.failures.len(),
                errors: self,
            })
        }
    }

    /// Get retryable failures for retry logic
    pub fn retryable_items(&self) -> Vec<&BatchFailure> {
        self.failures.iter().filter(|f| f.retryable).collect()
    }
}

pub enum BatchResult {
    Success { count: usize },
    PartialSuccess {
        succeeded: usize,
        failed: usize,
        errors: BatchError,
    },
}
```

---

## 3. API Ergonomics Refinement

### 3.1 Builder Pattern Standardization

All complex types use a consistent builder pattern:

```rust
// ============================================================
// Standardized Builder Pattern
// ============================================================

/// Trait for all builders
pub trait Builder<T> {
    type Error;

    /// Build the final object, consuming the builder
    fn build(self) -> Result<T, Self::Error>;

    /// Validate the current builder state without building
    fn validate(&self) -> Result<(), Self::Error>;
}

/// Example: VectorStoreConfig builder
pub struct VectorStoreConfigBuilder {
    collection_name: Option<String>,
    dimensions: Option<usize>,
    distance: Distance,
    hnsw: HnswConfigBuilder,
    compression: CompressionConfigBuilder,
}

impl VectorStoreConfigBuilder {
    pub fn new() -> Self {
        Self {
            collection_name: None,
            dimensions: None,
            distance: Distance::Cosine,
            hnsw: HnswConfigBuilder::default(),
            compression: CompressionConfigBuilder::default(),
        }
    }

    /// Required: Set collection name
    pub fn collection(mut self, name: impl Into<String>) -> Self {
        self.collection_name = Some(name.into());
        self
    }

    /// Required: Set vector dimensions
    pub fn dimensions(mut self, dims: usize) -> Self {
        self.dimensions = Some(dims);
        self
    }

    /// Optional: Set distance metric (default: Cosine)
    pub fn distance(mut self, distance: Distance) -> Self {
        self.distance = distance;
        self
    }

    /// Optional: Configure HNSW parameters
    pub fn hnsw(mut self, config: impl FnOnce(HnswConfigBuilder) -> HnswConfigBuilder) -> Self {
        self.hnsw = config(self.hnsw);
        self
    }

    /// Optional: Configure compression
    pub fn compression(mut self, config: impl FnOnce(CompressionConfigBuilder) -> CompressionConfigBuilder) -> Self {
        self.compression = config(self.compression);
        self
    }
}

impl Builder<VectorStoreConfig> for VectorStoreConfigBuilder {
    type Error = InfraError;

    fn validate(&self) -> InfraResult<()> {
        if self.collection_name.is_none() {
            return Err(infra_error!(InfraError::Validation {
                field: Some("collection_name".to_string()),
                message: "Collection name is required".to_string(),
                expected: Some("non-empty string".to_string()),
                actual: None,
            }));
        }

        if self.dimensions.is_none() || self.dimensions == Some(0) {
            return Err(infra_error!(InfraError::Validation {
                field: Some("dimensions".to_string()),
                message: "Dimensions must be a positive integer".to_string(),
                expected: Some("> 0".to_string()),
                actual: self.dimensions.map(|d| d.to_string()),
            }));
        }

        Ok(())
    }

    fn build(self) -> InfraResult<VectorStoreConfig> {
        self.validate()?;

        Ok(VectorStoreConfig {
            collection_name: self.collection_name.unwrap(),
            dimensions: self.dimensions.unwrap(),
            distance: self.distance,
            hnsw: self.hnsw.build()?,
            compression: self.compression.build()?,
        })
    }
}

// Fluent usage:
let config = VectorStoreConfig::builder()
    .collection("embeddings")
    .dimensions(1536)
    .distance(Distance::Cosine)
    .hnsw(|h| h.m(32).ef_construction(400))
    .compression(|c| c.enabled(true).auto_tier(true))
    .build()?;
```

### 3.2 Async Trait Ergonomics

```rust
// ============================================================
// Async Extensions for Better Ergonomics
// ============================================================

/// Extension trait for async operations with common patterns
#[async_trait]
pub trait AsyncExt<T> {
    /// Execute with timeout
    async fn with_timeout(self, duration: Duration) -> InfraResult<T>;

    /// Execute with retry on transient failures
    async fn with_retry(self, config: RetryConfig) -> InfraResult<T>;

    /// Execute with both timeout and retry
    async fn with_timeout_retry(self, timeout: Duration, retry: RetryConfig) -> InfraResult<T>;

    /// Map the success value
    async fn map_ok<U, F>(self, f: F) -> InfraResult<U>
    where
        F: FnOnce(T) -> U + Send;

    /// Inspect the result without consuming it
    async fn inspect_ok<F>(self, f: F) -> InfraResult<T>
    where
        F: FnOnce(&T) + Send;
}

impl<F, T> AsyncExt<T> for F
where
    F: Future<Output = InfraResult<T>> + Send,
    T: Send,
{
    async fn with_timeout(self, duration: Duration) -> InfraResult<T> {
        tokio::time::timeout(duration, self)
            .await
            .map_err(|_| InfraError::External {
                service: "timeout".to_string(),
                operation: "async_operation".to_string(),
                message: format!("Operation timed out after {:?}", duration),
                retry_after: None,
            })?
    }

    async fn with_retry(self, config: RetryConfig) -> InfraResult<T> {
        // Implementation using ExponentialBackoffRecovery
        todo!()
    }

    // ... other implementations
}

// Usage:
let result = vector_store
    .search(query, 10, None)
    .with_timeout(Duration::from_secs(5))
    .with_retry(RetryConfig::default())
    .await?;
```

### 3.3 Type-Safe Configuration Keys

```rust
// ============================================================
// Type-Safe Configuration Access
// ============================================================

/// Macro for defining type-safe config keys
#[macro_export]
macro_rules! define_config_keys {
    (
        $(
            $(#[$meta:meta])*
            $name:ident: $type:ty = $key:literal $(=> $default:expr)?
        ),* $(,)?
    ) => {
        pub mod config_keys {
            use super::*;

            $(
                $(#[$meta])*
                pub struct $name;

                impl ConfigKey for $name {
                    type Value = $type;

                    fn key() -> &'static str {
                        $key
                    }

                    fn default() -> Option<Self::Value> {
                        define_config_keys!(@default $($default)?)
                    }
                }
            )*
        }
    };

    (@default $default:expr) => { Some($default) };
    (@default) => { None };
}

// Define keys for vector store
define_config_keys! {
    /// Collection name for the vector store
    VectorCollection: String = "vector.collection",

    /// Vector dimensions
    VectorDimensions: usize = "vector.dimensions",

    /// Distance metric
    VectorDistance: Distance = "vector.distance" => Distance::Cosine,

    /// HNSW M parameter
    HnswM: usize = "vector.hnsw.m" => 16,

    /// HNSW ef_construction
    HnswEfConstruction: usize = "vector.hnsw.ef_construction" => 200,
}

// Type-safe access:
impl Config {
    pub fn get_typed<K: ConfigKey>(&self) -> InfraResult<K::Value> {
        self.get(K::key())
            .or_else(|| K::default())
            .ok_or_else(|| InfraError::Config {
                message: format!("Missing required config: {}", K::key()),
                key: Some(K::key().to_string()),
                source: None,
            })
    }
}

// Usage:
let dimensions: usize = config.get_typed::<config_keys::VectorDimensions>()?;
let distance: Distance = config.get_typed::<config_keys::VectorDistance>()?;
```

### 3.4 Method Chaining for Queries

```rust
// ============================================================
// Fluent Query Builder for Vector Search
// ============================================================

pub struct SearchQuery<'a> {
    store: &'a dyn VectorStore,
    vector: Vector,
    k: usize,
    filter: Option<MetadataFilter>,
    include_vectors: bool,
    include_metadata: bool,
    score_threshold: Option<f32>,
    ef_search: Option<usize>,
}

impl<'a> SearchQuery<'a> {
    pub fn new(store: &'a dyn VectorStore, vector: Vector) -> Self {
        Self {
            store,
            vector,
            k: 10,
            filter: None,
            include_vectors: false,
            include_metadata: true,
            score_threshold: None,
            ef_search: None,
        }
    }

    /// Set the number of results to return
    pub fn limit(mut self, k: usize) -> Self {
        self.k = k;
        self
    }

    /// Add a metadata filter
    pub fn filter(mut self, filter: MetadataFilter) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Filter by exact field match
    pub fn where_eq(self, field: &str, value: impl Into<Json>) -> Self {
        let new_filter = MetadataFilter::Eq {
            field: field.to_string(),
            value: value.into(),
        };
        self.and_filter(new_filter)
    }

    /// Filter by field in list
    pub fn where_in(self, field: &str, values: Vec<impl Into<Json>>) -> Self {
        let new_filter = MetadataFilter::In {
            field: field.to_string(),
            values: values.into_iter().map(Into::into).collect(),
        };
        self.and_filter(new_filter)
    }

    /// Combine with existing filter using AND
    fn and_filter(mut self, new_filter: MetadataFilter) -> Self {
        self.filter = Some(match self.filter {
            Some(existing) => MetadataFilter::And(vec![existing, new_filter]),
            None => new_filter,
        });
        self
    }

    /// Include vectors in results
    pub fn with_vectors(mut self) -> Self {
        self.include_vectors = true;
        self
    }

    /// Exclude metadata from results
    pub fn without_metadata(mut self) -> Self {
        self.include_metadata = false;
        self
    }

    /// Only return results above this score
    pub fn min_score(mut self, threshold: f32) -> Self {
        self.score_threshold = Some(threshold);
        self
    }

    /// Set HNSW ef_search parameter
    pub fn ef_search(mut self, ef: usize) -> Self {
        self.ef_search = Some(ef);
        self
    }

    /// Execute the search
    pub async fn execute(self) -> InfraResult<Vec<SearchResult>> {
        let mut results = self.store.search(self.vector, self.k, self.filter).await?;

        // Apply score threshold
        if let Some(threshold) = self.score_threshold {
            results.retain(|r| r.score >= threshold);
        }

        // Strip vectors if not requested
        if !self.include_vectors {
            for result in &mut results {
                result.vector = None;
            }
        }

        // Strip metadata if not requested
        if !self.include_metadata {
            for result in &mut results {
                result.metadata = None;
            }
        }

        Ok(results)
    }
}

// Fluent usage:
let results = vector_store
    .query(embedding)
    .limit(20)
    .where_eq("category", "technology")
    .where_in("tags", vec!["rust", "wasm"])
    .min_score(0.8)
    .with_vectors()
    .execute()
    .await?;
```

---

## 4. Edge Cases and Failure Modes

### 4.1 Failure Mode Catalog

| Component | Failure Mode | Detection | Recovery |
|-----------|--------------|-----------|----------|
| **infra-vector** | Index corruption | Checksum validation | Rebuild from source |
| **infra-vector** | Dimension mismatch | Runtime check | Clear error message |
| **infra-vector** | OOM during indexing | Memory monitor | Batch size reduction |
| **infra-http** | Connection timeout | Timeout detection | Retry with backoff |
| **infra-http** | SSL certificate error | TLS validation | Fail with clear message |
| **infra-http** | DNS resolution failure | Name resolution | Retry + fallback DNS |
| **infra-config** | File not found | IO error | Use defaults or fail |
| **infra-config** | Parse error | Deserialization | Show line/column |
| **infra-config** | Schema validation | Validator | Detailed errors |
| **infra-crypto** | Invalid key size | Size check | Clear requirements |
| **infra-crypto** | Decryption failure | Tag verification | No data returned |
| **infra-otel** | Exporter connection failed | Health check | Buffer + retry |
| **infra-otel** | Span buffer overflow | Counter | Sampling increase |
| **infra-mq** | Broker disconnect | Heartbeat | Auto-reconnect |
| **infra-mq** | Message too large | Size check | Reject with limit |

### 4.2 Graceful Degradation Patterns

```rust
// ============================================================
// Graceful Degradation Framework
// ============================================================

/// Trait for services that can operate in degraded mode
pub trait Degradable {
    /// Current health status
    fn health(&self) -> HealthStatus;

    /// Whether the service is operating in degraded mode
    fn is_degraded(&self) -> bool;

    /// Get current capabilities
    fn capabilities(&self) -> Capabilities;
}

#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded { reason: String, since: Instant },
    Unhealthy { reason: String, since: Instant },
}

#[derive(Debug, Clone)]
pub struct Capabilities {
    pub can_read: bool,
    pub can_write: bool,
    pub can_search: bool,
    pub max_batch_size: Option<usize>,
    pub estimated_latency: Option<Duration>,
}

/// Vector store with graceful degradation
pub struct ResilientVectorStore {
    primary: Box<dyn VectorStore>,
    fallback: Option<Box<dyn VectorStore>>,
    health_checker: HealthChecker,
}

impl ResilientVectorStore {
    pub async fn search(
        &self,
        query: Vector,
        k: usize,
        filter: Option<MetadataFilter>,
    ) -> InfraResult<Vec<SearchResult>> {
        // Try primary first
        match self.primary.search(query.clone(), k, filter.clone()).await {
            Ok(results) => Ok(results),
            Err(e) if self.fallback.is_some() && e.is_retryable() => {
                tracing::warn!(error = %e, "Primary search failed, using fallback");
                self.health_checker.record_failure();

                // Use fallback
                self.fallback
                    .as_ref()
                    .unwrap()
                    .search(query, k, filter)
                    .await
            }
            Err(e) => {
                self.health_checker.record_failure();
                Err(e)
            }
        }
    }
}
```

### 4.3 Resource Exhaustion Handling

```rust
// ============================================================
// Resource Management
// ============================================================

/// Bounded resource pool with backpressure
pub struct ResourcePool<T> {
    resources: Arc<ArrayQueue<T>>,
    max_size: usize,
    waiters: Arc<Semaphore>,
    creation_fn: Box<dyn Fn() -> InfraResult<T> + Send + Sync>,
}

impl<T: Send + 'static> ResourcePool<T> {
    pub fn new(
        max_size: usize,
        creation_fn: impl Fn() -> InfraResult<T> + Send + Sync + 'static,
    ) -> Self {
        Self {
            resources: Arc::new(ArrayQueue::new(max_size)),
            max_size,
            waiters: Arc::new(Semaphore::new(max_size)),
            creation_fn: Box::new(creation_fn),
        }
    }

    /// Acquire a resource with timeout
    pub async fn acquire(&self, timeout: Duration) -> InfraResult<PooledResource<T>> {
        // Try to acquire permit with timeout
        let permit = tokio::time::timeout(
            timeout,
            self.waiters.clone().acquire_owned(),
        )
        .await
        .map_err(|_| InfraError::External {
            service: "resource_pool".to_string(),
            operation: "acquire".to_string(),
            message: "Timed out waiting for available resource".to_string(),
            retry_after: Some(Duration::from_secs(1)),
        })?
        .map_err(|_| InfraError::External {
            service: "resource_pool".to_string(),
            operation: "acquire".to_string(),
            message: "Pool closed".to_string(),
            retry_after: None,
        })?;

        // Try to get existing resource or create new one
        let resource = match self.resources.pop() {
            Some(r) => r,
            None => (self.creation_fn)()?,
        };

        Ok(PooledResource {
            resource: Some(resource),
            pool: self.resources.clone(),
            _permit: permit,
        })
    }
}

pub struct PooledResource<T> {
    resource: Option<T>,
    pool: Arc<ArrayQueue<T>>,
    _permit: OwnedSemaphorePermit,
}

impl<T> Drop for PooledResource<T> {
    fn drop(&mut self) {
        if let Some(resource) = self.resource.take() {
            // Return resource to pool
            let _ = self.pool.push(resource);
        }
    }
}

impl<T> Deref for PooledResource<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.resource.as_ref().unwrap()
    }
}
```

---

## 5. Performance Optimization Guidelines

### 5.1 Memory Optimization

```rust
// ============================================================
// Memory-Efficient Patterns
// ============================================================

/// Zero-copy string handling for high-throughput scenarios
pub struct InternedString {
    inner: Arc<str>,
}

impl InternedString {
    thread_local! {
        static INTERNER: RefCell<HashSet<Arc<str>>> = RefCell::new(HashSet::new());
    }

    pub fn new(s: &str) -> Self {
        Self::INTERNER.with(|interner| {
            let mut set = interner.borrow_mut();
            if let Some(existing) = set.get(s) {
                Self { inner: Arc::clone(existing) }
            } else {
                let arc: Arc<str> = Arc::from(s);
                set.insert(Arc::clone(&arc));
                Self { inner: arc }
            }
        })
    }
}

/// Memory-mapped vector storage for large datasets
pub struct MmapVectorStorage {
    mmap: memmap2::Mmap,
    dimensions: usize,
    count: usize,
}

impl MmapVectorStorage {
    pub fn open(path: &Path, dimensions: usize) -> InfraResult<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { memmap2::Mmap::map(&file)? };
        let count = mmap.len() / (dimensions * std::mem::size_of::<f32>());

        Ok(Self { mmap, dimensions, count })
    }

    pub fn get(&self, index: usize) -> Option<&[f32]> {
        if index >= self.count {
            return None;
        }

        let offset = index * self.dimensions * std::mem::size_of::<f32>();
        let bytes = &self.mmap[offset..offset + self.dimensions * std::mem::size_of::<f32>()];

        // Safety: We know the file contains properly aligned f32 values
        Some(unsafe {
            std::slice::from_raw_parts(
                bytes.as_ptr() as *const f32,
                self.dimensions,
            )
        })
    }
}
```

### 5.2 CPU Optimization

```rust
// ============================================================
// SIMD-Accelerated Operations
// ============================================================

/// SIMD-accelerated vector operations
pub mod simd {
    use std::simd::{f32x8, SimdFloat};

    /// Compute L2 distance using SIMD
    #[inline]
    pub fn l2_distance_simd(a: &[f32], b: &[f32]) -> f32 {
        assert_eq!(a.len(), b.len());

        let chunks = a.len() / 8;
        let mut sum = f32x8::splat(0.0);

        for i in 0..chunks {
            let va = f32x8::from_slice(&a[i * 8..]);
            let vb = f32x8::from_slice(&b[i * 8..]);
            let diff = va - vb;
            sum += diff * diff;
        }

        let mut result = sum.reduce_sum();

        // Handle remainder
        for i in (chunks * 8)..a.len() {
            let diff = a[i] - b[i];
            result += diff * diff;
        }

        result.sqrt()
    }

    /// Compute cosine similarity using SIMD
    #[inline]
    pub fn cosine_similarity_simd(a: &[f32], b: &[f32]) -> f32 {
        assert_eq!(a.len(), b.len());

        let chunks = a.len() / 8;
        let mut dot = f32x8::splat(0.0);
        let mut norm_a = f32x8::splat(0.0);
        let mut norm_b = f32x8::splat(0.0);

        for i in 0..chunks {
            let va = f32x8::from_slice(&a[i * 8..]);
            let vb = f32x8::from_slice(&b[i * 8..]);
            dot += va * vb;
            norm_a += va * va;
            norm_b += vb * vb;
        }

        let dot_sum = dot.reduce_sum();
        let norm_a_sum = norm_a.reduce_sum();
        let norm_b_sum = norm_b.reduce_sum();

        // Handle remainder
        let mut dot_rem = 0.0;
        let mut norm_a_rem = 0.0;
        let mut norm_b_rem = 0.0;

        for i in (chunks * 8)..a.len() {
            dot_rem += a[i] * b[i];
            norm_a_rem += a[i] * a[i];
            norm_b_rem += b[i] * b[i];
        }

        let total_dot = dot_sum + dot_rem;
        let total_norm = ((norm_a_sum + norm_a_rem) * (norm_b_sum + norm_b_rem)).sqrt();

        if total_norm > 0.0 {
            total_dot / total_norm
        } else {
            0.0
        }
    }
}

// Feature-gated selection
#[cfg(target_feature = "avx2")]
pub use simd::*;

#[cfg(not(target_feature = "avx2"))]
pub mod fallback {
    pub fn l2_distance_simd(a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }
}
```

### 5.3 Concurrency Optimization

```rust
// ============================================================
// Concurrent Processing Patterns
// ============================================================

/// Parallel batch processor with work stealing
pub struct ParallelBatchProcessor<T, R> {
    worker_count: usize,
    batch_size: usize,
    processor: Arc<dyn Fn(T) -> InfraResult<R> + Send + Sync>,
}

impl<T: Send + 'static, R: Send + 'static> ParallelBatchProcessor<T, R> {
    pub async fn process(&self, items: Vec<T>) -> InfraResult<Vec<R>> {
        let (tx, rx) = async_channel::bounded(items.len());
        let results = Arc::new(Mutex::new(Vec::with_capacity(items.len())));
        let errors = Arc::new(Mutex::new(Vec::new()));

        // Spawn item sender
        for item in items {
            tx.send(item).await.unwrap();
        }
        drop(tx);

        // Spawn workers
        let mut handles = Vec::new();
        for _ in 0..self.worker_count {
            let rx = rx.clone();
            let processor = Arc::clone(&self.processor);
            let results = Arc::clone(&results);
            let errors = Arc::clone(&errors);

            handles.push(tokio::spawn(async move {
                while let Ok(item) = rx.recv().await {
                    match processor(item) {
                        Ok(result) => results.lock().await.push(result),
                        Err(e) => errors.lock().await.push(e),
                    }
                }
            }));
        }

        // Wait for completion
        for handle in handles {
            handle.await?;
        }

        // Check for errors
        let errors = errors.lock().await;
        if !errors.is_empty() {
            return Err(InfraError::Batch(BatchError {
                total: items.len(),
                succeeded: results.lock().await.len(),
                failures: errors.iter().map(|e| BatchFailure {
                    item_id: "unknown".to_string(),
                    error: e.clone(),
                    retryable: e.is_retryable(),
                }).collect(),
                is_partial_success: true,
            }));
        }

        Ok(Arc::try_unwrap(results).unwrap().into_inner())
    }
}
```

### 5.4 Caching Strategies

```rust
// ============================================================
// Multi-Level Caching
// ============================================================

/// LRU cache with TTL support
pub struct TtlCache<K, V> {
    cache: Arc<RwLock<LinkedHashMap<K, CacheEntry<V>>>>,
    max_size: usize,
    default_ttl: Duration,
}

struct CacheEntry<V> {
    value: V,
    expires_at: Instant,
}

impl<K: Hash + Eq + Clone, V: Clone> TtlCache<K, V> {
    pub fn new(max_size: usize, default_ttl: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(LinkedHashMap::new())),
            max_size,
            default_ttl,
        }
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        let cache = self.cache.read().await;
        cache.get(key).and_then(|entry| {
            if entry.expires_at > Instant::now() {
                Some(entry.value.clone())
            } else {
                None
            }
        })
    }

    pub async fn insert(&self, key: K, value: V) {
        self.insert_with_ttl(key, value, self.default_ttl).await
    }

    pub async fn insert_with_ttl(&self, key: K, value: V, ttl: Duration) {
        let mut cache = self.cache.write().await;

        // Evict expired entries
        let now = Instant::now();
        cache.retain(|_, entry| entry.expires_at > now);

        // Evict oldest if at capacity
        while cache.len() >= self.max_size {
            cache.pop_front();
        }

        cache.insert(key, CacheEntry {
            value,
            expires_at: Instant::now() + ttl,
        });
    }

    pub async fn invalidate(&self, key: &K) {
        self.cache.write().await.remove(key);
    }

    pub async fn clear(&self) {
        self.cache.write().await.clear();
    }
}

/// Two-level cache (hot L1 + warm L2)
pub struct TieredCache<K, V> {
    l1: TtlCache<K, V>,  // Small, fast, short TTL
    l2: TtlCache<K, V>,  // Large, slower, long TTL
}

impl<K: Hash + Eq + Clone, V: Clone> TieredCache<K, V> {
    pub async fn get(&self, key: &K) -> Option<V> {
        // Try L1 first
        if let Some(value) = self.l1.get(key).await {
            return Some(value);
        }

        // Try L2 and promote to L1
        if let Some(value) = self.l2.get(key).await {
            self.l1.insert(key.clone(), value.clone()).await;
            return Some(value);
        }

        None
    }

    pub async fn insert(&self, key: K, value: V) {
        // Insert into both levels
        self.l1.insert(key.clone(), value.clone()).await;
        self.l2.insert(key, value).await;
    }
}
```

---

## 6. Security Hardening

### 6.1 Input Validation Framework

```rust
// ============================================================
// Comprehensive Input Validation
// ============================================================

/// Validated wrapper types
pub mod validated {
    use super::*;

    /// Non-empty string that has been validated
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct NonEmptyString(String);

    impl NonEmptyString {
        pub fn new(s: impl Into<String>) -> InfraResult<Self> {
            let s = s.into();
            if s.is_empty() {
                return Err(InfraError::Validation {
                    field: None,
                    message: "String cannot be empty".to_string(),
                    expected: Some("non-empty string".to_string()),
                    actual: Some("empty string".to_string()),
                });
            }
            Ok(Self(s))
        }

        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    /// Bounded string with max length
    #[derive(Debug, Clone)]
    pub struct BoundedString<const MAX: usize>(String);

    impl<const MAX: usize> BoundedString<MAX> {
        pub fn new(s: impl Into<String>) -> InfraResult<Self> {
            let s = s.into();
            if s.len() > MAX {
                return Err(InfraError::Validation {
                    field: None,
                    message: format!("String exceeds maximum length of {}", MAX),
                    expected: Some(format!("<= {} chars", MAX)),
                    actual: Some(format!("{} chars", s.len())),
                });
            }
            Ok(Self(s))
        }
    }

    /// Validated email address
    #[derive(Debug, Clone)]
    pub struct Email(String);

    impl Email {
        pub fn new(s: impl Into<String>) -> InfraResult<Self> {
            let s = s.into();
            // Basic email validation
            if !s.contains('@') || s.len() < 3 {
                return Err(InfraError::Validation {
                    field: Some("email".to_string()),
                    message: "Invalid email format".to_string(),
                    expected: Some("valid email address".to_string()),
                    actual: Some(s),
                });
            }
            Ok(Self(s))
        }
    }

    /// Validated vector dimensions
    #[derive(Debug, Clone, Copy)]
    pub struct Dimensions(usize);

    impl Dimensions {
        pub const MIN: usize = 1;
        pub const MAX: usize = 65536;

        pub fn new(d: usize) -> InfraResult<Self> {
            if d < Self::MIN || d > Self::MAX {
                return Err(InfraError::Validation {
                    field: Some("dimensions".to_string()),
                    message: format!("Dimensions must be between {} and {}", Self::MIN, Self::MAX),
                    expected: Some(format!("{}-{}", Self::MIN, Self::MAX)),
                    actual: Some(d.to_string()),
                });
            }
            Ok(Self(d))
        }

        pub fn get(&self) -> usize {
            self.0
        }
    }
}
```

### 6.2 Secret Management

```rust
// ============================================================
// Secure Secret Handling
// ============================================================

/// Wrapper for sensitive data that prevents accidental logging
#[derive(Clone)]
pub struct Secret<T> {
    inner: T,
}

impl<T> Secret<T> {
    pub fn new(value: T) -> Self {
        Self { inner: value }
    }

    /// Expose the secret value (use sparingly)
    pub fn expose(&self) -> &T {
        &self.inner
    }

    /// Consume and expose the secret
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> std::fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl<T> std::fmt::Display for Secret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[REDACTED]")
    }
}

// Zeroize on drop for sensitive data
impl<T: zeroize::Zeroize> Drop for Secret<T> {
    fn drop(&mut self) {
        self.inner.zeroize();
    }
}

/// Secure comparison for secrets
impl<T: Eq> Secret<T> {
    pub fn constant_time_eq(&self, other: &Secret<T>) -> bool
    where
        T: AsRef<[u8]>,
    {
        constant_time_eq::constant_time_eq(
            self.inner.as_ref(),
            other.inner.as_ref(),
        )
    }
}
```

### 6.3 Audit Logging

```rust
// ============================================================
// Security Audit Trail
// ============================================================

/// Security-relevant event types
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum SecurityEvent {
    AuthenticationAttempt {
        identity: String,
        method: AuthMethod,
        success: bool,
        ip_address: Option<IpAddr>,
        user_agent: Option<String>,
    },
    AuthorizationCheck {
        identity: String,
        resource: String,
        action: String,
        granted: bool,
    },
    SecretAccess {
        secret_id: String,
        accessor: String,
        operation: SecretOperation,
    },
    ConfigChange {
        key: String,
        changed_by: String,
        old_value_hash: Option<String>,  // Never log actual values
        new_value_hash: String,
    },
    RateLimitExceeded {
        identity: String,
        endpoint: String,
        limit: u32,
        window: Duration,
    },
    SuspiciousActivity {
        description: String,
        source_ip: Option<IpAddr>,
        indicators: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize)]
pub enum AuthMethod {
    ApiKey,
    Jwt,
    OAuth,
    Certificate,
}

#[derive(Debug, Clone, Serialize)]
pub enum SecretOperation {
    Read,
    Write,
    Delete,
    Rotate,
}

/// Security audit logger
pub struct SecurityAuditLog {
    sink: Box<dyn AuditSink>,
    buffer: Arc<Mutex<Vec<SecurityEvent>>>,
    flush_interval: Duration,
}

#[async_trait]
pub trait AuditSink: Send + Sync {
    async fn write(&self, events: Vec<SecurityEvent>) -> InfraResult<()>;
}

impl SecurityAuditLog {
    pub async fn log(&self, event: SecurityEvent) {
        // Always log synchronously for security events
        let event_json = serde_json::to_string(&event).unwrap();
        tracing::info!(target: "security_audit", event = %event_json);

        // Buffer for batch persistence
        self.buffer.lock().await.push(event);
    }

    pub async fn flush(&self) -> InfraResult<()> {
        let events: Vec<_> = self.buffer.lock().await.drain(..).collect();
        if !events.is_empty() {
            self.sink.write(events).await?;
        }
        Ok(())
    }
}
```

---

## 7. Testing Strategies

### 7.1 Testing Matrix by Crate

| Crate | Unit Tests | Integration | Property | Fuzz | Benchmark |
|-------|------------|-------------|----------|------|-----------|
| infra-errors | ✅ | ❌ | ✅ | ❌ | ❌ |
| infra-config | ✅ | ✅ | ✅ | ✅ | ❌ |
| infra-json | ✅ | ❌ | ✅ | ✅ | ✅ |
| infra-crypto | ✅ | ❌ | ✅ | ✅ | ✅ |
| infra-vector | ✅ | ✅ | ✅ | ❌ | ✅ |
| infra-http | ✅ | ✅ | ❌ | ❌ | ✅ |
| infra-otel | ✅ | ✅ | ❌ | ❌ | ❌ |
| infra-auth | ✅ | ✅ | ✅ | ✅ | ❌ |
| infra-mq | ✅ | ✅ | ❌ | ❌ | ✅ |
| infra-fs | ✅ | ✅ | ❌ | ❌ | ❌ |

### 7.2 Property-Based Testing

```rust
// ============================================================
// Property-Based Tests with Proptest
// ============================================================

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        /// JSON roundtrip preserves data
        #[test]
        fn json_roundtrip(value in any::<serde_json::Value>()) {
            let json = Json::from_value(&value).unwrap();
            let serialized = json.to_string();
            let deserialized = Json::parse(&serialized).unwrap();
            prop_assert_eq!(json, deserialized);
        }

        /// Vector normalization produces unit vectors
        #[test]
        fn normalization_produces_unit_vector(
            vec in prop::collection::vec(-1000.0f32..1000.0, 1..1000)
        ) {
            let mut v = vec.clone();
            EmbeddingNormalizer::normalize(&mut v);

            let magnitude: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();

            // Should be ~1.0 (allowing for float precision)
            prop_assert!((magnitude - 1.0).abs() < 1e-5 || vec.iter().all(|&x| x == 0.0));
        }

        /// Encryption is reversible
        #[test]
        fn encryption_roundtrip(data in prop::collection::vec(any::<u8>(), 0..10000)) {
            let cipher = Aes256GcmCipher::generate().unwrap();
            let encrypted = cipher.encrypt(&data).unwrap();
            let decrypted = cipher.decrypt(&encrypted).unwrap();
            prop_assert_eq!(data, decrypted);
        }

        /// Config merge is associative
        #[test]
        fn config_merge_associative(
            a in config_strategy(),
            b in config_strategy(),
            c in config_strategy(),
        ) {
            let ab_c = a.clone().merge(b.clone()).unwrap().merge(c.clone()).unwrap();
            let a_bc = a.merge(b.merge(c).unwrap()).unwrap();

            // Merge result should be equivalent
            prop_assert_eq!(ab_c.values, a_bc.values);
        }
    }
}
```

### 7.3 Fuzzing Setup

```rust
// ============================================================
// Fuzz Testing with cargo-fuzz
// ============================================================

// fuzz/fuzz_targets/json_parse.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use infra_json::Json;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Should never panic, only return errors
        let _ = Json::parse(s);
    }
});

// fuzz/fuzz_targets/config_parse.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use infra_config::Config;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Try parsing as TOML
        let _ = Config::from_toml(s);
        // Try parsing as JSON
        let _ = Config::from_json(s);
    }
});

// fuzz/fuzz_targets/crypto_decrypt.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use infra_crypto::Aes256GcmCipher;

fuzz_target!(|data: &[u8]| {
    // Decryption should never panic on malformed input
    let key = [0u8; 32];
    let cipher = Aes256GcmCipher::new(key);
    let _ = cipher.decrypt(data);
});
```

### 7.4 Integration Test Fixtures

```rust
// ============================================================
// Test Fixtures and Utilities
// ============================================================

/// Test harness for infra crates
pub struct TestHarness {
    config: Config,
    _otel_guard: Option<OtelGuard>,
    temp_dir: TempDir,
}

impl TestHarness {
    pub async fn new() -> InfraResult<Self> {
        let temp_dir = TempDir::new()?;

        let config = Config::builder()
            .set("test.mode", true)
            .set("vector.dimensions", 128)
            .set("http.timeout_secs", 5)
            .build()?;

        // Initialize OTEL with test exporter
        let otel_guard = if std::env::var("INFRA_TEST_OTEL").is_ok() {
            Some(init_otel(&OtelConfig {
                service_name: "infra-test".to_string(),
                exporter: ExporterConfig::Stdout { pretty: true },
                ..Default::default()
            }).await?)
        } else {
            None
        };

        Ok(Self {
            config,
            _otel_guard: otel_guard,
            temp_dir,
        })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn temp_path(&self, name: &str) -> PathBuf {
        self.temp_dir.path().join(name)
    }

    /// Create a mock HTTP server
    pub async fn mock_http(&self) -> MockHttpServer {
        MockHttpServer::start().await
    }

    /// Create a test vector store
    pub async fn vector_store(&self) -> InfraResult<impl VectorStore> {
        MockVectorStore::new(128)
    }
}

/// Macro for async test with harness
#[macro_export]
macro_rules! infra_test {
    ($name:ident, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            let harness = TestHarness::new().await.unwrap();
            $body(harness).await;
        }
    };
}

// Usage:
infra_test!(test_vector_search, |harness| async move {
    let store = harness.vector_store().await.unwrap();
    // ... test code
});
```

---

## 8. Migration Paths

### 8.1 Migration from Direct RuvNet Usage

```rust
// ============================================================
// Migration Guide: ruvector-core → infra-vector
// ============================================================

// BEFORE: Direct ruvector-core usage
use ruvector_core::{Collection, HnswConfig, Distance};

let collection = Collection::new(
    "embeddings",
    1536,
    Distance::Cosine,
    HnswConfig::default(),
).await?;

collection.insert("id-1", &embedding, None).await?;
let results = collection.search(&query, 10, None).await?;

// AFTER: infra-vector wrapper
use infra_vector::{VectorStore, RuVectorStore, VectorStoreConfig};

let config = VectorStoreConfig::builder()
    .collection("embeddings")
    .dimensions(1536)
    .distance(Distance::Cosine)
    .build()?;

let store = RuVectorStore::new(config).await?;

store.insert(
    VectorId::new("id-1"),
    embedding,
    Some(json!({"category": "tech"})),
).await?;

let results = store
    .query(query)
    .limit(10)
    .where_eq("category", "tech")
    .execute()
    .await?;
```

### 8.2 Migration Checklist

```markdown
## Migration Checklist: Adopting Infra Crates

### Phase 1: Error Handling
- [ ] Replace custom error types with InfraError
- [ ] Add infra-errors dependency
- [ ] Update error conversion implementations
- [ ] Add error context using infra_error! macro
- [ ] Implement retry logic using recovery strategies

### Phase 2: Configuration
- [ ] Replace env var reading with infra-config
- [ ] Create configuration schema
- [ ] Add config validation
- [ ] Set up config hot-reloading if needed
- [ ] Migrate secrets to Secret<T> wrapper

### Phase 3: Observability
- [ ] Add infra-otel dependency
- [ ] Initialize OTEL at application startup
- [ ] Add spans to key operations
- [ ] Add metrics for performance monitoring
- [ ] Configure exporters for your environment

### Phase 4: HTTP
- [ ] Replace reqwest/hyper with infra-http
- [ ] Update request building code
- [ ] Add timeout and retry configuration
- [ ] Enable request/response logging

### Phase 5: Vector Operations
- [ ] Replace direct ruvector usage with infra-vector
- [ ] Update insert/search operations
- [ ] Add metadata filtering
- [ ] Configure compression if needed

### Phase 6: Crypto
- [ ] Replace crypto libraries with infra-crypto
- [ ] Update hashing operations
- [ ] Migrate encryption to Aes256GcmCipher
- [ ] Update JWT handling

### Phase 7: Testing
- [ ] Update tests to use TestHarness
- [ ] Add property-based tests
- [ ] Add integration tests
- [ ] Enable fuzz testing for parsing code
```

### 8.3 Compatibility Shims

```rust
// ============================================================
// Backward Compatibility Layer
// ============================================================

/// Shim module for gradual migration
pub mod compat {
    use super::*;

    /// Converts old-style error to InfraError
    pub trait IntoInfraError {
        fn into_infra_error(self) -> InfraError;
    }

    // Implement for common external error types
    impl IntoInfraError for ruvector_core::Error {
        fn into_infra_error(self) -> InfraError {
            InfraError::Vector {
                operation: VectorOperation::Search,
                message: self.to_string(),
                dimensions: None,
            }
        }
    }

    impl IntoInfraError for reqwest::Error {
        fn into_infra_error(self) -> InfraError {
            InfraError::Http {
                status: self.status().map(|s| s.as_u16()),
                message: self.to_string(),
                url: self.url().map(|u| u.to_string()),
            }
        }
    }

    /// Extension trait for Result types
    pub trait ResultExt<T, E> {
        fn map_infra_err(self) -> InfraResult<T>
        where
            E: IntoInfraError;
    }

    impl<T, E: IntoInfraError> ResultExt<T, E> for Result<T, E> {
        fn map_infra_err(self) -> InfraResult<T> {
            self.map_err(|e| e.into_infra_error())
        }
    }
}

// Usage during migration:
use infra_errors::compat::ResultExt;

let result = old_function()
    .map_infra_err()?;  // Converts to InfraError
```

---

## 9. Troubleshooting Guide

### 9.1 Common Issues and Solutions

```markdown
## Troubleshooting Guide

### Error: "Configuration key not found"

**Symptoms:**
```
InfraError::Config { message: "Missing required field: vector.dimensions", ... }
```

**Causes:**
1. Configuration file not loaded
2. Environment variable not set
3. Key name mismatch

**Solutions:**
1. Verify config file path:
   ```rust
   let config = ConfigLoader::new()
       .with_file("config.toml")  // Check this path
       .load()?;
   ```
2. Check environment variable:
   ```bash
   export LLM_VECTOR_DIMENSIONS=1536
   ```
3. Use debug logging:
   ```rust
   tracing::debug!(?config, "Loaded configuration");
   ```

---

### Error: "Vector dimension mismatch"

**Symptoms:**
```
InfraError::Vector { operation: Insert, message: "Dimension mismatch: expected 1536, got 768", ... }
```

**Causes:**
1. Different embedding models
2. Incorrect store configuration
3. Data corruption

**Solutions:**
1. Verify embedding model dimensions:
   ```rust
   let embedding = model.embed(text)?;
   assert_eq!(embedding.len(), 1536, "Wrong dimensions");
   ```
2. Check store configuration:
   ```rust
   let stats = store.stats().await?;
   println!("Store dimensions: {}", stats.dimensions);
   ```

---

### Error: "Circuit breaker is open"

**Symptoms:**
```
InfraError::External { service: "circuit_breaker", message: "Circuit breaker is open", ... }
```

**Causes:**
1. Downstream service failures
2. Network issues
3. Rate limiting

**Solutions:**
1. Check downstream service health
2. Wait for circuit breaker timeout
3. Review failure logs:
   ```rust
   tracing::info!(target: "circuit_breaker", state = ?breaker.state());
   ```

---

### Error: "OTEL exporter connection failed"

**Symptoms:**
```
Failed to export spans: transport error
```

**Causes:**
1. OTEL collector not running
2. Wrong endpoint configuration
3. Network/firewall issues

**Solutions:**
1. Verify collector is running:
   ```bash
   curl http://localhost:4317/v1/traces
   ```
2. Check configuration:
   ```toml
   [otel.exporter.otlp]
   endpoint = "http://localhost:4317"
   ```
3. Use stdout exporter for debugging:
   ```rust
   ExporterConfig::Stdout { pretty: true }
   ```
```

### 9.2 Debug Mode

```rust
// ============================================================
// Debug Mode for Development
// ============================================================

/// Enable comprehensive debug output
pub fn enable_debug_mode() {
    // Set up verbose tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .init();

    // Enable error backtraces
    std::env::set_var("RUST_BACKTRACE", "1");

    tracing::info!("Debug mode enabled");
}

/// Debug wrapper that logs all operations
pub struct DebugWrapper<T> {
    inner: T,
    name: &'static str,
}

impl<T> DebugWrapper<T> {
    pub fn new(inner: T, name: &'static str) -> Self {
        Self { inner, name }
    }
}

#[async_trait]
impl<T: VectorStore> VectorStore for DebugWrapper<T> {
    async fn insert(&self, id: VectorId, vector: Vector, metadata: Option<Json>) -> InfraResult<()> {
        let span = tracing::debug_span!(
            "debug_insert",
            store = self.name,
            id = %id,
            vector_len = vector.len(),
            has_metadata = metadata.is_some(),
        );

        let _guard = span.enter();
        tracing::debug!("Starting insert");

        let start = Instant::now();
        let result = self.inner.insert(id, vector, metadata).await;
        let elapsed = start.elapsed();

        match &result {
            Ok(_) => tracing::debug!(?elapsed, "Insert succeeded"),
            Err(e) => tracing::warn!(?elapsed, error = %e, "Insert failed"),
        }

        result
    }

    // ... implement other methods similarly
}
```

---

## 10. Acceptance Criteria

### 10.1 Refinement Phase Completion

- [x] Error handling optimization with context and recovery
- [x] API ergonomics (builders, async extensions, type-safe config)
- [x] Edge cases catalog and graceful degradation
- [x] Performance optimization guidelines (memory, CPU, caching)
- [x] Security hardening (validation, secrets, audit logging)
- [x] Testing strategies per crate
- [x] Migration paths from RuvNet
- [x] Troubleshooting guide

### 10.2 Ready for Next Phase

Upon user approval, proceed to **Phase 5: Completion** which will:
- Provide implementation summary
- List all deliverables
- Define success metrics
- Outline next steps for implementation
- Create project handoff documentation

---

**Document Status:** Awaiting user approval to proceed to Completion phase.
