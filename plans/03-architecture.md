# SPARC Phase 3: Architecture

## LLM-Dev-Ops Unified Infrastructure Layer

**Document Version:** 1.0
**Date:** 2025-12-06
**Status:** Draft - Pending User Approval
**Previous Phase:** [02-pseudocode.md](./02-pseudocode.md)

---

## 1. Overview

This document defines the complete architecture for the LLM-Dev-Ops infrastructure layer, including:
- Repository and directory structure
- Module organization for all 15 crates
- Dependency graph with strict layering
- Cargo workspace configuration
- CI/CD pipeline design
- WASM build system
- TypeScript SDK architecture
- Release and versioning strategy

---

## 2. Repository Structure

### 2.1 Top-Level Directory Layout

```
llm-dev-ops-infra/
├── .github/
│   ├── workflows/
│   │   ├── ci.yml                    # Main CI pipeline
│   │   ├── release.yml               # Release automation
│   │   ├── wasm-build.yml            # WASM-specific builds
│   │   └── security-audit.yml        # Security scanning
│   ├── CODEOWNERS
│   └── dependabot.yml
│
├── crates/                           # All Rust crates
│   ├── infra-errors/
│   ├── infra-config/
│   ├── infra-otel/
│   ├── infra-json/
│   ├── infra-http/
│   ├── infra-vector/
│   ├── infra-crypto/
│   ├── infra-auth/
│   ├── infra-id/
│   ├── infra-fs/
│   ├── infra-mq/
│   ├── infra-router/
│   ├── infra-schema/
│   ├── infra-audit/
│   └── infra-sim/
│
├── sdk/
│   └── ts/                           # TypeScript SDK
│       ├── packages/
│       │   ├── infra-core/           # Core types and errors
│       │   ├── infra-vector/         # Vector operations
│       │   ├── infra-crypto/         # Crypto utilities
│       │   └── infra-client/         # Full client bundle
│       ├── wasm/                     # Compiled WASM modules
│       ├── scripts/
│       ├── package.json
│       ├── pnpm-workspace.yaml
│       └── tsconfig.json
│
├── examples/
│   ├── rust/
│   │   ├── basic-usage/
│   │   ├── vector-search/
│   │   └── full-stack/
│   └── typescript/
│       ├── browser/
│       └── node/
│
├── benches/                          # Performance benchmarks
│   ├── vector_ops.rs
│   ├── crypto_ops.rs
│   └── http_throughput.rs
│
├── tests/
│   └── integration/                  # Cross-crate integration tests
│       ├── full_pipeline_test.rs
│       └── wasm_interop_test.rs
│
├── tools/
│   ├── wasm-builder/                 # WASM build tooling
│   ├── sdk-generator/                # SDK generation scripts
│   └── dependency-checker/           # Circular dependency detection
│
├── docs/
│   ├── architecture/
│   ├── api/
│   └── guides/
│
├── plans/                            # SPARC planning documents
│   ├── 01-specification.md
│   ├── 02-pseudocode.md
│   ├── 03-architecture.md            # This document
│   ├── 04-refinement.md
│   └── 05-completion.md
│
├── Cargo.toml                        # Workspace root
├── Cargo.lock
├── rust-toolchain.toml
├── deny.toml                         # cargo-deny configuration
├── clippy.toml
├── rustfmt.toml
├── LICENSE-MIT
├── LICENSE-APACHE
└── README.md
```

---

## 3. Crate Architecture

### 3.1 Individual Crate Structure

Each infra crate follows a consistent structure:

```
crates/infra-{name}/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs                        # Public API exports
│   ├── error.rs                      # Crate-specific error types (if needed)
│   ├── types.rs                      # Core type definitions
│   ├── traits.rs                     # Trait definitions
│   ├── impl/                         # Implementation modules
│   │   ├── mod.rs
│   │   └── ...
│   ├── wasm/                         # WASM-specific code (if applicable)
│   │   ├── mod.rs
│   │   └── bindings.rs
│   └── testing/                      # Test utilities (pub for other crates)
│       └── mod.rs
├── tests/
│   ├── unit/
│   │   └── ...
│   └── integration/
│       └── ...
└── benches/
    └── ...
```

### 3.2 Detailed Crate Architectures

#### 3.2.1 `infra-errors`

```
crates/infra-errors/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   │   └── // Re-exports: InfraError, InfraResult, error kinds
│   │
│   ├── error.rs
│   │   ├── pub enum InfraError { ... }
│   │   ├── impl std::error::Error
│   │   ├── impl std::fmt::Display
│   │   └── impl From<T> for various external errors
│   │
│   ├── kinds.rs
│   │   ├── pub enum VectorOperation
│   │   ├── pub enum AuthErrorKind
│   │   ├── pub enum CryptoOperation
│   │   ├── pub enum IoOperation
│   │   ├── pub enum SerializationFormat
│   │   └── pub enum MqOperation
│   │
│   ├── builder.rs
│   │   └── pub struct ErrorBuilder { ... }
│   │
│   ├── otel.rs
│   │   ├── impl InfraError { fn record_to_span() }
│   │   └── impl InfraError { fn error_type() }
│   │
│   ├── retry.rs
│   │   ├── impl InfraError { fn is_retryable() }
│   │   ├── impl InfraError { fn retry_after() }
│   │   └── pub struct RetryConfig
│   │
│   ├── wasm/
│   │   ├── mod.rs
│   │   ├── js_error.rs
│   │   │   └── pub struct JsInfraError
│   │   └── conversions.rs
│   │       └── impl Into<JsValue> for InfraError
│   │
│   └── testing/
│       └── mod.rs
│           ├── pub fn mock_config_error()
│           ├── pub fn mock_http_error()
│           └── pub fn mock_vector_error()
│
└── tests/
    ├── error_conversion_tests.rs
    └── retry_tests.rs
```

