<p align="center">
  <img src="https://img.shields.io/badge/rust-stable-orange?style=for-the-badge&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/license-Source%20Available-blue?style=for-the-badge" alt="License">
  <img src="https://img.shields.io/badge/tests-129%20passing-brightgreen?style=for-the-badge" alt="Tests">
  <img src="https://img.shields.io/badge/crates-15-purple?style=for-the-badge" alt="Crates">
</p>

<h1 align="center">LLM-Dev-Ops Infrastructure</h1>

<p align="center">
  <strong>A production-grade infrastructure layer for building LLM-powered applications</strong>
</p>

<p align="center">
  <a href="#features">Features</a> &bull;
  <a href="#quick-start">Quick Start</a> &bull;
  <a href="#architecture">Architecture</a> &bull;
  <a href="#crates">Crates</a> &bull;
  <a href="#typescript-sdk">TypeScript SDK</a> &bull;
  <a href="#documentation">Documentation</a>
</p>

---

## Overview

**LLM-Dev-Ops Infrastructure** is a comprehensive Rust workspace providing battle-tested primitives for building scalable, observable, and secure LLM applications. Built with a zero-circular-dependency architecture across 15 specialized crates, it offers everything from cryptographic operations to vector similarity search.

```rust
use infra_errors::InfraResult;
use infra_crypto::jwt::{JwtSigner, Claims};
use infra_vector::{Vector, cosine_similarity};

fn main() -> InfraResult<()> {
    // JWT Authentication
    let signer = JwtSigner::hs256(b"your-secret-key-at-least-32-bytes");
    let token = signer.sign(&Claims::new().with_subject("user-123"))?;

    // Vector Similarity
    let embedding_a = Vector::new(vec![0.1, 0.2, 0.3]);
    let embedding_b = Vector::new(vec![0.15, 0.25, 0.35]);
    let similarity = cosine_similarity(&embedding_a, &embedding_b)?;

    println!("Similarity: {:.4}", similarity);
    Ok(())
}
```

## Features

- **Unified Error Handling** - Single `InfraError` type with rich context across all crates
- **OpenTelemetry Integration** - Built-in tracing, metrics, and distributed context propagation
- **Cryptographic Primitives** - AES-256-GCM encryption, Ed25519 signatures, Argon2id hashing, JWT
- **Vector Operations** - Embeddings, similarity search, and in-memory indexing
- **Authentication & Authorization** - JWT tokens, sessions, RBAC policies
- **API Gateway** - Routing, load balancing, middleware support
- **Message Queues** - Async pub/sub with acknowledgment and dead-letter support
- **WASM Compatible** - Core crates compile to WebAssembly
- **TypeScript SDK** - First-class TypeScript support for web applications

## Infrastructure

### Docker Services

This repository includes Docker Compose configuration for development dependencies:

- **RuvVector** (PostgreSQL 17 with RuvVector extension) - Advanced vector database
  - pgvector-compatible `vector` type with HNSW indexing
  - Hyperbolic embeddings (Poincaré ball) for hierarchical data
  - GNN operations (GCN, GraphSAGE)
  - Graph operations with Cypher queries
  - Self-learning index optimization
- **Redis 7.4** for caching (optional)

#### Database Service Naming

The Postgres-compatible database service is provided by **RuvVector** and is named `ruvector` at the Docker level (container name: `infra-ruvector`). There is no service named `postgres` in this repository.

**Consumers must use environment variables for database connectivity:**

| Variable | Default | Description |
|----------|---------|-------------|
| `POSTGRES_HOST` | `localhost` | Database hostname |
| `POSTGRES_PORT` | `5432` | Database port |
| `POSTGRES_USER` | `infra` | Database user |
| `POSTGRES_PASSWORD` | `infra_password` | Database password |
| `POSTGRES_DB` | `infra_vectors` | Database name |
| `DATABASE_URL` | (constructed) | Full connection URL |

**Do not hardcode assumptions about a service named `postgres`.** If a `postgres` network alias exists, it is provided solely for backward compatibility with legacy configurations and should not be relied upon for new development. Always use the environment variables above to configure database connections.

### Quick Start (Infrastructure)

```bash
# Set up environment (development defaults)
source scripts/env-setup.sh

# Start services
docker compose up -d

# Wait for all services to be healthy
docker compose up -d --wait

# Verify RuvVector extensions
./scripts/verify-postgres-extensions.sh
```

### Environment Configuration

All configuration uses shell-exported environment variables. **No .env file is used.**

See `.env.example` for available variables and documentation.

### Security Considerations

- Default passwords are for development only
- Services bind to localhost (127.0.0.1) by default
- Change all passwords before production deployment
- Never commit secrets to the repository

### Production Readiness

The `ruvnet/ruvector:latest` image tag is intentionally unpinned during infrastructure wiring and local integration. **Before any production deployment, this image must be pinned to a specific version** (e.g., `ruvnet/ruvector:1.2.3`) in `docker-compose.yml` to ensure reproducible builds and avoid unexpected breaking changes.

### Production Deployment

