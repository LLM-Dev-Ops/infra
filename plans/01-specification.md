# SPARC Phase 1: Specification

## LLM-Dev-Ops Unified Infrastructure Layer

**Document Version:** 1.0
**Date:** 2025-12-06
**Status:** Draft - Pending User Approval

---

## 1. Executive Summary

This specification defines the unified INFRA layer for all 26 LLM-Dev-Ops repositories. The infrastructure wraps the verified RuvNet ecosystem (github.com/ruvnet) into consistent internal crates, providing a standardized foundation for LLM development operations.

### 1.1 Core Objectives

1. Create unified internal `infra-*` crates that wrap RuvNet ecosystem components
2. Establish one-directional compile-time dependencies (never circular)
3. Enforce a unified error model (`InfraError`)
4. Standardize OpenTelemetry 0.27 initialization across all crates
5. Implement unified configuration loading via `llm-config-manager`
6. Ensure WASM-compatible code paths for TypeScript SDK generation

---

## 2. RuvNet Ecosystem Scan Results

### 2.1 Verified RuvNet Repositories

Based on comprehensive GitHub scan of github.com/ruvnet:

| Repository | Status | Purpose | Key Capabilities |
|------------|--------|---------|------------------|
| **ruvector** | ✅ Verified | Distributed vector database with GNN learning | Vector search, HNSW indexing, Cypher queries, Raft consensus, WASM support |
| **ruv-FANN** | ✅ Verified | Neural network library + ruv-swarm orchestration | 27+ forecasting models, swarm intelligence, MCP integration |
| **claude-flow** | ✅ Verified | Agent orchestration platform | 100+ MCP tools, memory system, swarm coordination |
| **agentic-flow** | ✅ Verified | Multi-model agent framework | AgentDB, ReasoningBank, QUIC transport, multi-LLM routing |
| **ruvnet** | ✅ Verified | Core networking library | Base networking primitives |
| **ruv.io** | ✅ Verified | Unified npm package scope | Package distribution |

### 2.2 RuvNet Sub-Components (Within Verified Repos)

From **ruvector**:
- `ruvector-core` - Base vector search functionality
- `ruvector-graph` - Graph query engine (Cypher support)
- `ruvector-gnn` - Graph Neural Network layer
- `ruvector-attention` - 39 attention mechanisms
- `ruvector-sona` - Runtime adaptation (LoRA, EWC++)
- `ruvector-raft` - Consensus protocol
- `ruvector-cluster` - Clustering infrastructure
- `ruvector-replication` - Multi-master replication
- `ruvector-postgres` - pgvector-compatible extension
- `ruvector-gnn-wasm` - Browser/WASM support
- `ruvector-compression` - Tiered compression (f32→f16→PQ8→PQ4→Binary)

From **ruv-FANN**:
- `ruv-swarm` - Multi-agent swarm orchestration
- Neural inference engines (FastGRNN)

From **agentic-flow**:
- `agentdb` - Vector database with MCP integration
- `reasoningbank` - Persistent learning memory
- `agentic-flow-quic` - Rust QUIC transport crate

From **claude-flow**:
- Memory system (hybrid AgentDB + SQLite)
- 64 specialized agents
- 25 natural language skills
- Hooks system

### 2.3 Repositories Not Found (To Be Created as Infra Crates)

The following originally-specified repositories were **not found** in the RuvNet organization and will be implemented as new internal infra crates:

| Original Name | Status | Infra Crate Mapping |
|---------------|--------|---------------------|
| ruvhttp | ❌ Not Found | → `infra-http` (new implementation) |
| ruvjson | ❌ Not Found | → `infra-json` (new implementation) |
| ruvdsl | ❌ Not Found | → `infra-schema` (DSL for schemas) |
| ruvcrypto | ❌ Not Found | → `infra-crypto` (new implementation) |
| ruvauth | ❌ Not Found | → `infra-auth` (new implementation) |
| ruvfs | ❌ Not Found | → `infra-fs` (new implementation) |
| ruvotel | ❌ Not Found | → `infra-otel` (OpenTelemetry wrapper) |
| ruvid | ❌ Not Found | → `infra-id` (ID generation) |
| ruvsim | ❌ Not Found | → `infra-sim` (simulation/testing) |
| ruvmq | ❌ Not Found | → `infra-mq` (message queue) |
| ruvlang | ❌ Not Found | → `infra-schema` (language/DSL) |
| ruvaudit | ❌ Not Found | → `infra-audit` (audit logging) |