#### 3.2.2 `infra-config`

```
crates/infra-config/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   │   └── // Re-exports: Config, ConfigLoader, ConfigValue
│   │
│   ├── config.rs
│   │   ├── pub struct Config
│   │   ├── impl Config { get, get_or, require, set, merge }
│   │   └── pub enum ConfigValue
│   │
│   ├── loader.rs
│   │   ├── pub struct ConfigLoader
│   │   ├── impl ConfigLoader { new, with_file, with_env_prefix, load }
│   │   └── trait ConfigSourceProvider
│   │
│   ├── sources/
│   │   ├── mod.rs
│   │   ├── file.rs
│   │   │   └── pub struct FileSource
│   │   ├── env.rs
│   │   │   └── pub struct EnvSource
│   │   ├── llm_config_manager.rs
│   │   │   └── pub struct LlmConfigManagerSource
│   │   └── memory.rs
│   │       └── pub struct MemorySource
│   │
│   ├── watcher.rs
│   │   ├── pub struct ConfigWatcher
│   │   └── pub enum ConfigChange
│   │
│   ├── validation.rs
│   │   └── fn validate_config()
│   │
│   ├── conversion.rs
│   │   └── trait FromConfigValue
│   │
│   ├── wasm/
│   │   ├── mod.rs
│   │   └── js_config.rs
│   │       └── pub struct JsConfig
│   │
│   └── testing/
│       └── mod.rs
│           └── pub fn test_config()
│
└── tests/
    ├── loader_tests.rs
    ├── merge_tests.rs
    └── watcher_tests.rs
```

#### 3.2.3 `infra-otel`

```
crates/infra-otel/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   │   └── // Re-exports: init_otel, OtelConfig, OtelGuard
│   │
│   ├── config.rs
│   │   ├── pub struct OtelConfig
│   │   ├── pub enum ExporterConfig
│   │   ├── pub struct SamplingConfig
│   │   └── impl OtelConfig { from_config }
│   │
│   ├── init.rs
│   │   ├── pub async fn init_otel()
│   │   ├── fn init_tracer()
│   │   ├── fn init_meter()
│   │   └── fn init_logger()
│   │
│   ├── guard.rs
│   │   ├── pub struct OtelGuard
│   │   └── impl Drop for OtelGuard
│   │
│   ├── resource.rs
│   │   └── fn build_resource()
│   │
│   ├── sampling.rs
│   │   └── fn create_sampler()
│   │
│   ├── exporters/
│   │   ├── mod.rs
│   │   ├── otlp.rs
│   │   ├── jaeger.rs
│   │   └── stdout.rs
│   │
│   ├── instrumentation/
│   │   ├── mod.rs
│   │   ├── http.rs
│   │   │   └── pub struct InstrumentedHttpClient
│   │   └── llm.rs
│   │       ├── pub fn llm_span()
│   │       └── pub fn record_llm_metrics()
│   │
│   ├── propagation.rs
│   │   ├── pub struct PropagationContext
│   │   └── impl PropagationContext { from_headers, inject_into_headers }
│   │
│   └── testing/
│       └── mod.rs
│           └── pub struct MockTracer
│
└── tests/
    ├── init_tests.rs
    └── propagation_tests.rs
```

#### 3.2.4 `infra-vector`

```
crates/infra-vector/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   │   └── // Re-exports: VectorStore, Vector, SearchResult
│   │
│   ├── traits.rs
│   │   └── pub trait VectorStore
│   │
│   ├── types.rs
│   │   ├── pub struct VectorRecord
│   │   ├── pub struct SearchResult
│   │   ├── pub struct BatchInsertResult
│   │   └── pub struct CollectionStats
│   │
│   ├── filter.rs
│   │   ├── pub enum MetadataFilter
│   │   └── impl MetadataFilter { to_ruvector_filter }
│   │
│   ├── impl/
│   │   ├── mod.rs
│   │   ├── ruvector.rs
│   │   │   ├── pub struct RuVectorStore
│   │   │   └── impl VectorStore for RuVectorStore
│   │   └── config.rs
│   │       ├── pub struct VectorStoreConfig
│   │       └── pub struct CompressionConfig
│   │
│   ├── embedding.rs
│   │   └── pub struct EmbeddingNormalizer
│   │
│   ├── wasm/
│   │   ├── mod.rs
│   │   └── js_vector_store.rs
│   │       └── pub struct JsVectorStore
│   │
│   └── testing/
│       └── mod.rs
│           └── pub struct MockVectorStore
│
└── tests/
    ├── search_tests.rs
    ├── batch_tests.rs
    └── filter_tests.rs
```

#### 3.2.5 `infra-http`