For production deployments:

1. **Pin all Docker image versions** to specific tags (not `:latest`)
2. Set secure passwords via environment variables
3. Use external secrets management (Vault, AWS Secrets Manager, etc.)
4. Consider managed database services for PostgreSQL
5. Enable TLS for all connections
6. Configure proper network segmentation

## Quick Start (Development)

### Prerequisites

- Rust 1.75+ (stable)
- Cargo
- Docker and Docker Compose (for infrastructure services)

### Installation

Add the crates you need to your `Cargo.toml`:

```toml
[dependencies]
infra-errors = { git = "https://github.com/LLM-Dev-Ops/infra" }
infra-crypto = { git = "https://github.com/LLM-Dev-Ops/infra" }
infra-config = { git = "https://github.com/LLM-Dev-Ops/infra" }
infra-vector = { git = "https://github.com/LLM-Dev-Ops/infra" }
# ... add other crates as needed
```

### Building from Source

```bash
git clone https://github.com/LLM-Dev-Ops/infra.git
cd infra
cargo build --workspace
cargo test --workspace
```

## Architecture

The infrastructure is organized into a **4-layer dependency model** ensuring clean separation of concerns and zero circular dependencies:

```
+-----------------------------------------------------------------+
|  Layer 3: Application                                           |
|  +-------------+  +-------------+  +-------------+              |
|  | infra-vector|  | infra-auth  |  |infra-router |              |
|  +------+------+  +------+------+  +------+------+              |
+---------+----------------+----------------+---------------------+
|  Layer 2: Services       |                |                     |
|  +------+------+  +------+------+  +------+------+              |
|  | infra-otel  |  | infra-http  |  |  infra-fs   |              |
|  | infra-schema|  |  infra-mq   |  | infra-audit |              |
|  |  infra-sim  |  +-------------+  +-------------+              |
|  +------+------+                                                |
+---------+-------------------------------------------------------+
|  Layer 1: Utilities                                             |
|  +------+------+  +-------------+  +-------------+              |
|  |infra-config |  | infra-json  |  |infra-crypto |              |
|  |  infra-id   |  +-------------+  +-------------+              |
|  +------+------+                                                |
+---------+-------------------------------------------------------+
|  Layer 0: Foundation                                            |
|  +------+------+                                                |
|  |infra-errors |  <- All crates depend on this                  |
|  +-------------+                                                |
+-----------------------------------------------------------------+
```

## Crates

### Layer 0: Foundation

| Crate | Description |
|-------|-------------|
| **[infra-errors](./crates/infra-errors)** | Unified error handling with `InfraError` enum, retry configuration, and rich context |

### Layer 1: Utilities

| Crate | Description |
|-------|-------------|
| **[infra-config](./crates/infra-config)** | Hierarchical configuration loading with environment variable overlay and validation |
| **[infra-json](./crates/infra-json)** | JSON utilities with dot-notation path queries, diff, and merge operations |
| **[infra-crypto](./crates/infra-crypto)** | Cryptographic primitives: AES-256-GCM, Ed25519, Argon2id, JWT (HS256/384/512) |
| **[infra-id](./crates/infra-id)** | ID generation: UUID v4/v7, ULID, NanoID, Snowflake |

### Layer 2: Services

| Crate | Description |
|-------|-------------|
| **[infra-otel](./crates/infra-otel)** | OpenTelemetry 0.27 integration with tracing, metrics, and exporters |
| **[infra-http](./crates/infra-http)** | HTTP client/server with retry logic, circuit breaker, and middleware |
| **[infra-fs](./crates/infra-fs)** | File system operations with glob patterns and temp file management |
| **[infra-schema](./crates/infra-schema)** | JSON Schema validation with detailed error reporting |
| **[infra-mq](./crates/infra-mq)** | Message queue abstraction with pub/sub, acknowledgment, and TTL |
| **[infra-audit](./crates/infra-audit)** | Audit logging with configurable sinks and correlation tracking |
| **[infra-sim](./crates/infra-sim)** | Testing utilities: mock clock, chaos engineering, HTTP mocking |

### Layer 3: Application

| Crate | Description |
|-------|-------------|
| **[infra-vector](./crates/infra-vector)** | Vector operations, similarity metrics, and in-memory search index |
| **[infra-auth](./crates/infra-auth)** | Authentication (JWT), sessions, permissions, and policy engine |
| **[infra-router](./crates/infra-router)** | API gateway with routing, load balancing, and request handling |

## Usage Examples

### Configuration Management

```rust
use infra_config::{load_with_env, ConfigBuilder};
use serde::Deserialize;

#[derive(Deserialize)]
struct AppConfig {
    database_url: String,
    port: u16,
    debug: bool,
}

// Load from file with environment variable overlay
let config: AppConfig = load_with_env("config.toml", "APP_")?;

// Or build programmatically
let config = ConfigBuilder::new()
    .set("database_url", "postgres://localhost/db")
    .set("port", 8080)
    .build()?;
```

### Cryptography