---

## 3. Internal Infra Crates Specification

### 3.1 Complete Infra Crate Catalog

| Crate Name | RuvNet Source | Purpose | WASM Compatible |
|------------|---------------|---------|-----------------|
| `infra-errors` | New | Unified `InfraError` type, error conversion traits | ✅ Yes |
| `infra-config` | New | Configuration loading via llm-config-manager | ✅ Yes |
| `infra-otel` | New | OpenTelemetry 0.27 initialization & spans | ❌ No (runtime) |
| `infra-http` | New | HTTP client/server primitives | ✅ Yes |
| `infra-vector` | ruvector-core | Vector operations, embeddings, HNSW | ✅ Yes |
| `infra-json` | New | JSON serialization/deserialization | ✅ Yes |
| `infra-schema` | New | Schema validation, DSL parsing | ✅ Yes |
| `infra-mq` | New | Message queue abstractions | ⚠️ Partial |
| `infra-router` | agentic-flow router | Request routing, load balancing | ✅ Yes |
| `infra-fs` | New | Filesystem operations abstraction | ❌ No (native) |
| `infra-audit` | New | Audit logging, compliance tracking | ✅ Yes |
| `infra-auth` | New | Authentication primitives | ✅ Yes |
| `infra-crypto` | New | Cryptographic operations | ✅ Yes |
| `infra-id` | New | UUID/ULID/Snowflake ID generation | ✅ Yes |
| `infra-sim` | New | Testing utilities, mocks, simulation | ✅ Yes |

### 3.2 Core Crate Specifications

#### 3.2.1 `infra-errors`

**Purpose:** Unified error model for all infra crates

```rust
// Unified error type
pub enum InfraError {
    Config(ConfigError),
    Http(HttpError),
    Vector(VectorError),
    Auth(AuthError),
    Crypto(CryptoError),
    Io(IoError),
    Serialization(SerdeError),
    Validation(ValidationError),
    External(ExternalError),
}

// Required traits
impl std::error::Error for InfraError {}
impl From<T> for InfraError where T: Into<InfraErrorKind> {}

// WASM-compatible error representation
#[cfg(target_arch = "wasm32")]
impl Into<JsValue> for InfraError {}
```

**Dependencies:** None (leaf crate)

#### 3.2.2 `infra-config`

**Purpose:** Unified configuration loading via llm-config-manager

```rust
pub trait ConfigSource {
    fn load(&self) -> Result<Config, InfraError>;
    fn watch(&self) -> impl Stream<Item = ConfigChange>;
}

pub struct Config {
    // Hierarchical configuration with environment overlay
}

// Integration with llm-config-manager
impl From<LlmConfigManager> for Config {}
```

**Dependencies:** `infra-errors`

#### 3.2.3 `infra-otel`

**Purpose:** Standardized OpenTelemetry 0.27 initialization

```rust
pub fn init_otel(config: &OtelConfig) -> Result<OtelGuard, InfraError> {
    // Initialize tracer provider
    // Initialize meter provider
    // Initialize logger provider
    // Return guard for cleanup
}

pub struct OtelConfig {
    pub service_name: String,
    pub service_version: String,
    pub exporter: ExporterConfig,
    pub sampling_ratio: f64,
}
```

**Dependencies:** `infra-errors`, `infra-config`

**Note:** Not WASM-compatible (requires runtime exporters)

#### 3.2.4 `infra-vector`

**Purpose:** Vector operations wrapping ruvector-core

```rust
// Wraps ruvector capabilities
pub use ruvector_core::{Vector, VectorIndex, HnswConfig};

pub trait VectorStore {
    fn insert(&mut self, id: &str, vector: Vector) -> Result<(), InfraError>;
    fn search(&self, query: Vector, k: usize) -> Result<Vec<SearchResult>, InfraError>;
    fn delete(&mut self, id: &str) -> Result<(), InfraError>;
}

// WASM bindings via ruvector-gnn-wasm
#[cfg(target_arch = "wasm32")]
pub mod wasm { /* ... */ }
```