```
crates/infra-http/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   │
│   ├── client/
│   │   ├── mod.rs
│   │   ├── traits.rs
│   │   │   └── pub trait HttpClient
│   │   ├── request.rs
│   │   │   ├── pub struct Request
│   │   │   └── pub struct RequestBuilder
│   │   ├── response.rs
│   │   │   └── pub struct Response
│   │   ├── native.rs              // #[cfg(not(target_arch = "wasm32"))]
│   │   │   └── pub struct NativeHttpClient
│   │   └── wasm.rs                // #[cfg(target_arch = "wasm32")]
│   │       └── pub struct WasmHttpClient
│   │
│   ├── server/                    // #[cfg(not(target_arch = "wasm32"))]
│   │   ├── mod.rs
│   │   ├── server.rs
│   │   │   └── pub struct HttpServer
│   │   ├── router.rs
│   │   │   └── pub struct Router
│   │   └── handler.rs
│   │       └── pub trait Handler
│   │
│   ├── types.rs
│   │   ├── pub enum Method
│   │   ├── pub struct HeaderMap
│   │   ├── pub enum Body
│   │   └── pub struct StatusCode
│   │
│   ├── config.rs
│   │   ├── pub struct HttpClientConfig
│   │   └── pub struct HttpServerConfig
│   │
│   └── testing/
│       └── mod.rs
│           └── pub struct MockHttpClient
│
└── tests/
    ├── client_tests.rs
    └── server_tests.rs
```

#### 3.2.6 `infra-crypto`

```
crates/infra-crypto/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   │
│   ├── hash/
│   │   ├── mod.rs
│   │   ├── traits.rs
│   │   │   └── pub trait Hasher
│   │   ├── sha256.rs
│   │   │   └── pub struct Sha256Hasher
│   │   ├── blake3.rs
│   │   │   └── pub struct Blake3Hasher
│   │   └── password.rs
│   │       ├── pub struct PasswordHasher
│   │       └── pub enum PasswordAlgorithm
│   │
│   ├── cipher/
│   │   ├── mod.rs
│   │   ├── traits.rs
│   │   │   └── pub trait Cipher
│   │   └── aes_gcm.rs
│   │       └── pub struct Aes256GcmCipher
│   │
│   ├── sign/
│   │   ├── mod.rs
│   │   ├── traits.rs
│   │   │   ├── pub trait Signer
│   │   │   └── pub trait Verifier
│   │   ├── ed25519.rs
│   │   │   ├── pub struct Ed25519Signer
│   │   │   └── pub struct Ed25519Verifier
│   │   └── types.rs
│   │       ├── pub struct Signature
│   │       └── pub struct PublicKey
│   │
│   ├── jwt/
│   │   ├── mod.rs
│   │   ├── signer.rs
│   │   │   └── pub struct JwtSigner
│   │   └── claims.rs
│   │       └── pub struct TokenClaims
│   │
│   ├── wasm/
│   │   ├── mod.rs
│   │   ├── hash.rs
│   │   └── cipher.rs
│   │
│   └── testing/
│       └── mod.rs
│
└── tests/
    ├── hash_tests.rs
    ├── cipher_tests.rs
    ├── sign_tests.rs
    └── jwt_tests.rs
```

---

## 4. Dependency Graph

### 4.1 Strict Layering Model

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        LAYER 3 (Application Layer)                       │
│                                                                          │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                  │
│   │ infra-vector │  │  infra-auth  │  │ infra-router │                  │
│   │  (ruvector)  │  │              │  │              │                  │
│   └──────┬───────┘  └──────┬───────┘  └──────┬───────┘                  │
│          │                 │                 │                           │
├──────────┼─────────────────┼─────────────────┼───────────────────────────┤
│          │      LAYER 2 (Service Layer)      │                           │
│          │                 │                 │                           │
│   ┌──────┴───────┐  ┌──────┴───────┐  ┌──────┴───────┐  ┌─────────────┐ │
│   │  infra-otel  │  │  infra-http  │  │  infra-fs    │  │ infra-schema│ │
│   └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └──────┬──────┘ │
│          │                 │                 │                 │         │
│   ┌──────┴───────┐  ┌──────┴───────┐  ┌──────┴───────┐  ┌──────┴──────┐ │
│   │  infra-mq    │  │ infra-audit  │  │  infra-sim   │  │             │ │
│   └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  │             │ │
│          │                 │                 │          │             │ │
├──────────┼─────────────────┼─────────────────┼──────────┼─────────────┼─┤
│          │      LAYER 1 (Foundation Layer)   │          │             │ │
│          │                 │                 │          │             │ │
│   ┌──────┴───────┐  ┌──────┴───────┐  ┌──────┴───────┐  │             │ │
│   │ infra-config │  │  infra-json  │  │ infra-crypto │  │             │ │
│   └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  │             │ │
│          │                 │                 │          │             │ │
│   ┌──────┴───────┐                    ┌──────┴───────┐  │             │ │
│   │   infra-id   │                    │              │  │             │ │
│   └──────┬───────┘                    │              │  │             │ │
│          │                            │              │  │             │ │
├──────────┼────────────────────────────┼──────────────┼──┼─────────────┼─┤
│          │      LAYER 0 (Core Layer)  │              │  │             │ │
│          │                            │              │  │             │ │
│          │    ┌───────────────────────┴──────────────┴──┴─────────────┘ │
│          │    │                                                          │
│          └────┼───────────────────────────────────────────────────────┐ │
│               │                                                        │ │
│               ▼                                                        │ │
│        ┌─────────────────┐                                             │ │
│        │  infra-errors   │  ◄── ALL CRATES DEPEND ON THIS              │ │
│        │   (Leaf Crate)  │                                             │ │
│        └─────────────────┘                                             │ │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 4.2 Detailed Dependency Matrix

| Crate | Layer | Dependencies |
|-------|-------|--------------|
| `infra-errors` | 0 | *none* |
| `infra-config` | 1 | infra-errors |
| `infra-json` | 1 | infra-errors |
| `infra-crypto` | 1 | infra-errors |
| `infra-id` | 1 | infra-errors |
| `infra-otel` | 2 | infra-errors, infra-config |
| `infra-http` | 2 | infra-errors, infra-config |
| `infra-fs` | 2 | infra-errors, infra-config |
| `infra-schema` | 2 | infra-errors, infra-json |
| `infra-mq` | 2 | infra-errors, infra-config, infra-json |
| `infra-audit` | 2 | infra-errors, infra-config, infra-json |
| `infra-sim` | 2 | infra-errors, infra-config |
| `infra-vector` | 3 | infra-errors, infra-config, (ruvector-core) |
| `infra-auth` | 3 | infra-errors, infra-crypto, infra-http |
| `infra-router` | 3 | infra-errors, infra-config, infra-http |

