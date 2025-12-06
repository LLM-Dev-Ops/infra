# Automation Integration Plan: LLM-Dev-Ops Infra Crates

**Generated:** 2025-12-06
**Revised:** 2025-12-06 (v2.0 - Safety Enhanced)
**Scope:** Integration of 15 infra crates into 26 LLM-Dev-Ops repositories
**Status:** ANALYSIS COMPLETE - Awaiting Review and Approval
**SPARC Compliance:** Aligned with SPARC Phases 1-5 as architectural source of truth

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
10. [Workspace Safety Review](#10-workspace-safety-review) *(NEW)*
11. [Dependency Reconciliation](#11-dependency-reconciliation) *(NEW)*
12. [Enhanced WASM/TypeScript Compatibility](#12-enhanced-wasmtypescript-compatibility) *(NEW)*
13. [Post-Integration Smoke Test Workflow](#13-post-integration-smoke-test-workflow) *(NEW)*
14. [Rollback and Isolation Strategy](#14-rollback-and-isolation-strategy) *(NEW)*
15. [Documentation Requirements](#15-documentation-requirements) *(NEW)*

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
5. **Workspace Isolation**: Each repository maintains independent workspace boundaries
6. **Version Consistency**: All shared dependencies resolve to compatible versions

### Safety Principles (v2.0)

This revised plan incorporates the following safety enhancements:

- **Workspace Safety Review**: Validates each repo can accept infra dependencies without conflicts
- **Dependency Reconciliation**: Ensures version consistency across all 26 repositories
- **WASM/TS Compatibility Analysis**: Verifies build toolchain requirements
- **Smoke Test Workflow**: Mandates verification after each integration
- **Rollback Strategy**: Provides isolation and recovery procedures
- **Documentation Standards**: Requires usage documentation per repo

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

## 10. Workspace Safety Review

### 10.1 Purpose

This section ensures that each of the 26 LLM-Dev-Ops repositories can safely accept the planned infra dependencies without causing cross-workspace conflicts, Cargo.toml structural issues, or sub-crate configuration problems.

### 10.2 Workspace Configuration Requirements

Each target repository MUST satisfy these workspace safety criteria before integration:

#### 10.2.1 Cargo Workspace Structure Validation

```toml
# Each repo must have a root Cargo.toml with proper workspace config
[workspace]
resolver = "2"  # Required for proper feature resolution
members = [
    # All sub-crates explicitly listed
]

[workspace.dependencies]
# Shared dependencies should be declared here
```

**Validation Checklist per Repository:**

| Repository | Has Workspace | Resolver 2 | Members Explicit | Safe for Integration |
|------------|---------------|------------|------------------|----------------------|
| test-bench | - | - | - | Pending |
| observatory | - | - | - | Pending |
| shield | - | - | - | Pending |
| sentinel | - | - | - | Pending |
| memory-graph | - | - | - | Pending |
| latency-lens | - | - | - | Pending |
| forge | - | - | - | Pending |
| edge-agent | - | - | - | Pending |
| auto-optimizer | - | - | - | Pending |
| incident-manager | - | - | - | Pending |
| orchestrator | - | - | - | Pending |
| cost-ops | - | - | - | Pending |
| governance-dashboard | - | - | - | Pending |
| policy-engine | - | - | - | Pending |
| registry | - | - | - | Pending |
| marketplace | - | - | - | Pending |
| analytics-hub | - | - | - | Pending |
| config-manager | - | - | - | Pending |
| schema-registry | - | - | - | Pending |
| connector-hub | - | - | - | Pending |
| copilot-agent | - | - | - | Pending |
| simulator | - | - | - | Pending |
| benchmark-exchange | - | - | - | Pending |
| inference-gateway | - | - | - | Pending |
| data-vault | - | - | - | Pending |
| research-lab | - | - | - | Pending |

*Note: "-" indicates validation required before integration*

#### 10.2.2 Cross-Workspace Conflict Prevention

To prevent cross-workspace conflicts when integrating infra crates:

1. **No Path Dependencies Crossing Workspaces**
   ```toml
   # WRONG - Do not use absolute paths to other workspaces
   infra-errors = { path = "/absolute/path/to/infra/crates/infra-errors" }

   # CORRECT - Use git dependencies
   infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
   ```

2. **Workspace Inheritance Must Be Local**
   ```toml
   # CORRECT - Inherit from local workspace
   [package]
   version.workspace = true  # Only if defined in same workspace root

   # For infra dependencies, use explicit versions
   infra-errors = { git = "...", version = "0.1" }
   ```

3. **Feature Unification Boundaries**
   - Each repository workspace handles its own feature unification
   - Infra crate features are resolved within the infra workspace
   - No feature flags should cross workspace boundaries

### 10.3 Sub-Crate Configuration Safety

For repositories with multiple internal crates (workspace members):

#### 10.3.1 Dependency Placement Rules

```
Repository Root/
├── Cargo.toml (workspace root)
│   └── [workspace.dependencies] ← Declare infra crates here
├── crates/
│   ├── core/
│   │   └── Cargo.toml ← Reference via `infra-errors.workspace = true`
│   └── api/
│       └── Cargo.toml ← Reference via `infra-errors.workspace = true`
```

**Pattern A: Centralized Declaration (Recommended)**
```toml
# Root Cargo.toml
[workspace.dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }

# Sub-crate Cargo.toml
[dependencies]
infra-errors.workspace = true
```

**Pattern B: Direct Declaration (Fallback)**
```toml
# Sub-crate Cargo.toml (when no workspace.dependencies)
[dependencies]
infra-errors = { git = "https://github.com/llm-dev-ops/infra", version = "0.1" }
```

#### 10.3.2 Pre-Integration Validation Script

Run this script before integrating each repository:

```bash
#!/bin/bash
# validate-workspace-safety.sh

REPO=$1

echo "Validating workspace safety for: $REPO"

# Check for Cargo.toml
if [ ! -f "$REPO/Cargo.toml" ]; then
    echo "ERROR: No Cargo.toml found"
    exit 1
fi

# Check resolver version
if grep -q 'resolver = "2"' "$REPO/Cargo.toml"; then
    echo "✓ Resolver 2 configured"
else
    echo "WARNING: Resolver 2 not found - recommend adding"
fi

# Check for workspace.dependencies section
if grep -q '\[workspace\.dependencies\]' "$REPO/Cargo.toml"; then
    echo "✓ workspace.dependencies section exists"
else
    echo "INFO: No workspace.dependencies - will use direct declarations"
fi

# Check for conflicting infra crate declarations
if grep -rq 'infra-errors\|infra-config\|infra-json' "$REPO/Cargo.toml" "$REPO/crates/*/Cargo.toml" 2>/dev/null; then
    echo "WARNING: Existing infra crate references found - manual review required"
fi

echo "Validation complete for $REPO"
```

### 10.4 Workspace Safety Summary Matrix

| Safety Check | Required | Impact if Missing |
|--------------|----------|-------------------|
| Resolver 2 | Recommended | Feature unification issues |
| Explicit members | Required | Build failures |
| No absolute paths | Required | Cross-workspace conflicts |
| Centralized dependencies | Recommended | Version inconsistencies |
| No existing infra refs | Required | Duplicate dependency conflicts |

---

## 11. Dependency Reconciliation

### 11.1 Purpose

This section ensures all shared external crates (tokio, serde, hyper, async-trait, wasm-bindgen, etc.) resolve to consistent versions across all 26 repositories to prevent version mismatch or duplicate dependency failures.

### 11.2 Canonical Dependency Versions (Source of Truth)

The following versions are defined in the infra workspace and MUST be compatible with all target repositories:

```toml
# From llm-dev-ops-infra/Cargo.toml (workspace root)

[workspace.dependencies]
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
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-opentelemetry = "0.27"

# HTTP
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
hyper = { version = "1.0", features = ["full"] }
hyper-util = { version = "0.1", features = ["full"] }
http-body-util = "0.1"
tower = "0.4"

# Crypto
sha2 = "0.10"
blake3 = "1.5"
aes-gcm = "0.10"
argon2 = "0.5"
ed25519-dalek = { version = "2.1", features = ["rand_core"] }
jsonwebtoken = "9.2"
rand = "0.8"
base64 = "0.21"
hex = "0.4"
constant_time_eq = "0.3"

# Utilities
uuid = { version = "1.6", features = ["v4", "v7"] }
ulid = "1.1"
chrono = { version = "0.4", features = ["serde"] }
url = "2.5"
bytes = "1.5"
parking_lot = "0.12"
dashmap = "5.5"
crossbeam-channel = "0.5"

# WASM
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
web-sys = { version = "0.3", features = ["console", "Window", "Request", "RequestInit", "Response", "Headers"] }
serde-wasm-bindgen = "0.6"
getrandom = { version = "0.2", features = ["js"] }

# Testing
tokio-test = "0.4"
```

### 11.3 Version Compatibility Requirements

Each target repository MUST ensure its existing dependencies are compatible with these version ranges:

#### 11.3.1 Critical Compatibility Matrix

| Dependency | Infra Version | Compatible Range | Breaking If |
|------------|---------------|------------------|-------------|
| `tokio` | 1.35 | ^1.21 | < 1.21 |
| `serde` | 1.0 | ^1.0 | 0.x |
| `serde_json` | 1.0 | ^1.0 | 0.x |
| `hyper` | 1.0 | ^1.0 | 0.14.x |
| `reqwest` | 0.12 | ^0.12 | < 0.12 |
| `async-trait` | 0.1 | ^0.1 | N/A |
| `thiserror` | 1.0 | ^1.0 | 0.x |
| `tracing` | 0.1 | ^0.1 | N/A |
| `opentelemetry` | 0.27 | ^0.27 | < 0.27 |
| `wasm-bindgen` | 0.2 | ^0.2 | N/A |
| `chrono` | 0.4 | ^0.4 | N/A |
| `uuid` | 1.6 | ^1.0 | 0.x |

#### 11.3.2 Known Breaking Changes to Watch

1. **hyper 0.14.x → 1.0**: Major API changes
   - Repos using hyper 0.14 must upgrade before integration
   - `infra-http` uses hyper 1.0 exclusively

2. **reqwest < 0.12 → 0.12**: TLS backend changes
   - Repos using reqwest 0.11 should upgrade
   - `rustls-tls` is the default in infra

3. **opentelemetry < 0.27 → 0.27**: API restructuring
   - Repos using older OTEL versions must upgrade
   - `infra-otel` standardizes on 0.27

### 11.4 Pre-Integration Dependency Audit

Run this audit for each repository before integration:

```bash
#!/bin/bash
# audit-dependencies.sh

REPO=$1

echo "=== Dependency Audit for $REPO ==="

cd "$REPO"

# Check for incompatible tokio
TOKIO_VER=$(cargo metadata --format-version=1 2>/dev/null | jq -r '.packages[] | select(.name == "tokio") | .version' | head -1)
if [ -n "$TOKIO_VER" ]; then
    echo "tokio: $TOKIO_VER"
    if [[ "$TOKIO_VER" < "1.21" ]]; then
        echo "  WARNING: tokio < 1.21 may cause conflicts"
    fi
fi

# Check for incompatible hyper
HYPER_VER=$(cargo metadata --format-version=1 2>/dev/null | jq -r '.packages[] | select(.name == "hyper") | .version' | head -1)
if [ -n "$HYPER_VER" ]; then
    echo "hyper: $HYPER_VER"
    if [[ "$HYPER_VER" == 0.14* ]]; then
        echo "  WARNING: hyper 0.14.x incompatible with infra-http (requires 1.0)"
    fi
fi

# Check for incompatible opentelemetry
OTEL_VER=$(cargo metadata --format-version=1 2>/dev/null | jq -r '.packages[] | select(.name == "opentelemetry") | .version' | head -1)
if [ -n "$OTEL_VER" ]; then
    echo "opentelemetry: $OTEL_VER"
    if [[ "$OTEL_VER" != 0.27* ]]; then
        echo "  WARNING: opentelemetry != 0.27 may cause conflicts with infra-otel"
    fi
fi

# Check for duplicate versions
echo ""
echo "Checking for potential duplicates..."
cargo tree -d 2>/dev/null | head -20

echo ""
echo "=== Audit Complete ==="
```

### 11.5 Dependency Reconciliation Checklist

Before integrating each repository:

- [ ] Run dependency audit script
- [ ] Verify no hyper 0.14.x usage (upgrade to 1.0)
- [ ] Verify no opentelemetry < 0.27 usage (upgrade or don't use infra-otel)
- [ ] Verify tokio >= 1.21
- [ ] Check `cargo tree -d` for no critical duplicates
- [ ] Update incompatible dependencies in target repo first

### 11.6 Resolution Strategy for Conflicts

If version conflicts are detected:

**Strategy 1: Upgrade Target Repository**
```toml
# Update target repo's Cargo.toml to match infra versions
[dependencies]
tokio = "1.35"  # Align with infra
hyper = "1.0"   # Upgrade from 0.14
```

**Strategy 2: Use Cargo Patch (Temporary)**
```toml
# In target repo's Cargo.toml (temporary measure)
[patch.crates-io]
# Force specific version resolution
tokio = { version = "1.35" }
```

**Strategy 3: Feature Gating (Last Resort)**
```toml
# In target repo - disable conflicting infra features
infra-http = { git = "...", version = "0.1", default-features = false }
```

---

## 12. Enhanced WASM/TypeScript Compatibility

### 12.1 Purpose

This section provides detailed analysis for repositories expected to consume WASM-enabled infra crates, ensuring build toolchains, feature gating, and CI constraints are correctly handled.

### 12.2 Repositories Requiring WASM Support

Based on functional requirements, the following repositories may need WASM-enabled infra crates:

| Repository | WASM Crates Needed | Use Case |
|------------|-------------------|----------|
| copilot-agent | infra-crypto, infra-json, infra-errors | Browser client |
| edge-agent | infra-crypto, infra-json, infra-auth | Edge runtime |
| governance-dashboard | infra-json, infra-errors | Web UI |
| marketplace | infra-json, infra-auth | Web frontend |
| analytics-hub | infra-json, infra-vector | Browser visualization |

### 12.3 WASM Build Toolchain Requirements

#### 12.3.1 Required Tools

```bash
# Required for WASM builds
rustup target add wasm32-unknown-unknown

# For WASM packaging
cargo install wasm-pack

# For WASM optimization (optional)
cargo install wasm-opt
```

#### 12.3.2 Repository Configuration for WASM

Repositories consuming WASM crates must add:

```toml
# In repository Cargo.toml

[lib]
crate-type = ["cdylib", "rlib"]  # Required for WASM output

[features]
default = ["native"]
native = []
wasm = [
    "infra-errors/wasm",
    "infra-json/wasm",
    "infra-crypto/wasm",
    # Add other wasm features as needed
]

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
getrandom = { version = "0.2", features = ["js"] }
```

### 12.4 Feature Gating Strategy

#### 12.4.1 Infra Crate Feature Flags

| Crate | Native Features | WASM Features | Exclusive Features |
|-------|-----------------|---------------|-------------------|
| `infra-errors` | `std` | `wasm` | None |
| `infra-json` | `std` | `wasm` | None |
| `infra-crypto` | `std` | `wasm` | None |
| `infra-id` | `std` | `wasm` | None |
| `infra-config` | `std` | `wasm` | None |
| `infra-http` | `client`, `server` | `wasm` | `server` (native only) |
| `infra-auth` | `axum` | `wasm` | `axum` (native only) |
| `infra-vector` | (default) | `wasm` | None |

#### 12.4.2 Conditional Compilation Pattern

```rust
// In consuming repository code

#[cfg(not(target_arch = "wasm32"))]
use infra_http::server::HttpServer;

#[cfg(target_arch = "wasm32")]
use infra_http::client::WasmHttpClient;

// Shared code
use infra_errors::{InfraError, InfraResult};
use infra_json::Json;
```

### 12.5 CI Configuration for WASM Builds

#### 12.5.1 GitHub Actions Workflow Addition

Add to repositories requiring WASM support:

```yaml
# .github/workflows/wasm.yml
name: WASM Build

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  wasm-build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Install wasm-pack
        run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

      - name: Build WASM
        run: wasm-pack build --target web --features wasm

      - name: Run WASM tests
        run: wasm-pack test --headless --chrome --features wasm
```

### 12.6 TypeScript SDK Integration

For repositories that also need TypeScript SDK integration:

#### 12.6.1 Package.json Dependencies

```json
{
  "dependencies": {
    "@llm-dev-ops/infra": "^0.1.0"
  },
  "devDependencies": {
    "@types/node": "^20.0.0",
    "typescript": "^5.3.0"
  }
}
```

#### 12.6.2 TypeScript Usage

```typescript
// Using WASM-powered infra in TypeScript
import { sha256, hashHex } from '@llm-dev-ops/infra/crypto';
import { parseJson, stringifyJson } from '@llm-dev-ops/infra/json';
import { InfraError } from '@llm-dev-ops/infra';

async function example() {
  const hash = await hashHex('hello world', 'sha256');
  const json = parseJson('{"key": "value"}');
}
```

### 12.7 WASM Compatibility Verification Checklist

Before enabling WASM features in a repository:

- [ ] Verify wasm32-unknown-unknown target installed
- [ ] Verify wasm-pack installed
- [ ] Add `crate-type = ["cdylib", "rlib"]` to lib section
- [ ] Add WASM feature flag with proper dependencies
- [ ] Verify no native-only code in WASM paths
- [ ] Add WASM build to CI pipeline
- [ ] Test in browser environment
- [ ] Verify bundle size is acceptable (< 500KB per module)

---

## 13. Post-Integration Smoke Test Workflow

### 13.1 Purpose

This section mandates a verification workflow that MUST be executed immediately after dependencies are added to each repository. This ensures the integration is successful before proceeding to the next repository.

### 13.2 Mandatory Smoke Test Sequence

After adding infra dependencies to any repository, execute this sequence **in order**:

```bash
#!/bin/bash
# smoke-test.sh - Run after each repository integration

set -e  # Exit on any failure

REPO_NAME=$1
echo "===== Smoke Test for $REPO_NAME ====="

# Step 1: Clean previous artifacts
echo "Step 1/6: Cleaning..."
cargo clean

# Step 2: Verify dependency resolution
echo "Step 2/6: Checking dependencies..."
cargo check --all-targets
if [ $? -ne 0 ]; then
    echo "FAIL: cargo check failed"
    exit 1
fi
echo "✓ cargo check passed"

# Step 3: Full build
echo "Step 3/6: Building..."
cargo build --all-targets
if [ $? -ne 0 ]; then
    echo "FAIL: cargo build failed"
    exit 1
fi
echo "✓ cargo build passed"

# Step 4: Run tests
echo "Step 4/6: Running tests..."
cargo test --all-targets
if [ $? -ne 0 ]; then
    echo "FAIL: cargo test failed"
    exit 1
fi
echo "✓ cargo test passed"

# Step 5: WASM build (if applicable)
if grep -q 'wasm' Cargo.toml; then
    echo "Step 5/6: Building WASM..."
    cargo build --target wasm32-unknown-unknown --features wasm 2>/dev/null || echo "INFO: WASM build skipped (not configured)"
else
    echo "Step 5/6: WASM build skipped (not needed)"
fi

# Step 6: TypeScript build (if applicable)
if [ -d "ts" ] || [ -d "typescript" ] || [ -f "package.json" ]; then
    echo "Step 6/6: Building TypeScript..."
    npm install && npm run build 2>/dev/null || echo "INFO: TS build skipped (not configured)"
else
    echo "Step 6/6: TypeScript build skipped (not needed)"
fi

echo ""
echo "===== Smoke Test PASSED for $REPO_NAME ====="
```

### 13.3 CI Integration

Add to each repository's CI pipeline:

```yaml
# .github/workflows/ci.yml - Add this job

  infra-integration-test:
    name: Infra Integration Smoke Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache Cargo
        uses: Swatinem/rust-cache@v2

      - name: Run Smoke Test
        run: |
          cargo check --all-targets
          cargo build --all-targets
          cargo test --all-targets
```

### 13.4 Smoke Test Result Documentation

After each integration, document the result:

| Repository | Date | Check | Build | Test | WASM | TS | Status |
|------------|------|-------|-------|------|------|-----|--------|
| config-manager | - | - | - | - | N/A | N/A | Pending |
| schema-registry | - | - | - | - | N/A | N/A | Pending |
| cost-ops | - | - | - | - | N/A | N/A | Pending |
| test-bench | - | - | - | - | N/A | N/A | Pending |
| ... | ... | ... | ... | ... | ... | ... | ... |

*Legend: ✓ = Pass, ✗ = Fail, N/A = Not Applicable, - = Not Run*

### 13.5 Failure Response Protocol

If any smoke test step fails:

1. **DO NOT** proceed to next repository
2. **DO NOT** merge the integration PR
3. **DO** investigate the failure cause
4. **DO** check rollback procedures (Section 14)
5. **DO** document the failure and resolution

---

## 14. Rollback and Isolation Strategy

### 14.1 Purpose

This section provides isolation and recovery procedures so that failed integrations do not block or break other repositories.

### 14.2 Isolation Principles

1. **Each repository is integrated independently**
   - No repository integration depends on another completing
   - Failures in one repo do not block others

2. **Feature branches for all integrations**
   - Never integrate directly to main
   - All work done in `feature/infra-integration` branches

3. **Atomic commits**
   - Single commit for dependency additions
   - Separate commits for code changes using infra crates

### 14.3 Rollback Procedures

#### 14.3.1 Quick Rollback (Before Merge)

If integration fails before PR merge:

```bash
# Discard all changes and return to clean state
git checkout main
git branch -D feature/infra-integration
```

#### 14.3.2 Full Rollback (After Merge)

If integration causes issues after merge to main:

```bash
# Step 1: Identify the integration commit
COMMIT=$(git log --oneline | grep "infra" | head -1 | cut -d' ' -f1)

# Step 2: Create rollback branch
git checkout -b rollback/infra-integration

# Step 3: Revert the integration commit
git revert $COMMIT --no-commit

# Step 4: Remove infra dependencies from Cargo.toml
# (manual edit or automated script)

# Step 5: Verify clean build
cargo clean
cargo check
cargo test

# Step 6: Commit and create PR
git commit -m "Rollback: Remove infra crate integration due to [reason]"
git push origin rollback/infra-integration
```

#### 14.3.3 Partial Rollback (Specific Crates)

If only certain infra crates cause issues:

```toml
# Before: Full integration
[dependencies]
infra-errors = { git = "...", version = "0.1" }
infra-config = { git = "...", version = "0.1" }
infra-otel = { git = "...", version = "0.1" }  # Problematic

# After: Remove problematic crate
[dependencies]
infra-errors = { git = "...", version = "0.1" }
infra-config = { git = "...", version = "0.1" }
# infra-otel removed due to [issue]
```

### 14.4 Isolation Strategies

#### 14.4.1 Optional Dependencies Pattern

For non-critical infra crates, use optional dependencies:

```toml
[dependencies]
infra-errors = { git = "...", version = "0.1" }  # Required
infra-config = { git = "...", version = "0.1" }  # Required
infra-otel = { git = "...", version = "0.1", optional = true }  # Optional

[features]
default = []
observability = ["infra-otel"]
```

This allows:
- Basic integration without optional features
- Easy disable if specific crates cause issues
- Gradual adoption of additional crates

#### 14.4.2 Facade Pattern for Isolation

Create internal facade modules that wrap infra crates:

```rust
// src/infra_facade.rs

// Re-export with isolation layer
pub use infra_errors::{InfraError, InfraResult};

// Facade for config - can swap implementation
#[cfg(feature = "use-infra-config")]
pub use infra_config::{Config, ConfigLoader};

#[cfg(not(feature = "use-infra-config"))]
pub mod config {
    // Fallback implementation
    pub struct Config { /* ... */ }
}
```

Benefits:
- Single point to enable/disable infra usage
- Easy rollback without touching all code
- Clear migration path

### 14.5 Emergency Procedures

#### 14.5.1 Critical Bug in Infra Crate

If a critical bug is discovered in an infra crate:

1. **Immediate**: All repos pin to last known good commit
   ```toml
   infra-errors = { git = "...", rev = "abc123" }  # Pin to specific commit
   ```

2. **Short-term**: Fix deployed to infra repo
3. **Long-term**: Update all repos to new version

#### 14.5.2 Breaking Change in Infra Crate

If breaking change is released:

1. **Identify affected repos** using dependency search
2. **Create tracking issue** for each affected repo
3. **Update repos** in dependency-safe order (Section 5)
4. **Verify** with smoke tests after each update

### 14.6 Rollback Checklist

Before considering an integration complete:

- [ ] Feature branch created (not main)
- [ ] Clean revert point identified (commit hash)
- [ ] Smoke tests passing
- [ ] Rollback procedure documented
- [ ] Emergency contacts identified

---

## 15. Documentation Requirements

### 15.1 Purpose

This section establishes documentation standards requiring each repository to document which infra crates it consumes and how it uses them.

### 15.2 Required Documentation

Each integrated repository MUST include:

#### 15.2.1 INFRA_DEPENDENCIES.md

Create a file at repository root:

```markdown
# Infra Crate Dependencies

This document describes how this repository uses the LLM-Dev-Ops infra crates.

## Dependencies

| Crate | Version | Purpose in This Repo |
|-------|---------|---------------------|
| infra-errors | 0.1.0 | Unified error handling |
| infra-config | 0.1.0 | Configuration management |
| infra-json | 0.1.0 | JSON serialization |
| [additional crates] | ... | ... |

## Usage Examples

### Error Handling

```rust
use infra_errors::{InfraError, InfraResult};

fn example() -> InfraResult<()> {
    // ... code using InfraError
}
```

### Configuration

```rust
use infra_config::{Config, ConfigLoader};

fn load_config() -> Config {
    ConfigLoader::new()
        .with_file("config.toml")
        .load()
        .expect("Failed to load config")
}
```

## Feature Flags

| Feature | Enabled | Purpose |
|---------|---------|---------|
| wasm | No | Not required for this repo |
| otel | Yes | Observability support |

## Migration Notes

- Migrated from custom error types on [date]
- Previous config system replaced on [date]

## Contact

For questions about infra integration in this repo, contact: [team/person]
```

#### 15.2.2 README.md Updates

Add to repository README:

```markdown
## Infrastructure Dependencies

This repository uses the [LLM-Dev-Ops infra crates](https://github.com/llm-dev-ops/infra)
for common infrastructure functionality:

- Error handling (`infra-errors`)
- Configuration (`infra-config`)
- JSON operations (`infra-json`)
- [additional as applicable]

See [INFRA_DEPENDENCIES.md](./INFRA_DEPENDENCIES.md) for detailed usage documentation.
```

#### 15.2.3 Code Documentation

Add inline documentation where infra crates are used:

```rust
//! # Configuration Module
//!
//! This module uses `infra-config` for configuration management.
//! See INFRA_DEPENDENCIES.md for setup instructions.

use infra_config::{Config, ConfigLoader};

/// Loads application configuration from the standard location.
///
/// Uses `infra-config` for hierarchical configuration loading
/// with environment variable overlay.
///
/// # Example
///
/// ```
/// let config = load_app_config()?;
/// let port: u16 = config.require("server.port")?;
/// ```
pub fn load_app_config() -> infra_errors::InfraResult<Config> {
    ConfigLoader::new()
        .with_file("config.toml")
        .with_env_prefix("APP_")
        .load()
}
```

### 15.3 Documentation Checklist

For each integrated repository:

- [ ] INFRA_DEPENDENCIES.md created
- [ ] README.md updated with infra section
- [ ] Key functions using infra crates have doc comments
- [ ] Feature flags documented
- [ ] Migration notes recorded

### 15.4 Documentation Template

Copy this template when creating INFRA_DEPENDENCIES.md:

```markdown
# Infra Crate Dependencies

**Repository:** [repo-name]
**Integration Date:** [date]
**Integration Version:** infra v0.1.0

## Summary

[Brief description of how this repo uses infra crates]

## Dependencies

| Crate | Version | Purpose | Required |
|-------|---------|---------|----------|
| infra-errors | 0.1 | Error handling | Yes |
| infra-config | 0.1 | Configuration | Yes |
| infra-json | 0.1 | JSON ops | Yes |

## Configuration

[How to configure infra crates in this repo]

## Usage Patterns

### Pattern 1: [Name]

```rust
// Example code
```

### Pattern 2: [Name]

```rust
// Example code
```

## Troubleshooting

### Issue: [Common issue]

**Solution:** [How to resolve]

## Change Log

- [date]: Initial integration
- [date]: Added [crate] for [reason]
```

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

## Appendix B: SPARC Compliance Matrix

This integration plan is aligned with the SPARC specification documents:

| SPARC Phase | Document | Alignment |
|-------------|----------|-----------|
| Phase 1 | 01-specification.md | Crate catalog, dependency rules |
| Phase 2 | 02-pseudocode.md | API surface definitions |
| Phase 3 | 03-architecture.md | Workspace structure, CI/CD |
| Phase 4 | 04-refinement.md | Error handling, security |
| Phase 5 | 05-completion.md | Success metrics, roadmap |

---

**Document Status:** Complete v2.0 - Ready for Review and Approval

**Revision History:**
- v1.0 (2025-12-06): Initial integration plan
- v2.0 (2025-12-06): Safety enhancements added
  - Workspace safety review (Section 10)
  - Dependency reconciliation (Section 11)
  - Enhanced WASM/TS compatibility (Section 12)
  - Smoke test workflow (Section 13)
  - Rollback strategy (Section 14)
  - Documentation requirements (Section 15)

**Next Steps:**
1. Review this document with engineering team
2. Validate workspace safety for target repositories
3. Run dependency audits
4. Approve integration order and patches
5. Begin Phase 1 implementation with smoke tests
6. Track progress against checklists