**Dependencies:** `infra-errors`, `ruvector-core`, `ruvector-gnn-wasm`

#### 3.2.5 `infra-http`

**Purpose:** HTTP client/server primitives

```rust
pub trait HttpClient {
    async fn request(&self, req: Request) -> Result<Response, InfraError>;
}

pub trait HttpServer {
    fn route(&mut self, path: &str, handler: impl Handler);
    async fn serve(&self, addr: SocketAddr) -> Result<(), InfraError>;
}

// WASM: Uses fetch API
// Native: Uses hyper/reqwest
```

**Dependencies:** `infra-errors`, `infra-config`

#### 3.2.6 `infra-auth`

**Purpose:** Authentication primitives

```rust
pub trait Authenticator {
    fn authenticate(&self, credentials: Credentials) -> Result<Identity, InfraError>;
    fn authorize(&self, identity: &Identity, resource: &Resource) -> Result<bool, InfraError>;
}

pub struct JwtValidator { /* ... */ }
pub struct ApiKeyValidator { /* ... */ }
pub struct OAuthClient { /* ... */ }
```

**Dependencies:** `infra-errors`, `infra-crypto`, `infra-http`

#### 3.2.7 `infra-crypto`

**Purpose:** Cryptographic operations

```rust
pub trait Hasher {
    fn hash(&self, data: &[u8]) -> Vec<u8>;
    fn verify(&self, data: &[u8], hash: &[u8]) -> bool;
}

pub trait Signer {
    fn sign(&self, data: &[u8]) -> Result<Signature, InfraError>;
    fn verify(&self, data: &[u8], signature: &Signature) -> Result<bool, InfraError>;
}

pub trait Cipher {
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, InfraError>;
    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, InfraError>;
}
```

**Dependencies:** `infra-errors`

---

## 4. The 26 LLM-Dev-Ops Repositories

### 4.1 Repository Catalog

| # | Repository | Purpose | Category |
|---|------------|---------|----------|
| 1 | `llm-orchestrator` | Central workflow orchestration | Core |
| 2 | `llm-router` | Request routing & load balancing | Core |
| 3 | `llm-config-manager` | Configuration management | Core |
| 4 | `llm-prompt-engine` | Prompt templating & management | Core |
| 5 | `llm-vector-store` | Vector database operations | Data |
| 6 | `llm-model-registry` | Model versioning & registry | Data |
| 7 | `llm-api-gateway` | API gateway & rate limiting | Gateway |
| 8 | `llm-cache-layer` | Response & embedding caching | Performance |
| 9 | `llm-monitoring` | Metrics & observability | Operations |
| 10 | `llm-auth-service` | Authentication & authorization | Security |
| 11 | `llm-rate-limiter` | Rate limiting & quotas | Security |
| 12 | `llm-embeddings` | Embedding generation service | AI |
| 13 | `llm-fine-tuning` | Fine-tuning pipeline | AI |
| 14 | `llm-inference-server` | Model inference serving | AI |
| 15 | `llm-batch-processor` | Batch job processing | Processing |
| 16 | `llm-streaming-service` | Real-time streaming | Processing |
| 17 | `llm-context-manager` | Context window management | Processing |
| 18 | `llm-token-counter` | Token counting & estimation | Utilities |
| 19 | `llm-cost-tracker` | Cost estimation & tracking | Utilities |
| 20 | `llm-logging` | Centralized logging | Operations |
| 21 | `llm-testing-framework` | Testing utilities | Development |
| 22 | `llm-sdk-generator` | SDK generation (including TS) | Development |
| 23 | `llm-schema-validator` | Schema validation | Utilities |
| 24 | `llm-message-queue` | Async message handling | Messaging |
| 25 | `llm-state-manager` | Distributed state management | Data |
| 26 | `llm-deployment-tools` | Deployment automation | Operations |

---

## 5. Dependency Mapping Table

