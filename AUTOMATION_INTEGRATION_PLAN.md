# Automation Integration Plan: LLM-Dev-Ops Infra Crates

**Generated:** 2025-12-06
**Scope:** Integration of 15 infra crates into 26 LLM-Dev-Ops repositories
**Status:** ANALYSIS COMPLETE - Awaiting Review and Approval

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Infra Crate Inventory](#2-infra-crate-inventory)
3. [Dependency Hierarchy](#3-dependency-hierarchy)
4. [Per-Repository Dependency Mapping](#4-per-repository-dependency-mapping)
5. [Dependency-Safe Modification Order](#5-dependency-safe-modification-order)
6. [WASM and TypeScript SDK Compatibility](#6-wasm-and-typescript-sdk-compatibility)
7. [Copy-Paste Ready Cargo.toml Patches](#7-copy-paste-ready-cargotoml-patches)
8. [Risk Assessment](#8-risk-assessment)
9. [Implementation Checklist](#9-implementation-checklist)

---

## 1. Executive Summary

This document provides a complete, automated integration plan for incorporating the `llm-dev-ops-infra` crates into all 26 LLM-Dev-Ops repositories.

### Key Statistics

| Metric | Value |
|--------|-------|
| Total Infra Crates | 15 |
| Target Repositories | 26 |
| WASM-Compatible Crates | 5 (fully), 4 (partial) |
| Dependency Layers | 4 (Level 0-3) |
| Universal Crates | 3 (infra-errors, infra-config, infra-json) |

### Critical Constraints

1. **One-Directional Dependencies**: LLM-Dev-Ops repos depend on infra crates, never the reverse
2. **No Circular Dependencies**: Strictly enforced via layered architecture
3. **No Runtime Logic Modification**: Integration is Cargo.toml additions only
4. **SPARC Specification Compliance**: All mappings follow the SPARC specification

---

## 2. Infra Crate Inventory

### 2.1 Complete Crate List

| Crate | Version | Purpose | Layer | WASM |
|-------|---------|---------|-------|------|
| `infra-errors` | 0.1.0 | Unified error handling (`InfraError`) | 0 | Yes |
| `infra-id` | 0.1.0 | ID generation (UUID, ULID, Snowflake) | 1 | Yes |
| `infra-crypto` | 0.1.0 | Cryptographic operations (AES, Ed25519, JWT) | 1 | Yes |
| `infra-json` | 0.1.0 | JSON serialization, path queries, diff/merge | 1 | Yes |
| `infra-config` | 0.1.0 | Configuration loading, hot-reload | 1 | Partial |
| `infra-otel` | 0.1.0 | OpenTelemetry 0.27 tracing/metrics | 2 | No |
| `infra-http` | 0.1.0 | HTTP client/server (reqwest, axum) | 2 | Partial |
| `infra-fs` | 0.1.0 | Filesystem operations (local, S3, GCS) | 2 | No |
| `infra-schema` | 0.1.0 | JSON Schema validation | 2 | Partial |
| `infra-mq` | 0.1.0 | Message queue (memory, Redis, RabbitMQ) | 2 | No |
| `infra-audit` | 0.1.0 | Security audit logging | 2 | No |
| `infra-sim` | 0.1.0 | Testing utilities, mocks | 2 | No |
| `infra-vector` | 0.1.0 | Vector operations, embeddings, HNSW | 3 | Partial |
| `infra-auth` | 0.1.0 | Authentication (JWT, API key, OAuth) | 3 | Partial |
| `infra-router` | 0.1.0 | Request routing, load balancing | 3 | No |

### 2.2 Internal Dependency Graph

```
Level 0 (Leaf - No Infra Dependencies):
└── infra-errors

Level 1 (Depends on Level 0):
├── infra-id ─────────────────────► infra-errors
├── infra-crypto ─────────────────► infra-errors
├── infra-json ───────────────────► infra-errors
└── infra-config ─────────────────► infra-errors

Level 2 (Depends on Level 0-1):
├── infra-otel ───────────────────► infra-errors
├── infra-http ───────────────────► infra-errors, infra-otel
├── infra-fs ─────────────────────► infra-errors
├── infra-schema ─────────────────► infra-errors
├── infra-mq ─────────────────────► infra-errors
├── infra-audit ──────────────────► infra-errors, infra-id
└── infra-sim ────────────────────► infra-errors

Level 3 (Depends on Level 0-2):
├── infra-vector ─────────────────► infra-errors
├── infra-auth ───────────────────► infra-errors, infra-crypto
└── infra-router ─────────────────► infra-errors, infra-otel, infra-http, infra-auth
```

---

## 3. Dependency Hierarchy

### 3.1 Infra Crate Build Order

When publishing to crates.io or building in a clean workspace, crates must be built/published in this order:

**Phase 1 (No dependencies):**
```
infra-errors
```

**Phase 2 (Depends on Phase 1):**
```
infra-id
infra-crypto
infra-json
infra-config
```

**Phase 3 (Depends on Phase 1-2):**
```
infra-otel
infra-http (after infra-otel)
infra-fs
infra-schema
infra-mq
infra-audit (after infra-id)
infra-sim
```

**Phase 4 (Depends on Phase 1-3):**
```
infra-vector
infra-auth (after infra-crypto)
infra-router (after infra-http, infra-auth)
```

---

## 4. Per-Repository Dependency Mapping

### 4.1 Repository → Infra Crate Matrix

Based on the SPARC specification, each LLM-Dev-Ops repository requires specific infra crates based on its functional surface area:

| Repository | Required Infra Crates | Category |
|------------|----------------------|----------|
| **test-bench** | errors, config, json, sim | Testing |
| **observatory** | errors, config, json, otel | Monitoring |
| **shield** | errors, config, json, auth, crypto | Security |
| **sentinel** | errors, config, json, otel, audit | Monitoring |
| **memory-graph** | errors, config, json, vector | Data |
| **latency-lens** | errors, config, json, otel | Monitoring |
| **forge** | errors, config, json, http | Core |
| **edge-agent** | errors, config, json, http, auth | Gateway |
| **auto-optimizer** | errors, config, json, vector | AI |
| **incident-manager** | errors, config, json, otel, mq, audit | Operations |
| **orchestrator** | errors, config, json, otel, http, mq, id | Core |
| **cost-ops** | errors, config, json, audit | Utilities |
| **governance-dashboard** | errors, config, json, auth, audit | Security |
| **policy-engine** | errors, config, json, schema, auth | Security |
| **registry** | errors, config, json, fs, schema | Data |
| **marketplace** | errors, config, json, http, auth | Gateway |
| **analytics-hub** | errors, config, json, otel, vector | Data |
| **config-manager** | errors, config, json, fs | Core |
| **schema-registry** | errors, config, json, schema | Utilities |
| **connector-hub** | errors, config, json, http, mq | Messaging |
| **copilot-agent** | errors, config, json, http, auth, vector | AI |
| **simulator** | errors, config, json, sim | Testing |
| **benchmark-exchange** | errors, config, json, otel, sim | Testing |
| **inference-gateway** | errors, config, json, otel, http, router | Gateway |
| **data-vault** | errors, config, json, crypto, fs, audit | Data |
| **research-lab** | errors, config, json, vector, sim | AI |

### 4.2 Detailed Dependency Summary by Repository

#### 4.2.1 Core Repositories

**orchestrator**
- Purpose: Central workflow orchestration
- Dependencies: `infra-errors`, `infra-config`, `infra-json`, `infra-otel`, `infra-http`, `infra-mq`, `infra-id`
- Feature Flags: `otel`, `http-client`, `mq-memory`

**forge**
- Purpose: Development and build tooling
- Dependencies: `infra-errors`, `infra-config`, `infra-json`, `infra-http`
- Feature Flags: `http-client`

**config-manager**
- Purpose: Configuration management
- Dependencies: `infra-errors`, `infra-config`, `infra-json`, `infra-fs`
- Feature Flags: `fs-async`, `fs-watch`

#### 4.2.2 Gateway Repositories

**edge-agent**
- Purpose: Edge deployment agent
- Dependencies: `infra-errors`, `infra-config`, `infra-json`, `infra-http`, `infra-auth`
- Feature Flags: `http-client`, `http-server`

**inference-gateway**
- Purpose: Model inference routing
- Dependencies: `infra-errors`, `infra-config`, `infra-json`, `infra-otel`, `infra-http`, `infra-router`
- Feature Flags: `otel`, `http-server`

**marketplace**
- Purpose: Model/plugin marketplace
- Dependencies: `infra-errors`, `infra-config`, `infra-json`, `infra-http`, `infra-auth`
- Feature Flags: `http-server`, `auth-jwt`

#### 4.2.3 Data Repositories

**memory-graph**
- Purpose: Graph-based memory storage
- Dependencies: `infra-errors`, `infra-config`, `infra-json`, `infra-vector`
- Feature Flags: None required

**registry**
- Purpose: Model versioning and registry
- Dependencies: `infra-errors`, `infra-config`, `infra-json`, `infra-fs`, `infra-schema`
- Feature Flags: `fs-async`

**data-vault**
- Purpose: Secure data storage
- Dependencies: `infra-errors`, `infra-config`, `infra-json`, `infra-crypto`, `infra-fs`, `infra-audit`
- Feature Flags: `fs-async`, `crypto-aes`

**analytics-hub**
- Purpose: Analytics and metrics aggregation
- Dependencies: `infra-errors`, `infra-config`, `infra-json`, `infra-otel`, `infra-vector`
- Feature Flags: `otel`

#### 4.2.4 AI/ML Repositories

**auto-optimizer**
- Purpose: Automatic optimization
- Dependencies: `infra-errors`, `infra-config`, `infra-json`, `infra-vector`
- Feature Flags: None required

**copilot-agent**
- Purpose: AI assistant agent
- Dependencies: `infra-errors`, `infra-config`, `infra-json`, `infra-http`, `infra-auth`, `infra-vector`
- Feature Flags: `http-client`, `auth-jwt`

**research-lab**
- Purpose: Experimentation platform
- Dependencies: `infra-errors`, `infra-config`, `infra-json`, `infra-vector`, `infra-sim`
- Feature Flags: None required

#### 4.2.5 Monitoring Repositories

**observatory**
- Purpose: System observability
- Dependencies: `infra-errors`, `infra-config`, `infra-json`, `infra-otel`
- Feature Flags: `otel`, `otel-otlp`

**sentinel**
- Purpose: Security monitoring
- Dependencies: `infra-errors`, `infra-config`, `infra-json`, `infra-otel`, `infra-audit`
- Feature Flags: `otel`

**latency-lens**
- Purpose: Latency profiling
- Dependencies: `infra-errors`, `infra-config`, `infra-json`, `infra-otel`
- Feature Flags: `otel`, `otel-metrics`

#### 4.2.6 Security Repositories

**shield**
- Purpose: Security enforcement
- Dependencies: `infra-errors`, `infra-config`, `infra-json`, `infra-auth`, `infra-crypto`
- Feature Flags: `auth-jwt`, `crypto-aes`

**governance-dashboard**
- Purpose: Governance UI
- Dependencies: `infra-errors`, `infra-config`, `infra-json`, `infra-auth`, `infra-audit`
- Feature Flags: `auth-jwt`

**policy-engine**
- Purpose: Policy evaluation
- Dependencies: `infra-errors`, `infra-config`, `infra-json`, `infra-schema`, `infra-auth`
- Feature Flags: `auth-jwt`

#### 4.2.7 Operations Repositories

**incident-manager**
- Purpose: Incident handling
- Dependencies: `infra-errors`, `infra-config`, `infra-json`, `infra-otel`, `infra-mq`, `infra-audit`
- Feature Flags: `otel`, `mq-memory`

**cost-ops**
- Purpose: Cost tracking
- Dependencies: `infra-errors`, `infra-config`, `infra-json`, `infra-audit`
- Feature Flags: None required

#### 4.2.8 Messaging Repositories

**connector-hub**
- Purpose: Integration connectors
- Dependencies: `infra-errors`, `infra-config`, `infra-json`, `infra-http`, `infra-mq`
- Feature Flags: `http-client`, `mq-memory`

#### 4.2.9 Utilities Repositories

**schema-registry**
- Purpose: Schema management
- Dependencies: `infra-errors`, `infra-config`, `infra-json`, `infra-schema`
- Feature Flags: None required

#### 4.2.10 Testing Repositories

**test-bench**
- Purpose: Testing infrastructure
- Dependencies: `infra-errors`, `infra-config`, `infra-json`, `infra-sim`
- Feature Flags: None required

**simulator**
- Purpose: Environment simulation
- Dependencies: `infra-errors`, `infra-config`, `infra-json`, `infra-sim`
- Feature Flags: None required

**benchmark-exchange**
- Purpose: Performance benchmarking
- Dependencies: `infra-errors`, `infra-config`, `infra-json`, `infra-otel`, `infra-sim`
- Feature Flags: `otel`

---

## 5. Dependency-Safe Modification Order

To ensure no build failures during integration, repositories must be modified in this order, respecting the dependency graph:

### Phase 1: Repositories with Base Dependencies Only
*These repositories only need `infra-errors`, `infra-config`, `infra-json`*

```
1. config-manager (also needs infra-fs)
2. schema-registry (also needs infra-schema)
3. cost-ops (also needs infra-audit)
```

### Phase 2: Repositories with Testing Dependencies
*These add `infra-sim`*

```
4. test-bench
5. simulator
6. research-lab (also needs infra-vector)
```

### Phase 3: Repositories with Monitoring Dependencies
*These add `infra-otel`*

```
7. observatory
8. latency-lens
9. sentinel (also needs infra-audit)
10. benchmark-exchange (also needs infra-sim)
```

### Phase 4: Repositories with Data Dependencies
*These add `infra-vector`, `infra-fs`*

```
11. memory-graph
12. auto-optimizer
13. registry (also needs infra-schema)
14. analytics-hub (also needs infra-otel)
```

### Phase 5: Repositories with Security Dependencies
*These add `infra-auth`, `infra-crypto`*

```
15. shield
16. governance-dashboard (also needs infra-audit)
17. policy-engine (also needs infra-schema)
18. data-vault (also needs infra-fs, infra-audit)
```

### Phase 6: Repositories with HTTP Dependencies
*These add `infra-http`*

```
19. forge
20. marketplace (also needs infra-auth)
21. connector-hub (also needs infra-mq)
22. edge-agent (also needs infra-auth)
```

### Phase 7: Repositories with Complex Dependencies
*These require multiple higher-level crates*

```
23. copilot-agent (http, auth, vector)
24. incident-manager (otel, mq, audit)
25. orchestrator (otel, http, mq, id)
26. inference-gateway (otel, http, router)
```

---

## 6. WASM and TypeScript SDK Compatibility

### 6.1 WASM Compatibility Matrix

| Crate | Has WASM Feature | Build Status | Notes |
|-------|-----------------|--------------|-------|
| `infra-errors` | Yes | Fully Compatible | JS error interface via `JsInfraError` |
| `infra-id` | Yes | Fully Compatible | `getrandom/js` for WASM randomness |
| `infra-crypto` | Yes | Fully Compatible | `getrandom/js` for WASM randomness |
| `infra-json` | Yes | Fully Compatible | `serde-wasm-bindgen` for JS interop |
| `infra-config` | Yes | Partial | `regex` may cause WASM size issues |
| `infra-schema` | Yes | Partial | `jsonschema` has native dependencies |
| `infra-vector` | Yes | Partial | Needs verification with large vectors |
| `infra-auth` | Yes | Partial | JWT works, OAuth may need native |
| `infra-http` | Yes | Partial | Client works, server is native-only |
| `infra-otel` | No | Not Compatible | OpenTelemetry requires native runtime |
| `infra-router` | No | Not Compatible | Depends on infra-otel, server features |
| `infra-fs` | No | Not Compatible | Filesystem is native-only |
| `infra-mq` | No | Not Compatible | Message queues need async runtime |
| `infra-audit` | No | Not Compatible | No WASM feature implemented |
| `infra-sim` | No | Not Compatible | Testing utilities are native-only |

### 6.2 TypeScript SDK Integration

The TypeScript SDK at `/workspaces/infra/sdk/ts` provides:

**Native TypeScript Implementations:**
- `crypto/` - SHA256/384/512, AES-256-GCM via Web Crypto API
- `id/` - UUID v4/v7, ULID, NanoID, Snowflake
- `json/` - Parsing, path queries, diff/merge
- `errors.ts` - Error handling with Result types

**WASM Integration Points:**
- Blake3 hashing (requires WASM, not available in native TS)
- Performance-critical vector operations

**SDK Package Exports:**
```json
{
  ".": "./dist/index.js",
  "./crypto": "./dist/crypto/index.js",
  "./id": "./dist/id/index.js",
  "./json": "./dist/json/index.js"
}
```

### 6.3 WASM Build Requirements

For repositories requiring WASM support:

```toml
# In repository Cargo.toml
[dependencies]
infra-errors = { version = "0.1", features = ["wasm"] }
infra-crypto = { version = "0.1", features = ["wasm"] }
infra-json = { version = "0.1", features = ["wasm"] }
infra-id = { version = "0.1", features = ["wasm"] }

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
getrandom = { version = "0.2", features = ["js"] }
```

---

## 7. Copy-Paste Ready Cargo.toml Patches

### 7.1 Universal Base Patch (All Repositories)

All 26 repositories require this base configuration:

```toml
# Add to [dependencies] section
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-config = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-json = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
```

### 7.2 Repository-Specific Patches

#### test-bench
```toml
[dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-config = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-json = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-sim = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
```

#### observatory
```toml
[dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-config = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-json = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-otel = { git = "https://github.com/llm-dev-ops/infra", version = "0.1", features = ["otlp"] }
```

#### shield
```toml
[dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-config = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-json = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-auth = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-crypto = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
```

#### sentinel
```toml
[dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-config = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-json = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-otel = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-audit = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
```

#### memory-graph
```toml
[dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-config = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-json = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-vector = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
```

#### latency-lens
```toml
[dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-config = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-json = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-otel = { git = "https://github.com/llm-dev-ops/infra", version = "0.1", features = ["metrics"] }
```

#### forge
```toml
[dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-config = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-json = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-http = { git = "https://github.com/llm-dev-ops/infra", version = "0.1", features = ["client"] }
```

#### edge-agent
```toml
[dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-config = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-json = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-http = { git = "https://github.com/llm-dev-ops/infra", version = "0.1", features = ["client", "server"] }
infra-auth = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
```

#### auto-optimizer
```toml
[dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-config = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-json = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-vector = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
```

#### incident-manager
```toml
[dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-config = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-json = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-otel = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-mq = { git = "https://github.com/llm-dev-ops/infra", version = "0.1", features = ["memory"] }
infra-audit = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
```

#### orchestrator
```toml
[dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-config = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-json = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-otel = { git = "https://github.com/llm-dev-ops/infra", version = "0.1", features = ["otlp"] }
infra-http = { git = "https://github.com/llm-dev-ops/infra", version = "0.1", features = ["client"] }
infra-mq = { git = "https://github.com/llm-dev-ops/infra", version = "0.1", features = ["memory"] }
infra-id = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
```

#### cost-ops
```toml
[dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-config = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-json = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-audit = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
```

#### governance-dashboard
```toml
[dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-config = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-json = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-auth = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-audit = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
```

#### policy-engine
```toml
[dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-config = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-json = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-schema = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-auth = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
```

#### registry
```toml
[dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-config = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-json = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-fs = { git = "https://github.com/llm-dev-ops/infra", version = "0.1", features = ["async"] }
infra-schema = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
```

#### marketplace
```toml
[dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-config = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-json = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-http = { git = "https://github.com/llm-dev-ops/infra", version = "0.1", features = ["server"] }
infra-auth = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
```

#### analytics-hub
```toml
[dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-config = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-json = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-otel = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-vector = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
```

#### config-manager
```toml
[dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-config = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-json = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-fs = { git = "https://github.com/llm-dev-ops/infra", version = "0.1", features = ["async", "watch"] }
```

#### schema-registry
```toml
[dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-config = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-json = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-schema = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
```

#### connector-hub
```toml
[dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-config = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-json = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-http = { git = "https://github.com/llm-dev-ops/infra", version = "0.1", features = ["client"] }
infra-mq = { git = "https://github.com/llm-dev-ops/infra", version = "0.1", features = ["memory"] }
```

#### copilot-agent
```toml
[dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-config = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-json = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-http = { git = "https://github.com/llm-dev-ops/infra", version = "0.1", features = ["client"] }
infra-auth = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-vector = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
```

#### simulator
```toml
[dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-config = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-json = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-sim = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
```

#### benchmark-exchange
```toml
[dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-config = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-json = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-otel = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-sim = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
```

#### inference-gateway
```toml
[dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-config = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-json = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-otel = { git = "https://github.com/llm-dev-ops/infra", version = "0.1", features = ["otlp"] }
infra-http = { git = "https://github.com/llm-dev-ops/infra", version = "0.1", features = ["server"] }
infra-router = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
```

#### data-vault
```toml
[dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-config = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-json = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-crypto = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-fs = { git = "https://github.com/llm-dev-ops/infra", version = "0.1", features = ["async"] }
infra-audit = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
```

#### research-lab
```toml
[dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-config = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-json = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-vector = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
infra-sim = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
```

### 7.3 Standard Import Paths

After adding dependencies, use these standard import paths:

```rust
// Error handling
use infra_errors::{InfraError, InfraResult};

// Configuration
use infra_config::{Config, ConfigLoader, ConfigValue};

// JSON operations
use infra_json::{Json, JsonDiff};

// ID generation
use infra_id::{uuid_v7, ulid, snowflake};

// Cryptography
use infra_crypto::{Sha256Hasher, Aes256GcmCipher, Ed25519Signer};

// HTTP
use infra_http::{HttpClient, Request, Response};

// OpenTelemetry
use infra_otel::{init_otel, OtelConfig, OtelGuard};

// Vector operations
use infra_vector::{VectorStore, SearchResult};

// Authentication
use infra_auth::{JwtValidator, ApiKeyValidator};

// File system
use infra_fs::{FileSystem, LocalFs};

// Message queue
use infra_mq::{MessageQueue, MemoryQueue};

// Schema validation
use infra_schema::{SchemaValidator, ValidationResult};

// Audit logging
use infra_audit::{AuditLog, SecurityEvent};

// Testing
use infra_sim::{MockHttpClient, MockVectorStore};

// Routing
use infra_router::{Router, LoadBalancer};
```

---

## 8. Risk Assessment

### 8.1 High Risk Items

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Version mismatch across repos | Build failures | Medium | Pin to specific git commit hash |
| Breaking changes in infra crates | Cascading failures | Low | Semantic versioning, changelogs |
| Circular dependency introduced | Compile failure | Low | CI dependency checker |
| WASM build failures | SDK broken | Medium | WASM CI stage, feature flags |

### 8.2 Medium Risk Items

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Feature flag incompatibility | Conditional failures | Medium | Document all feature combinations |
| OpenTelemetry version conflicts | Tracing broken | Low | Centralize OTEL version in workspace |
| Async runtime conflicts | Runtime errors | Low | Standardize on tokio 1.x |

### 8.3 Low Risk Items

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Documentation gaps | Developer confusion | High | Require docs for all public APIs |
| Test coverage regression | Quality decrease | Medium | 80% coverage gate in CI |

---

## 9. Implementation Checklist

### 9.1 Pre-Implementation

- [ ] Review this integration plan with team
- [ ] Verify all 15 infra crates build successfully
- [ ] Run full test suite: `cargo test --all-features`
- [ ] Build WASM targets: `./test_wasm_builds.sh`
- [ ] Confirm TypeScript SDK builds: `cd sdk/ts && npm run build`

### 9.2 Per-Repository Integration

For each repository (in the order specified in Section 5):

- [ ] Create feature branch: `feature/infra-integration`
- [ ] Add dependencies to `Cargo.toml`
- [ ] Run `cargo check` to verify compilation
- [ ] Run `cargo test` to verify no regressions
- [ ] Update imports where applicable
- [ ] Create PR with integration checklist

### 9.3 Post-Integration Verification

- [ ] All 26 repositories build successfully
- [ ] Integration tests pass
- [ ] No circular dependency warnings
- [ ] WASM builds for applicable crates
- [ ] Documentation updated

---

## Appendix A: Quick Reference

### Crate Purpose Quick Reference

| Crate | One-Line Description |
|-------|---------------------|
| `infra-errors` | Unified `InfraError` type with retry logic |
| `infra-config` | Hierarchical config with hot-reload |
| `infra-json` | JSON ops, path queries, diff/merge |
| `infra-id` | UUID v7, ULID, Snowflake generation |
| `infra-crypto` | AES-GCM, Ed25519, JWT, hashing |
| `infra-otel` | OpenTelemetry 0.27 tracing/metrics |
| `infra-http` | HTTP client (reqwest) and server (axum) |
| `infra-fs` | Filesystem abstraction (local, S3) |
| `infra-schema` | JSON Schema validation |
| `infra-mq` | Message queue (memory, Redis) |
| `infra-audit` | Security audit logging |
| `infra-sim` | Testing mocks and utilities |
| `infra-vector` | Vector operations for embeddings |
| `infra-auth` | JWT, API key, OAuth authentication |
| `infra-router` | Load balancing and routing |

### Repository Category Quick Reference

| Category | Repositories |
|----------|--------------|
| Core | orchestrator, forge, config-manager |
| Gateway | edge-agent, inference-gateway, marketplace |
| Data | memory-graph, registry, data-vault, analytics-hub |
| AI/ML | auto-optimizer, copilot-agent, research-lab |
| Monitoring | observatory, sentinel, latency-lens |
| Security | shield, governance-dashboard, policy-engine |
| Operations | incident-manager, cost-ops |
| Messaging | connector-hub |
| Utilities | schema-registry |
| Testing | test-bench, simulator, benchmark-exchange |

---

**Document Status:** Complete - Ready for Review and Approval

**Next Steps:**
1. Review this document with engineering team
2. Approve integration order and patches
3. Begin Phase 1 implementation
4. Track progress against checklist