```rust
use infra_crypto::{Aes256GcmCipher, Cipher, PasswordHasher, PasswordAlgorithm};
use infra_crypto::jwt::{JwtSigner, Claims};

// Symmetric encryption
let cipher = Aes256GcmCipher::from_passphrase(b"secure-passphrase")?;
let ciphertext = cipher.encrypt(b"secret data")?;
let plaintext = cipher.decrypt(&ciphertext)?;

// Password hashing
let hasher = PasswordHasher::new(PasswordAlgorithm::Argon2id);
let hash = hasher.hash("user-password")?;
assert!(hasher.verify("user-password", &hash)?);

// JWT tokens
let signer = JwtSigner::hs256(b"secret-key-at-least-32-bytes!!!");
let claims = Claims::new()
    .with_subject("user-123")
    .with_issuer("my-app");
let token = signer.sign(&claims)?;
```

### Vector Similarity Search

```rust
use infra_vector::{Vector, VectorIndex, IndexConfig, cosine_similarity};

// Create vectors
let v1 = Vector::new(vec![1.0, 0.0, 0.0]);
let v2 = Vector::new(vec![0.707, 0.707, 0.0]);

// Calculate similarity
let similarity = cosine_similarity(&v1, &v2)?; // ~0.707

// Build a search index
let mut index = VectorIndex::new(IndexConfig::new(3)); // 3 dimensions
index.insert("doc-1", Vector::new(vec![1.0, 0.0, 0.0]))?;
index.insert("doc-2", Vector::new(vec![0.0, 1.0, 0.0]))?;

// Search for similar vectors
let query = Vector::new(vec![0.9, 0.1, 0.0]);
let results = index.search(&query, 5)?; // top 5 results
```

### Authentication & Authorization

```rust
use infra_auth::{Identity, PermissionSet, PolicyEngine, Policy, Effect};

// Create an identity
let identity = Identity::user("user-123")
    .with_role("admin")
    .with_metadata("department", "engineering");

// Check permissions
let permissions = PermissionSet::new()
    .grant("documents", "read")
    .grant("documents", "write");

assert!(permissions.can("documents", "read"));

// Policy-based authorization
let mut engine = PolicyEngine::new();
engine.add_policy(Policy::new("allow-admin")
    .with_effect(Effect::Allow)
    .with_action("*")
    .with_resource("*")
    .with_condition("role", "admin"));

let allowed = engine.evaluate(&identity, "delete", "documents/123")?;
```

### API Routing

```rust
use infra_router::{Gateway, GatewayConfig, Route, Method, LoadBalancer, Backend};

// Create a gateway
let mut gateway = Gateway::new(GatewayConfig::default());

// Add routes
gateway.add_route(
    Route::new("/api/users/:id")
        .method(Method::GET)
        .handler(get_user_handler)
);

// Add load-balanced backend
let mut balancer = LoadBalancer::round_robin();
balancer.add_backend(Backend::new("http://backend-1:8080")).await;
balancer.add_backend(Backend::new("http://backend-2:8080")).await;
gateway.add_backend("users-service", balancer);
```

## TypeScript SDK

The TypeScript SDK provides browser and Node.js compatible implementations:

```bash
cd sdk/ts
npm install
npm run build
```

```typescript
import { InfraError, ErrorKind } from '@llm-dev-ops/infra';
import { generateUUID, generateULID, generateNanoID } from '@llm-dev-ops/infra/id';
import { encrypt, decrypt, hashPassword } from '@llm-dev-ops/infra/crypto';

// ID Generation
const uuid = generateUUID();      // UUID v4
const ulid = generateULID();      // ULID
const nanoid = generateNanoID();  // NanoID

// Encryption (Web Crypto API)
const key = await generateKey();
const encrypted = await encrypt(key, 'secret message');
const decrypted = await decrypt(key, encrypted);
```

## Testing

Run the complete test suite:

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p infra-crypto

# With output
cargo test --workspace -- --nocapture
```

**Test Coverage**: 129 tests across all crates

## Performance

The infrastructure is designed for production workloads:

- **Zero-copy parsing** where possible
- **Async-first** design with Tokio
- **Connection pooling** in HTTP client
- **Lock-free** data structures where applicable
- **SIMD-optimized** vector operations (via nalgebra)

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the **LLM-Dev-Ops Permanent Source-Available License**. See [LICENSE](LICENSE) for details.

## Acknowledgments

Built with these excellent crates:

- [tokio](https://tokio.rs) - Async runtime
- [serde](https://serde.rs) - Serialization
- [axum](https://github.com/tokio-rs/axum) - Web framework
- [opentelemetry](https://opentelemetry.io) - Observability
- [jsonwebtoken](https://github.com/Keats/jsonwebtoken) - JWT support
- [aes-gcm](https://github.com/RustCrypto/AEADs) - Encryption
- [ed25519-dalek](https://github.com/dalek-cryptography/ed25519-dalek) - Signatures
- [argon2](https://github.com/RustCrypto/password-hashes) - Password hashing

---

<p align="center">
  <sub>Built with Rust for the LLM era</sub>
</p>