### 5.1 Infra Crate → LLM-Dev-Ops Repository Mapping

| Infra Crate | Required By (LLM-Dev-Ops Repos) |
|-------------|--------------------------------|
| `infra-errors` | ALL 26 repositories |
| `infra-config` | ALL 26 repositories |
| `infra-otel` | llm-orchestrator, llm-router, llm-api-gateway, llm-monitoring, llm-inference-server, llm-batch-processor, llm-streaming-service, llm-logging |
| `infra-http` | llm-orchestrator, llm-router, llm-api-gateway, llm-embeddings, llm-inference-server, llm-sdk-generator |
| `infra-vector` | llm-vector-store, llm-embeddings, llm-cache-layer, llm-context-manager |
| `infra-json` | ALL 26 repositories |
| `infra-schema` | llm-prompt-engine, llm-model-registry, llm-schema-validator, llm-sdk-generator |
| `infra-mq` | llm-orchestrator, llm-batch-processor, llm-streaming-service, llm-message-queue |
| `infra-router` | llm-router, llm-api-gateway, llm-rate-limiter |
| `infra-fs` | llm-config-manager, llm-model-registry, llm-fine-tuning, llm-logging |
| `infra-audit` | llm-auth-service, llm-api-gateway, llm-cost-tracker, llm-logging |
| `infra-auth` | llm-auth-service, llm-api-gateway, llm-rate-limiter |
| `infra-crypto` | llm-auth-service, llm-api-gateway, llm-embeddings |
| `infra-id` | llm-orchestrator, llm-batch-processor, llm-logging, llm-state-manager |
| `infra-sim` | llm-testing-framework, llm-sdk-generator |

### 5.2 Detailed Repository Dependencies

| Repository | Infra Crates Required |
|------------|----------------------|
| `llm-orchestrator` | infra-errors, infra-config, infra-otel, infra-http, infra-mq, infra-id |
| `llm-router` | infra-errors, infra-config, infra-otel, infra-http, infra-router |
| `llm-config-manager` | infra-errors, infra-config, infra-fs, infra-json |
| `llm-prompt-engine` | infra-errors, infra-config, infra-json, infra-schema |
| `llm-vector-store` | infra-errors, infra-config, infra-vector, infra-json |
| `llm-model-registry` | infra-errors, infra-config, infra-fs, infra-schema, infra-json |
| `llm-api-gateway` | infra-errors, infra-config, infra-otel, infra-http, infra-router, infra-auth, infra-crypto, infra-audit |
| `llm-cache-layer` | infra-errors, infra-config, infra-vector, infra-json |
| `llm-monitoring` | infra-errors, infra-config, infra-otel, infra-json |
| `llm-auth-service` | infra-errors, infra-config, infra-auth, infra-crypto, infra-audit, infra-json |
| `llm-rate-limiter` | infra-errors, infra-config, infra-router, infra-auth, infra-json |
| `llm-embeddings` | infra-errors, infra-config, infra-http, infra-vector, infra-crypto, infra-json |
| `llm-fine-tuning` | infra-errors, infra-config, infra-fs, infra-json |
| `llm-inference-server` | infra-errors, infra-config, infra-otel, infra-http, infra-json |
| `llm-batch-processor` | infra-errors, infra-config, infra-otel, infra-mq, infra-id, infra-json |
| `llm-streaming-service` | infra-errors, infra-config, infra-otel, infra-mq, infra-json |
| `llm-context-manager` | infra-errors, infra-config, infra-vector, infra-json |
| `llm-token-counter` | infra-errors, infra-config, infra-json |
| `llm-cost-tracker` | infra-errors, infra-config, infra-audit, infra-json |
| `llm-logging` | infra-errors, infra-config, infra-otel, infra-fs, infra-audit, infra-id, infra-json |
| `llm-testing-framework` | infra-errors, infra-config, infra-sim, infra-json |
| `llm-sdk-generator` | infra-errors, infra-config, infra-http, infra-schema, infra-sim, infra-json |
| `llm-schema-validator` | infra-errors, infra-config, infra-schema, infra-json |
| `llm-message-queue` | infra-errors, infra-config, infra-mq, infra-json |
| `llm-state-manager` | infra-errors, infra-config, infra-id, infra-json |
| `llm-deployment-tools` | infra-errors, infra-config, infra-json |