### 4.3 External Dependencies (RuvNet Ecosystem)

```
┌─────────────────────────────────────────────────────────────┐
│                    INFRA LAYER                               │
│                                                              │
│   infra-vector ────────────────┐                            │
│                                │                             │
└────────────────────────────────┼─────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────┐
│                   RUVNET ECOSYSTEM                           │
│                                                              │
│   ┌─────────────────┐    ┌─────────────────┐                │
│   │  ruvector-core  │    │ ruvector-gnn-   │                │
│   │                 │    │     wasm        │                │
│   │  • Vector ops   │    │                 │                │
│   │  • HNSW index   │    │  • Browser API  │                │
│   │  • Cypher       │    │  • WASM build   │                │
│   └─────────────────┘    └─────────────────┘                │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## 5. Cargo Workspace Configuration

### 5.1 Root `Cargo.toml`

```toml
[workspace]
resolver = "2"
members = [
    "crates/infra-errors",
    "crates/infra-config",
    "crates/infra-otel",
    "crates/infra-json",
    "crates/infra-http",
    "crates/infra-vector",
    "crates/infra-crypto",
    "crates/infra-auth",
    "crates/infra-id",
    "crates/infra-fs",
    "crates/infra-mq",
    "crates/infra-router",
    "crates/infra-schema",
    "crates/infra-audit",
    "crates/infra-sim",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
license = "MIT OR Apache-2.0"
repository = "https://github.com/llm-dev-ops/infra"
authors = ["LLM-Dev-Ops Team"]
keywords = ["llm", "infrastructure", "devops", "ai"]
categories = ["development-tools", "web-programming"]

[workspace.dependencies]
# Internal crates
infra-errors = { path = "crates/infra-errors", version = "0.1.0" }
infra-config = { path = "crates/infra-config", version = "0.1.0" }
infra-otel = { path = "crates/infra-otel", version = "0.1.0" }
infra-json = { path = "crates/infra-json", version = "0.1.0" }
infra-http = { path = "crates/infra-http", version = "0.1.0" }
infra-vector = { path = "crates/infra-vector", version = "0.1.0" }
infra-crypto = { path = "crates/infra-crypto", version = "0.1.0" }
infra-auth = { path = "crates/infra-auth", version = "0.1.0" }
infra-id = { path = "crates/infra-id", version = "0.1.0" }
infra-fs = { path = "crates/infra-fs", version = "0.1.0" }
infra-mq = { path = "crates/infra-mq", version = "0.1.0" }
infra-router = { path = "crates/infra-router", version = "0.1.0" }
infra-schema = { path = "crates/infra-schema", version = "0.1.0" }
infra-audit = { path = "crates/infra-audit", version = "0.1.0" }
infra-sim = { path = "crates/infra-sim", version = "0.1.0" }

# RuvNet ecosystem
ruvector-core = { git = "https://github.com/ruvnet/ruvector", branch = "main" }
ruvector-gnn-wasm = { git = "https://github.com/ruvnet/ruvector", branch = "main" }

# Async runtime
tokio = { version = "1.35", features = ["full"] }
async-trait = "0.1"
futures = "0.3"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# OpenTelemetry (0.27)
opentelemetry = "0.27"
opentelemetry_sdk = { version = "0.27", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.27", features = ["grpc-tonic"] }
opentelemetry-jaeger = "0.22"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-opentelemetry = "0.27"

# HTTP
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
hyper = { version = "1.0", features = ["full"] }
axum = "0.7"
tower = "0.4"

# Crypto
sha2 = "0.10"
blake3 = "1.5"
aes-gcm = "0.10"
argon2 = "0.5"
bcrypt = "0.15"
ed25519-dalek = "2.1"
jsonwebtoken = "9.2"
rand = "0.8"

# Utilities
uuid = { version = "1.6", features = ["v4", "v7"] }
ulid = "1.1"
chrono = { version = "0.4", features = ["serde"] }
hex = "0.4"
base64 = "0.21"
url = "2.5"

# WASM
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
web-sys = { version = "0.3", features = [
    "console",
    "Window",
    "Request",
    "RequestInit",
    "Response",
    "Headers",
] }
serde-wasm-bindgen = "0.6"

# Testing
mockall = "0.12"
wiremock = "0.6"
tokio-test = "0.4"
proptest = "1.4"
criterion = "0.5"

[workspace.lints.rust]
unsafe_code = "deny"
missing_docs = "warn"

[workspace.lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"

[profile.release]
lto = true
codegen-units = 1
panic = "abort"
strip = true

[profile.release-wasm]
inherits = "release"
opt-level = "z"  # Optimize for size

[profile.bench]
debug = true
```

### 5.2 Example Crate `Cargo.toml` (`infra-errors`)

```toml
[package]
name = "infra-errors"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true
description = "Unified error handling for LLM-Dev-Ops infrastructure"
keywords = ["errors", "llm", "infrastructure"]
categories = ["development-tools"]
readme = "README.md"

[features]
default = ["std"]
std = []
wasm = ["wasm-bindgen", "js-sys", "serde-wasm-bindgen"]
otel = ["tracing"]

[dependencies]
thiserror = { workspace = true }
serde = { workspace = true }

# Optional WASM support
wasm-bindgen = { workspace = true, optional = true }
js-sys = { workspace = true, optional = true }
serde-wasm-bindgen = { workspace = true, optional = true }

# Optional OTEL integration
tracing = { workspace = true, optional = true }

[dev-dependencies]
tokio = { workspace = true, features = ["rt-multi-thread", "macros"] }
serde_json = { workspace = true }

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = { workspace = true }
js-sys = { workspace = true }

[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]

[lints]
workspace = true
```

### 5.3 Example Crate `Cargo.toml` (`infra-vector`)

```toml
[package]
name = "infra-vector"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true
description = "Vector database operations for LLM-Dev-Ops infrastructure"
keywords = ["vector", "embeddings", "llm", "search"]
categories = ["database"]
readme = "README.md"

[features]
default = ["std", "ruvector"]
std = []
ruvector = ["ruvector-core"]
wasm = ["ruvector-gnn-wasm", "wasm-bindgen", "js-sys"]

[dependencies]
infra-errors = { workspace = true }
infra-config = { workspace = true }

# Core dependencies
async-trait = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
chrono = { workspace = true }

# RuVector (optional, enabled by default)
ruvector-core = { workspace = true, optional = true }

# WASM support
ruvector-gnn-wasm = { workspace = true, optional = true }
wasm-bindgen = { workspace = true, optional = true }
js-sys = { workspace = true, optional = true }

[dev-dependencies]
tokio = { workspace = true, features = ["rt-multi-thread", "macros"] }
infra-sim = { workspace = true }

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
tokio = { workspace = true }

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen-futures = { workspace = true }

[lints]
workspace = true
```

---

## 6. CI/CD Pipeline Architecture

### 6.1 Main CI Workflow (`.github/workflows/ci.yml`)

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
  RUSTFLAGS: "-D warnings"

jobs:
  # ============================================
  # Stage 1: Quick Checks (Parallel)
  # ============================================

  fmt:
    name: Format Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check

  clippy:
    name: Clippy Lints
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --all-targets --all-features -- -D warnings

  deny:
    name: Cargo Deny
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: EmbarkStudios/cargo-deny-action@v1

  # ============================================
  # Stage 2: Build & Test Matrix
  # ============================================

  build-and-test:
    name: Build & Test (${{ matrix.os }}, ${{ matrix.rust }})
    needs: [fmt, clippy]
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, "1.75"]  # MSRV
        exclude:
          - os: windows-latest
            rust: "1.75"
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}
      - uses: Swatinem/rust-cache@v2

      - name: Build all crates
        run: cargo build --all-targets --all-features

      - name: Run tests
        run: cargo test --all-features -- --nocapture

      - name: Run doc tests
        run: cargo test --doc --all-features

  # ============================================
  # Stage 3: WASM Build
  # ============================================

  wasm-build:
    name: WASM Build
    needs: [fmt, clippy]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - uses: Swatinem/rust-cache@v2

      - name: Install wasm-pack
        run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

      - name: Build WASM crates
        run: |
          for crate in infra-errors infra-config infra-json infra-crypto infra-vector infra-id infra-auth infra-schema; do
            echo "Building $crate for WASM..."
            cd crates/$crate
            wasm-pack build --target web --features wasm
            cd ../..
          done

      - name: Upload WASM artifacts
        uses: actions/upload-artifact@v4
        with:
          name: wasm-builds
          path: crates/*/pkg/

  # ============================================
  # Stage 4: Integration Tests
  # ============================================

  integration-tests:
    name: Integration Tests
    needs: [build-and-test]
    runs-on: ubuntu-latest
    services:
      # Add any required services (e.g., Redis, NATS)
      redis:
        image: redis:7
        ports:
          - 6379:6379
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Run integration tests
        run: cargo test --test '*' --all-features
        env:
          REDIS_URL: redis://localhost:6379

  # ============================================
  # Stage 5: TypeScript SDK Build
  # ============================================

  typescript-sdk:
    name: TypeScript SDK
    needs: [wasm-build]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
      - uses: pnpm/action-setup@v2
        with:
          version: 8

      - name: Download WASM artifacts
        uses: actions/download-artifact@v4
        with:
          name: wasm-builds
          path: sdk/ts/wasm/

      - name: Install dependencies
        working-directory: sdk/ts
        run: pnpm install

      - name: Build SDK
        working-directory: sdk/ts
        run: pnpm build

      - name: Run SDK tests
        working-directory: sdk/ts
        run: pnpm test

      - name: Upload SDK artifacts
        uses: actions/upload-artifact@v4
        with:
          name: typescript-sdk
          path: sdk/ts/dist/

  # ============================================
  # Stage 6: Documentation
  # ============================================

  docs:
    name: Documentation
    needs: [build-and-test]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - uses: Swatinem/rust-cache@v2

      - name: Build docs
        run: RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --all-features --no-deps

      - name: Upload docs
        uses: actions/upload-artifact@v4
        with:
          name: documentation
          path: target/doc/

  # ============================================
  # Stage 7: Dependency Check
  # ============================================

  circular-deps:
    name: Circular Dependency Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Check for circular dependencies
        run: |
          cargo tree --all-features -e no-dev 2>&1 | grep -i "circular" && exit 1 || exit 0

      - name: Verify layer constraints
        run: |
          # Custom script to verify dependency layers
          ./tools/dependency-checker/check-layers.sh

  # ============================================
  # Final: Summary
  # ============================================

  ci-success:
    name: CI Success
    needs: [build-and-test, wasm-build, integration-tests, typescript-sdk, docs, circular-deps, deny]
    runs-on: ubuntu-latest
    steps:
      - run: echo "All CI checks passed!"
```

### 6.2 Release Workflow (`.github/workflows/release.yml`)

```yaml
name: Release

on:
  push:
    tags:
      - 'v[0-9]+.[0-9]+.[0-9]+'
      - 'v[0-9]+.[0-9]+.[0-9]+-*'

permissions:
  contents: write
  packages: write

env:
  CARGO_TERM_COLOR: always

jobs:
  # ============================================
  # Build Release Artifacts
  # ============================================

  build-release:
    name: Build Release (${{ matrix.target }})
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
          - target: x86_64-apple-darwin
            os: macos-latest
          - target: aarch64-apple-darwin
            os: macos-latest
          - target: x86_64-pc-windows-msvc
            os: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Build
        run: cargo build --release --target ${{ matrix.target }}

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: release-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/

  # ============================================
  # Publish to crates.io
  # ============================================

  publish-crates:
    name: Publish to crates.io
    needs: [build-release]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Publish crates (in dependency order)
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
        run: |
          # Layer 0
          cargo publish -p infra-errors --allow-dirty
          sleep 30

          # Layer 1
          cargo publish -p infra-config --allow-dirty
          cargo publish -p infra-json --allow-dirty
          cargo publish -p infra-crypto --allow-dirty
          cargo publish -p infra-id --allow-dirty
          sleep 30

          # Layer 2
          cargo publish -p infra-otel --allow-dirty
          cargo publish -p infra-http --allow-dirty
          cargo publish -p infra-fs --allow-dirty
          cargo publish -p infra-schema --allow-dirty
          cargo publish -p infra-mq --allow-dirty
          cargo publish -p infra-audit --allow-dirty
          cargo publish -p infra-sim --allow-dirty
          sleep 30

          # Layer 3
          cargo publish -p infra-vector --allow-dirty
          cargo publish -p infra-auth --allow-dirty
          cargo publish -p infra-router --allow-dirty

  # ============================================
  # Publish TypeScript SDK
  # ============================================

  publish-npm:
    name: Publish to npm
    needs: [build-release]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          registry-url: 'https://registry.npmjs.org'
      - uses: pnpm/action-setup@v2
        with:
          version: 8

      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Build WASM
        run: |
          curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
          ./tools/wasm-builder/build-all.sh

      - name: Build and publish SDK
        working-directory: sdk/ts
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
        run: |
          pnpm install
          pnpm build
          pnpm publish --access public --no-git-checks

  # ============================================
  # Create GitHub Release
  # ============================================

  create-release:
    name: Create GitHub Release
    needs: [publish-crates, publish-npm]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts/

      - name: Create release
        uses: softprops/action-gh-release@v1
        with:
          draft: false
          prerelease: ${{ contains(github.ref, '-') }}
          generate_release_notes: true
          files: |
            artifacts/**/*
```

---

## 7. WASM Build Architecture

### 7.1 WASM Build Script (`tools/wasm-builder/build-all.sh`)

```bash
#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
OUTPUT_DIR="$ROOT_DIR/sdk/ts/wasm"

# WASM-compatible crates
WASM_CRATES=(
    "infra-errors"
    "infra-config"
    "infra-json"
    "infra-crypto"
    "infra-vector"
    "infra-id"
    "infra-auth"
    "infra-schema"
)

echo "Building WASM modules..."

mkdir -p "$OUTPUT_DIR"

for crate in "${WASM_CRATES[@]}"; do
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Building: $crate"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    cd "$ROOT_DIR/crates/$crate"

    # Build with wasm-pack
    wasm-pack build \
        --target web \
        --out-dir "$OUTPUT_DIR/$crate" \
        --features wasm \
        --release

    # Optimize with wasm-opt if available
    if command -v wasm-opt &> /dev/null; then
        wasm-opt -O3 \
            --enable-simd \
            "$OUTPUT_DIR/$crate/${crate//-/_}_bg.wasm" \
            -o "$OUTPUT_DIR/$crate/${crate//-/_}_bg.wasm"
    fi

    echo "✓ $crate built successfully"
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "All WASM modules built successfully!"
echo "Output directory: $OUTPUT_DIR"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
```

### 7.2 WASM Feature Flags

Each WASM-compatible crate uses consistent feature flags:

```toml
[features]
default = ["std"]
std = []                    # Standard library (native)
wasm = [                    # WASM target
    "wasm-bindgen",
    "js-sys",
    "serde-wasm-bindgen",
]
```

### 7.3 Conditional Compilation Pattern

```rust
// Native implementation
#[cfg(not(target_arch = "wasm32"))]
mod native {
    pub struct NativeImpl { /* ... */ }
}

// WASM implementation
#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct WasmImpl { /* ... */ }
}

// Re-export appropriate implementation
#[cfg(not(target_arch = "wasm32"))]
pub use native::NativeImpl as Impl;

#[cfg(target_arch = "wasm32")]
pub use wasm::WasmImpl as Impl;
```

---

## 8. TypeScript SDK Architecture

### 8.1 SDK Package Structure

```
sdk/ts/
├── packages/
│   ├── infra-core/                   # Core types and utilities
│   │   ├── src/
│   │   │   ├── index.ts
│   │   │   ├── errors.ts
│   │   │   ├── config.ts
│   │   │   ├── json.ts
│   │   │   └── types.ts
│   │   ├── package.json
│   │   └── tsconfig.json
│   │
│   ├── infra-vector/                 # Vector operations
│   │   ├── src/
│   │   │   ├── index.ts
│   │   │   ├── store.ts
│   │   │   ├── filter.ts
│   │   │   └── types.ts
│   │   ├── package.json
│   │   └── tsconfig.json
│   │
│   ├── infra-crypto/                 # Crypto utilities
│   │   ├── src/
│   │   │   ├── index.ts
│   │   │   ├── hash.ts
│   │   │   ├── cipher.ts
│   │   │   ├── sign.ts
│   │   │   └── jwt.ts
│   │   ├── package.json
│   │   └── tsconfig.json
│   │
│   └── infra-client/                 # Full bundle
│       ├── src/
│       │   └── index.ts              # Re-exports all packages
│       ├── package.json
│       └── tsconfig.json
│
├── wasm/                             # Compiled WASM modules
│   ├── infra-errors/
│   ├── infra-config/
│   ├── infra-json/
│   ├── infra-crypto/
│   ├── infra-vector/
│   └── ...
│
├── scripts/
│   ├── build.ts
│   ├── generate-types.ts
│   └── bundle.ts
│
├── package.json
├── pnpm-workspace.yaml
├── tsconfig.base.json
└── vitest.config.ts
```

### 8.2 SDK Root `package.json`

```json
{
  "name": "@llm-dev-ops/infra-sdk",
  "version": "0.1.0",
  "description": "TypeScript SDK for LLM-Dev-Ops Infrastructure",
  "private": true,
  "type": "module",
  "scripts": {
    "build": "turbo run build",
    "test": "vitest run",
    "test:watch": "vitest",
    "lint": "eslint packages/*/src/**/*.ts",
    "typecheck": "turbo run typecheck",
    "clean": "turbo run clean && rm -rf node_modules"
  },
  "devDependencies": {
    "@types/node": "^20.0.0",
    "eslint": "^8.0.0",
    "prettier": "^3.0.0",
    "tsup": "^8.0.0",
    "turbo": "^2.0.0",
    "typescript": "^5.3.0",
    "vitest": "^1.0.0"
  },
  "engines": {
    "node": ">=18.0.0",
    "pnpm": ">=8.0.0"
  }
}
```

### 8.3 SDK Package Example (`packages/infra-core/package.json`)

```json
{
  "name": "@llm-dev-ops/infra-core",
  "version": "0.1.0",
  "description": "Core types and utilities for LLM-Dev-Ops Infrastructure SDK",
  "type": "module",
  "main": "./dist/index.cjs",
  "module": "./dist/index.js",
  "types": "./dist/index.d.ts",
  "exports": {
    ".": {
      "import": "./dist/index.js",
      "require": "./dist/index.cjs",
      "types": "./dist/index.d.ts"
    },
    "./errors": {
      "import": "./dist/errors.js",
      "require": "./dist/errors.cjs",
      "types": "./dist/errors.d.ts"
    },
    "./config": {
      "import": "./dist/config.js",
      "require": "./dist/config.cjs",
      "types": "./dist/config.d.ts"
    }
  },
  "files": [
    "dist",
    "wasm"
  ],
  "scripts": {
    "build": "tsup src/index.ts --format cjs,esm --dts --clean",
    "typecheck": "tsc --noEmit",
    "clean": "rm -rf dist"
  },
  "dependencies": {
    "@llm-dev-ops/infra-wasm-errors": "workspace:*"
  },
  "peerDependencies": {
    "typescript": ">=5.0.0"
  }
}
```

### 8.4 SDK Usage Example

```typescript
// Using the full client
import { InfraClient } from '@llm-dev-ops/infra-client';

const client = await InfraClient.create({
  vectorDimensions: 1536,
  enableCrypto: true,
});

// Vector operations
const store = client.vector;
await store.insert('doc-1', new Float32Array(1536).fill(0.1), {
  title: 'My Document',
});

const results = await store.search(queryVector, 10, {
  filter: { field: 'category', op: 'eq', value: 'tech' },
});

// Crypto operations
const crypto = client.crypto;
const hash = crypto.sha256('hello world');
const encrypted = crypto.encrypt(data, key);

// Using individual packages
import { VectorStore } from '@llm-dev-ops/infra-vector';
import { sha256, AES256GCM } from '@llm-dev-ops/infra-crypto';

const store = await VectorStore.create(1536);
const hash = sha256('data');
```

---

## 9. Release & Versioning Strategy

### 9.1 Versioning Scheme

All infra crates follow **Semantic Versioning 2.0.0**:

```
MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]

Examples:
- 0.1.0        Initial development
- 0.2.0        New features (backward compatible)
- 0.2.1        Bug fixes
- 1.0.0        First stable release
- 1.0.0-alpha  Pre-release
- 1.0.0-rc.1   Release candidate
```

### 9.2 Version Synchronization

All 15 infra crates share the same version number:

```toml
# Workspace Cargo.toml
[workspace.package]
version = "0.1.0"

# Individual crate Cargo.toml
[package]
version.workspace = true
```

### 9.3 Release Process

```
┌─────────────────────────────────────────────────────────────┐
│                    RELEASE PROCESS                           │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  1. Update Version                                           │
│     └── Edit workspace.package.version in root Cargo.toml   │
│                                                              │
│  2. Update Changelog                                         │
│     └── Add release notes to CHANGELOG.md                   │
│                                                              │
│  3. Create Release PR                                        │
│     └── PR: "Release v0.2.0"                                │
│                                                              │
│  4. Merge to main                                            │
│     └── Triggers CI pipeline                                │
│                                                              │
│  5. Create Git Tag                                           │
│     └── git tag v0.2.0 && git push --tags                   │
│                                                              │
│  6. Automated Release (GitHub Actions)                       │
│     ├── Build all targets                                   │
│     ├── Publish to crates.io (in order)                     │
│     ├── Publish TypeScript SDK to npm                       │
│     └── Create GitHub Release                               │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 9.4 Crate Publishing Order

Due to inter-crate dependencies, publishing must follow this order:

```
Phase 1 (Layer 0):
  └── infra-errors

Phase 2 (Layer 1) - Can be parallel:
  ├── infra-config
  ├── infra-json
  ├── infra-crypto
  └── infra-id

Phase 3 (Layer 2) - Can be parallel:
  ├── infra-otel
  ├── infra-http
  ├── infra-fs
  ├── infra-schema
  ├── infra-mq
  ├── infra-audit
  └── infra-sim

Phase 4 (Layer 3) - Can be parallel:
  ├── infra-vector
  ├── infra-auth
  └── infra-router
```

---

## 10. Configuration Files

### 10.1 `rust-toolchain.toml`

```toml
[toolchain]
channel = "1.75"
components = ["rustfmt", "clippy", "rust-src"]
targets = ["wasm32-unknown-unknown"]
profile = "default"
```

### 10.2 `deny.toml`

```toml
[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"
notice = "warn"

[licenses]
unlicensed = "deny"
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Zlib",
    "MPL-2.0",
    "Unicode-DFS-2016",
]
copyleft = "deny"
default = "deny"

[bans]
multiple-versions = "warn"
wildcards = "deny"
highlight = "all"
deny = [
    # Banned crates
]
skip = [
    # Known duplicates to skip
]

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
allow-git = [
    "https://github.com/ruvnet/ruvector",
]
```

### 10.3 `clippy.toml`

```toml
# Clippy configuration
msrv = "1.75"
cognitive-complexity-threshold = 30
disallowed-methods = [
    { path = "std::env::var", reason = "Use infra-config instead" },
]
disallowed-types = [
    { path = "std::collections::HashMap", reason = "Use hashbrown::HashMap for consistency" },
]
```

### 10.4 `rustfmt.toml`

```toml
edition = "2021"
max_width = 100
tab_spaces = 4
use_small_heuristics = "Default"
imports_granularity = "Module"
group_imports = "StdExternalCrate"
reorder_imports = true
reorder_modules = true
format_code_in_doc_comments = true
format_macro_matchers = true
format_strings = true
wrap_comments = true
comment_width = 80
normalize_comments = true
normalize_doc_attributes = true
```

---

## 11. Directory Creation Script

```bash
#!/bin/bash
# tools/init-project.sh
# Creates the complete project structure

set -euo pipefail

ROOT_DIR="$(pwd)"

# Create crate directories
CRATES=(
    "infra-errors"
    "infra-config"
    "infra-otel"
    "infra-json"
    "infra-http"
    "infra-vector"
    "infra-crypto"
    "infra-auth"
    "infra-id"
    "infra-fs"
    "infra-mq"
    "infra-router"
    "infra-schema"
    "infra-audit"
    "infra-sim"
)

echo "Creating project structure..."

# Create crate structure
for crate in "${CRATES[@]}"; do
    echo "Creating crate: $crate"
    mkdir -p "crates/$crate/src/impl"
    mkdir -p "crates/$crate/src/wasm"
    mkdir -p "crates/$crate/src/testing"
    mkdir -p "crates/$crate/tests/unit"
    mkdir -p "crates/$crate/tests/integration"
    mkdir -p "crates/$crate/benches"
    touch "crates/$crate/src/lib.rs"
    touch "crates/$crate/src/impl/mod.rs"
    touch "crates/$crate/src/wasm/mod.rs"
    touch "crates/$crate/src/testing/mod.rs"
    touch "crates/$crate/README.md"
done

# Create SDK structure
mkdir -p "sdk/ts/packages/infra-core/src"
mkdir -p "sdk/ts/packages/infra-vector/src"
mkdir -p "sdk/ts/packages/infra-crypto/src"
mkdir -p "sdk/ts/packages/infra-client/src"
mkdir -p "sdk/ts/wasm"
mkdir -p "sdk/ts/scripts"

# Create other directories
mkdir -p "examples/rust/basic-usage"
mkdir -p "examples/rust/vector-search"
mkdir -p "examples/rust/full-stack"
mkdir -p "examples/typescript/browser"
mkdir -p "examples/typescript/node"
mkdir -p "benches"
mkdir -p "tests/integration"
mkdir -p "tools/wasm-builder"
mkdir -p "tools/sdk-generator"
mkdir -p "tools/dependency-checker"
mkdir -p "docs/architecture"
mkdir -p "docs/api"
mkdir -p "docs/guides"
mkdir -p ".github/workflows"

echo "Project structure created successfully!"
```

---

## 12. Acceptance Criteria

### 12.1 Architecture Phase Completion

- [x] Complete repository structure defined
- [x] All 15 crate architectures specified
- [x] Dependency graph with strict layering
- [x] Workspace Cargo.toml configuration
- [x] CI/CD pipeline (ci.yml, release.yml)
- [x] WASM build system
- [x] TypeScript SDK architecture
- [x] Release and versioning strategy
- [x] Configuration files (deny.toml, clippy.toml, rustfmt.toml)

### 12.2 Ready for Next Phase

Upon user approval, proceed to **Phase 4: Refinement** which will:
- Review and optimize the architecture
- Address edge cases and error handling
- Refine API ergonomics
- Add performance considerations
- Document migration paths

---

**Document Status:** Awaiting user approval to proceed to Refinement phase.