---

## 6. Dependency Rules & Constraints

### 6.1 Mandatory Constraints

1. **One-Directional Dependencies**
   - Infra crates are compile-time dependencies only
   - LLM-Dev-Ops repos consume infra crates, never the reverse
   - No infra crate may depend on any LLM-Dev-Ops repo

2. **No Circular Dependencies**
   - Strictly enforced via Cargo workspace configuration
   - CI/CD will fail builds with circular dependency detection

3. **Dependency Direction**
   ```
   LLM-Dev-Ops Repos → Infra Crates → RuvNet Ecosystem
   (consumers)         (wrappers)     (upstream)
   ```

### 6.2 Infra Crate Dependency Graph

```
Level 0 (Leaf - No Dependencies):
├── infra-errors

Level 1 (Depends on Level 0):
├── infra-config (→ infra-errors)
├── infra-json (→ infra-errors)
├── infra-crypto (→ infra-errors)
├── infra-id (→ infra-errors)

Level 2 (Depends on Level 0-1):
├── infra-otel (→ infra-errors, infra-config)
├── infra-http (→ infra-errors, infra-config)
├── infra-fs (→ infra-errors, infra-config)
├── infra-schema (→ infra-errors, infra-json)
├── infra-audit (→ infra-errors, infra-config, infra-json)
├── infra-sim (→ infra-errors, infra-config)
├── infra-mq (→ infra-errors, infra-config, infra-json)

Level 3 (Depends on Level 0-2):
├── infra-vector (→ infra-errors, infra-config, ruvector-core)
├── infra-router (→ infra-errors, infra-config, infra-http)
├── infra-auth (→ infra-errors, infra-crypto, infra-http)
```

---

## 7. Cargo.toml Structure Standards

### 7.1 Standard Infra Crate Cargo.toml

```toml
[package]
name = "infra-{name}"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
license = "MIT OR Apache-2.0"
repository = "https://github.com/llm-dev-ops/infra"
description = "LLM-Dev-Ops Infrastructure: {description}"
keywords = ["llm", "infrastructure", "devops"]
categories = ["development-tools"]

[features]
default = ["std"]
std = []
wasm = ["wasm-bindgen"]

[dependencies]
infra-errors = { path = "../infra-errors" }
# Additional infra deps as needed

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = { version = "0.2", optional = true }
js-sys = { version = "0.3", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

### 7.2 Workspace Cargo.toml

```toml
[workspace]
resolver = "2"
members = [
    "crates/infra-errors",
    "crates/infra-config",
    "crates/infra-otel",
    "crates/infra-http",
    "crates/infra-vector",
    "crates/infra-json",
    "crates/infra-schema",
    "crates/infra-mq",
    "crates/infra-router",
    "crates/infra-fs",
    "crates/infra-audit",
    "crates/infra-auth",
    "crates/infra-crypto",
    "crates/infra-id",
    "crates/infra-sim",
]

[workspace.package]
edition = "2021"
rust-version = "1.75"
license = "MIT OR Apache-2.0"
repository = "https://github.com/llm-dev-ops/infra"

[workspace.dependencies]
# Shared dependency versions
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
tracing = "0.1"
opentelemetry = "0.27"
opentelemetry_sdk = "0.27"
```

---

## 8. WASM Compatibility Requirements

### 8.1 TypeScript SDK Structure

```
/sdk/ts/
├── package.json
├── tsconfig.json
├── src/
│   ├── index.ts
│   ├── errors.ts          # Generated from infra-errors
│   ├── config.ts          # Generated from infra-config
│   ├── vector.ts          # Generated from infra-vector
│   ├── json.ts            # Generated from infra-json
│   ├── schema.ts          # Generated from infra-schema
│   ├── auth.ts            # Generated from infra-auth
│   ├── crypto.ts          # Generated from infra-crypto
│   ├── id.ts              # Generated from infra-id
│   └── types/
│       └── index.d.ts
└── wasm/
    ├── infra_errors.wasm
    ├── infra_vector.wasm
    └── ...
```

### 8.2 WASM Build Configuration

Each WASM-compatible crate must include:

```toml
[lib]
crate-type = ["cdylib", "rlib"]

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
web-sys = { version = "0.3", features = ["console"] }
```

---

## 9. OpenTelemetry 0.27 Integration

### 9.1 Standard Initialization Pattern

```rust
use infra_otel::{init_otel, OtelConfig, OtelGuard};

pub async fn setup_observability() -> Result<OtelGuard, InfraError> {
    let config = OtelConfig::from_env()?;
    init_otel(&config).await
}
```

### 9.2 Required Exports

All observable crates must export:
- `tracing::Span` compatible spans
- Metrics via `opentelemetry::metrics`
- Structured logs via `tracing` with OTEL export

---

## 10. Unified Error Model

### 10.1 InfraError Design

```rust
#[derive(Debug, thiserror::Error)]
pub enum InfraError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("HTTP error: {0}")]
    Http(#[from] HttpError),

    #[error("Vector operation error: {0}")]
    Vector(#[from] VectorError),

    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),

    #[error("Cryptographic error: {0}")]
    Crypto(#[from] CryptoError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Validation error: {message}")]
    Validation { message: String, field: Option<String> },

    #[error("External service error: {service}: {message}")]
    External { service: String, message: String },
}

// OpenTelemetry integration
impl InfraError {
    pub fn record_span(&self, span: &tracing::Span) {
        span.record("error.type", self.error_type());
        span.record("error.message", self.to_string());
    }

    pub fn error_type(&self) -> &'static str {
        match self {
            Self::Config(_) => "config",
            Self::Http(_) => "http",
            Self::Vector(_) => "vector",
            // ...
        }
    }
}
```

---

## 11. Configuration via llm-config-manager

### 11.1 Integration Pattern

```rust
use llm_config_manager::ConfigManager;
use infra_config::{Config, ConfigSource};

impl ConfigSource for ConfigManager {
    fn load(&self) -> Result<Config, InfraError> {
        // Load from llm-config-manager
        // Apply environment overlays
        // Validate configuration
    }
}
```

### 11.2 Configuration Hierarchy

1. Default values (compiled in)
2. Configuration files (TOML/YAML)
3. Environment variables
4. Runtime overrides

---

## 12. Acceptance Criteria

### 12.1 Specification Phase Completion

- [x] RuvNet ecosystem scanned and documented
- [x] All 15 infra crates defined with purposes
- [x] All 26 LLM-Dev-Ops repositories cataloged
- [x] Complete dependency mapping table created
- [x] One-directional dependency rules defined
- [x] Cargo.toml structure standardized
- [x] WASM compatibility requirements specified
- [x] OpenTelemetry 0.27 integration defined
- [x] Unified error model (InfraError) specified
- [x] llm-config-manager integration specified

### 12.2 Ready for Next Phase

Upon user approval, proceed to **Phase 2: Pseudocode** which will define:
- Detailed implementation pseudocode for each infra crate
- API surface definitions
- Error handling flows
- WASM binding generation strategy

---

## Appendix A: RuvNet Ecosystem Reference

| Source Repository | Documentation |
|-------------------|---------------|
| [ruvector](https://github.com/ruvnet/ruvector) | Distributed vector database with GNN learning |
| [ruv-FANN](https://github.com/ruvnet/ruv-FANN) | Neural network library + ruv-swarm |
| [claude-flow](https://github.com/ruvnet/claude-flow) | Agent orchestration platform |
| [agentic-flow](https://github.com/ruvnet/agentic-flow) | Multi-model agent framework |

## Appendix B: Glossary

| Term | Definition |
|------|------------|
| **InfraError** | Unified error type used across all infra crates |
| **HNSW** | Hierarchical Navigable Small World - graph-based ANN algorithm |
| **GNN** | Graph Neural Network |
| **WASM** | WebAssembly |
| **OTEL** | OpenTelemetry |
| **MCP** | Model Context Protocol |

---

**Document Status:** Awaiting user approval to proceed to Pseudocode phase.
