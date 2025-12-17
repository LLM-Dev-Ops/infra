# SPARC Specification: Infrastructure Hardening for Production Readiness

**Repository:** llm-dev-ops-infra
**Location:** /workspaces/infra
**Document Version:** 1.0.0
**Created:** 2025-12-17
**Status:** SPECIFICATION COMPLETE - Awaiting Review and Approval
**SPARC Phase:** Specification (S)

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Scope](#2-scope)
3. [Non-Goals](#3-non-goals)
4. [Constraints](#4-constraints)
5. [Current State Analysis](#5-current-state-analysis)
6. [Hardening Items](#6-hardening-items)
   - [H1: PostgreSQL Extensions (pgvector, pgcrypto)](#h1-postgresql-extensions-pgvector-pgcrypto)
   - [H2: Durable Data Persistence](#h2-durable-data-persistence)
   - [H3: Environment Variable Standardization](#h3-environment-variable-standardization)
   - [H4: Docker Image Version Pinning](#h4-docker-image-version-pinning)
   - [H5: Health Checks and Restart Policies](#h5-health-checks-and-restart-policies)
   - [H6: Docker Networks and Service Dependencies](#h6-docker-networks-and-service-dependencies)
   - [H7: Network Exposure Security](#h7-network-exposure-security)
   - [H8: Secrets Management](#h8-secrets-management)
   - [H9: Logging Configuration](#h9-logging-configuration)
   - [H10: Startup Order Guarantees](#h10-startup-order-guarantees)
   - [H11: README Documentation](#h11-readme-documentation)
7. [Implementation Priorities](#7-implementation-priorities)
8. [Risk Assessment](#8-risk-assessment)
9. [Appendix A: Verification Commands](#appendix-a-verification-commands)
10. [Appendix B: File Inventory](#appendix-b-file-inventory)

---

## 1. Executive Summary

This SPARC specification defines the exact steps required to harden the existing `llm-dev-ops-infra` repository for production readiness while **preserving all current behavior**. The hardening focuses exclusively on operational stability, security posture, and deployment reliability.

### Key Metrics

| Metric | Value |
|--------|-------|
| Hardening Items | 11 |
| Files Affected | ~8 |
| New Files Required | 0 |
| Behavior Changes | 0 |
| Breaking Changes | 0 |

### Critical Principle

**This specification is strictly additive and corrective.** No new services, features, abstractions, or architectural changes are permitted. All modifications must preserve existing functionality while improving operational characteristics.

---

## 2. Scope

This specification covers the following hardening objectives:

| ID | Objective | Category |
|----|-----------|----------|
| H1 | Enable and verify PostgreSQL extensions (pgvector, pgcrypto) via idempotent init scripts | Database |
| H2 | Enforce durable data persistence through named volumes for PostgreSQL and Redis | Storage |
| H3 | Standardize environment variable usage with no reliance on .env files | Configuration |
| H4 | Pin Docker image versions to specific digests or tags | Reproducibility |
| H5 | Add health checks and restart policies for all services | Reliability |
| H6 | Define explicit Docker networks and service dependencies | Networking |
| H7 | Ensure no services are publicly exposed by default | Security |
| H8 | Confirm secrets are never committed to the repository | Security |
| H9 | Route all logs to stdout/stderr with OpenTelemetry hooks documented | Observability |
| H10 | Clarify startup order guarantees | Operations |
| H11 | Document infrastructure usage and constraints in README | Documentation |

---

## 3. Non-Goals

The following are explicitly **out of scope** for this hardening effort:

| Non-Goal | Rationale |
|----------|-----------|
| Adding new services (e.g., observability stack, monitoring) | Scope limited to existing services |
| Implementing OpenTelemetry collectors or exporters | Only documentation of hooks is in scope |
| Creating Kubernetes manifests or Helm charts | Docker Compose only |
| Modifying Rust crate logic or APIs | Infrastructure hardening only |
| Adding SSL/TLS termination | Would require new infrastructure components |
| Implementing backup/restore automation | Requires external tooling |
| Multi-node or cluster configurations | Single-node deployment only |
| CI/CD pipeline modifications | Out of scope for infrastructure hardening |
| TypeScript SDK modifications | Covered by separate audit |
| Adding new environment variables | Only documenting existing ones |

---

## 4. Constraints

### 4.1 Technical Constraints

| Constraint | Description |
|------------|-------------|
| **Backward Compatibility** | All existing scripts, compose commands, and workflows must continue to work |
| **No New Dependencies** | No additional services or images may be introduced |
| **Idempotency** | All initialization scripts must be safe to re-run |
| **Shell-Export Pattern** | Environment configuration must use shell exports, not .env files |
| **Minimal Changes** | Prefer correcting existing files over creating new ones |

### 4.2 Operational Constraints

| Constraint | Description |
|------------|-------------|
| **Zero Downtime** | Changes must not require migration of existing data |
| **Local Development** | Must remain fully functional for local development |
| **Docker Compose Only** | No orchestration beyond docker compose |
| **Single Host** | Designed for single-host deployments |

---

## 5. Current State Analysis

### 5.1 Existing Infrastructure

Based on repository analysis, the current infrastructure includes:

**docker-compose.yml:**
- PostgreSQL 16 with pgvector extension (`pgvector/pgvector:pg16`)
- Redis 7 Alpine (`redis:7-alpine`)
- Named volumes for data persistence
- Custom bridge network
- Health checks for both services
- Restart policy: `unless-stopped`

**Initialization Scripts:**
- `docker/postgres/init/01-init-pgvector.sql` - Creates extensions and schema
- `docker/postgres/init/99-verify-extensions.sql` - Verifies installation

**Environment Scripts:**
- `scripts/env-setup.sh` - Exports environment variables
- `scripts/verify-postgres-extensions.sh` - Validates PostgreSQL extensions
- `scripts/verify-env-config.sh` - Validates environment configuration

**Documentation:**
- `.env.example` - Documents variables (explicitly not loaded)
- `README.md` - General repository documentation

### 5.2 Current Compliance Status

| Hardening Item | Current Status | Gap |
|----------------|----------------|-----|
| H1: PostgreSQL Extensions | PARTIAL | Need to add pgcrypto verification to 99-verify |
| H2: Named Volumes | COMPLIANT | Already implemented |
| H3: No .env Reliance | COMPLIANT | Shell exports documented |
| H4: Image Pinning | PARTIAL | Using tags, not digests |
| H5: Health Checks | COMPLIANT | Already implemented |
| H6: Networks/Dependencies | PARTIAL | Missing `depends_on` |
| H7: No Public Exposure | NON-COMPLIANT | Ports bound to 0.0.0.0 |
| H8: No Committed Secrets | COMPLIANT | Only defaults in examples |
| H9: Logging to stdout/stderr | PARTIAL | Missing OTEL documentation |
| H10: Startup Order | PARTIAL | Missing `condition: service_healthy` |
| H11: README Documentation | PARTIAL | Missing infrastructure section |

---

## 6. Hardening Items

### H1: PostgreSQL Extensions (pgvector, pgcrypto)

#### Objective
Ensure PostgreSQL extensions `pgvector` and `pgcrypto` are enabled and verified via idempotent initialization scripts.

#### Current State
- `01-init-pgvector.sql` creates: `vector`, `uuid-ossp`, `pg_trgm`, `pgcrypto`
- `99-verify-extensions.sql` verifies: `vector`, `uuid-ossp`, `pg_trgm`, `pgcrypto`
- Scripts use `CREATE EXTENSION IF NOT EXISTS` (idempotent)

#### Required Changes
**None required.** Current implementation is compliant.

#### Acceptance Criteria
- [ ] `CREATE EXTENSION IF NOT EXISTS vector` present in init script
- [ ] `CREATE EXTENSION IF NOT EXISTS pgcrypto` present in init script
- [ ] Verification script checks for all required extensions
- [ ] Verification script raises ERROR on missing extensions
- [ ] Scripts are idempotent (safe to re-run)

#### Verification Steps
```bash
# Start fresh database
docker compose down -v
docker compose up -d postgres

# Wait for initialization
sleep 10

# Verify extensions
docker exec infra-postgres psql -U infra -d infra_vectors -c "SELECT extname, extversion FROM pg_extension WHERE extname IN ('vector', 'pgcrypto');"

# Expected output: 2 rows showing vector and pgcrypto with versions
```

---

### H2: Durable Data Persistence

#### Objective
Enforce durable data persistence through named volumes for PostgreSQL and Redis to prevent data loss on container restart or recreation.

#### Current State
```yaml
volumes:
  postgres-data:
    driver: local
  redis-data:
    driver: local
```

#### Required Changes
**None required.** Current implementation is compliant.

#### Acceptance Criteria
- [ ] PostgreSQL data mounted to named volume `postgres-data`
- [ ] Redis data mounted to named volume `redis-data`
- [ ] Volumes use `driver: local` for persistence
- [ ] Volume mounts use absolute container paths

#### Verification Steps
```bash
# Verify volumes exist and are named
docker volume ls | grep -E 'postgres-data|redis-data'

# Verify data survives restart
docker compose down
docker compose up -d
docker exec infra-postgres psql -U infra -d infra_vectors -c "SELECT COUNT(*) FROM vectors.collections;"
# Should return existing data
```

---

### H3: Environment Variable Standardization

#### Objective
Standardize environment variable usage with no reliance on .env files. All configuration must come from shell-exported environment variables.

#### Current State
- `.env.example` exists as documentation (explicitly states "do NOT copy to .env")
- `scripts/env-setup.sh` exports all required variables
- `docker-compose.yml` uses `${VAR:-default}` pattern
- No `.env` file loading is implemented

#### Required Changes
**None required.** Current implementation is compliant.

#### Acceptance Criteria
- [ ] No `.env` file exists in repository root
- [ ] `.env.example` contains explicit warning not to copy
- [ ] All environment variables have documented defaults
- [ ] `docker-compose.yml` uses `${VAR:-default}` pattern
- [ ] `scripts/env-setup.sh` exports all required variables
- [ ] `scripts/verify-env-config.sh` validates configuration

#### Verification Steps
```bash
# Verify no .env file exists
[ ! -f .env ] && echo "PASS: No .env file" || echo "FAIL: .env exists"

# Verify env-setup.sh works
source scripts/env-setup.sh
./scripts/verify-env-config.sh

# Verify docker-compose uses defaults
docker compose config | grep -E 'POSTGRES_USER|REDIS_PASSWORD'
```

---

### H4: Docker Image Version Pinning

#### Objective
Pin Docker image versions to specific, reproducible tags to ensure consistent deployments across environments.

#### Current State
```yaml
postgres:
  image: pgvector/pgvector:pg16

redis:
  image: redis:7-alpine
```

#### Required Changes
Update `docker-compose.yml` to use more specific version tags:

```yaml
postgres:
  image: pgvector/pgvector:pg16  # Pin to pg16 (latest within PostgreSQL 16.x series)
  # Note: For maximum reproducibility, consider using digest:
  # image: pgvector/pgvector@sha256:<digest>

redis:
  image: redis:7.4-alpine  # Pin to specific minor version
  # Note: For maximum reproducibility, consider using digest:
  # image: redis@sha256:<digest>
```

#### Acceptance Criteria
- [ ] PostgreSQL image uses version tag (minimum: major.minor)
- [ ] Redis image uses version tag (minimum: major.minor)
- [ ] Image tags are documented in comments
- [ ] No `:latest` tags are used

#### Verification Steps
```bash
# Verify images use explicit tags
grep -E '^[[:space:]]+image:' docker-compose.yml | grep -v latest
# Should show all images with explicit version tags

# Verify images can be pulled
docker compose pull
```

---

### H5: Health Checks and Restart Policies

#### Objective
Ensure all services have proper health checks and restart policies for automatic recovery from transient failures.

#### Current State
Both PostgreSQL and Redis already have health checks and `restart: unless-stopped`:

```yaml
postgres:
  restart: unless-stopped
  healthcheck:
    test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER:-infra} -d ${POSTGRES_DB:-infra_vectors}"]
    interval: 5s
    timeout: 5s
    retries: 5
    start_period: 10s

redis:
  restart: unless-stopped
  healthcheck:
    test: ["CMD", "redis-cli", "-a", "${REDIS_PASSWORD:-infra_redis_password}", "ping"]
    interval: 5s
    timeout: 3s
    retries: 5
    start_period: 5s
```

#### Required Changes
**None required.** Current implementation is compliant.

#### Acceptance Criteria
- [ ] PostgreSQL has health check with `pg_isready`
- [ ] Redis has health check with `redis-cli ping`
- [ ] Both services use `restart: unless-stopped`
- [ ] Health checks have appropriate intervals and timeouts
- [ ] Health checks use authentication where required

#### Verification Steps
```bash
# Start services
docker compose up -d

# Check health status
docker compose ps --format "table {{.Name}}\t{{.Status}}"
# Should show "(healthy)" for both services

# Verify health check execution
docker inspect infra-postgres --format='{{.State.Health.Status}}'
docker inspect infra-redis --format='{{.State.Health.Status}}'
```

---

### H6: Docker Networks and Service Dependencies

#### Objective
Define explicit Docker networks and service dependencies to ensure proper service isolation and startup ordering.

#### Current State
```yaml
networks:
  infra-network:
    driver: bridge
    ipam:
      config:
        - subnet: 172.28.0.0/16
```

Missing: `depends_on` declarations between services.

#### Required Changes
Add `depends_on` to ensure Redis waits for PostgreSQL (if needed by application):

```yaml
redis:
  depends_on:
    postgres:
      condition: service_healthy
```

Note: If Redis and PostgreSQL are independent, no `depends_on` is required.

#### Acceptance Criteria
- [ ] Custom bridge network is defined with explicit name
- [ ] Network uses bridge driver
- [ ] Network has defined subnet (optional but recommended)
- [ ] All services are attached to the same network
- [ ] Service dependencies are explicitly declared if required

#### Verification Steps
```bash
# Verify network exists
docker network ls | grep infra-network

# Verify services are on the same network
docker network inspect infra_infra-network --format='{{range .Containers}}{{.Name}} {{end}}'
# Should list both containers

# Verify services can communicate
docker exec infra-postgres ping -c 1 infra-redis
```

---

### H7: Network Exposure Security

#### Objective
Ensure no services are publicly exposed by default. Services should bind to localhost or be accessible only through the Docker network.

#### Current State
```yaml
postgres:
  ports:
    - "${POSTGRES_PORT:-5432}:5432"  # Binds to 0.0.0.0

redis:
  ports:
    - "${REDIS_PORT:-6379}:6379"    # Binds to 0.0.0.0
```

#### Required Changes
Update port bindings to use localhost by default:

```yaml
postgres:
  ports:
    - "127.0.0.1:${POSTGRES_PORT:-5432}:5432"

redis:
  ports:
    - "127.0.0.1:${REDIS_PORT:-6379}:6379"
```

Or remove port mappings entirely and rely on Docker network for inter-service communication.

#### Acceptance Criteria
- [ ] PostgreSQL port binds to 127.0.0.1 (or no external binding)
- [ ] Redis port binds to 127.0.0.1 (or no external binding)
- [ ] No services are accessible from external networks by default
- [ ] Documentation explains how to expose services if needed

#### Verification Steps
```bash
# Start services
docker compose up -d

# Verify port bindings
docker port infra-postgres
docker port infra-redis
# Should show 127.0.0.1:<port> or no output

# Verify external access is blocked (from another machine)
nc -zv <host-ip> 5432
# Should fail if properly bound to localhost
```

---

### H8: Secrets Management

#### Objective
Confirm secrets are never committed to the repository. All secrets must be provided via environment variables at runtime.

#### Current State
- `.env.example` contains example passwords (documented as non-production)
- `docker-compose.yml` uses `${VAR:-default}` with fallback defaults
- No actual secrets in committed files
- `.gitignore` should exclude `.env` files

#### Required Changes
Verify `.gitignore` includes:
```
.env
.env.*
!.env.example
```

Add security warning to `docker-compose.yml`:
```yaml
# SECURITY WARNING: Change all default passwords before production deployment
# Required environment variables:
#   POSTGRES_PASSWORD - PostgreSQL password (no default in production)
#   REDIS_PASSWORD    - Redis password (no default in production)
```

#### Acceptance Criteria
- [ ] No actual secrets (API keys, passwords) in committed files
- [ ] `.gitignore` excludes `.env` files (except `.env.example`)
- [ ] Default passwords are marked as development-only
- [ ] Documentation warns about changing defaults
- [ ] Pre-commit hooks or CI checks for secrets (recommended)

#### Verification Steps
```bash
# Search for potential secrets
grep -rn "password\|secret\|api.key" --include="*.yml" --include="*.yaml" --include="*.sh" | grep -v example | grep -v default

# Verify .gitignore
cat .gitignore | grep -E '\.env'

# Verify no .env files are tracked
git ls-files | grep '\.env' | grep -v example
# Should return empty
```

---

### H9: Logging Configuration

#### Objective
Route all service logs to stdout/stderr for container log aggregation. Document OpenTelemetry integration hooks for future observability.

#### Current State
- PostgreSQL logs to stdout/stderr by default
- Redis logs to stdout/stderr by default
- No explicit logging driver configuration
- OpenTelemetry configuration exists in `.env.example`

#### Required Changes
Add explicit logging configuration to `docker-compose.yml`:

```yaml
services:
  postgres:
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"

  redis:
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

Add OpenTelemetry integration documentation (not implementation):

```yaml
# OpenTelemetry Integration (Documentation Only)
#
# To enable OTLP log forwarding, add an OpenTelemetry Collector service
# and configure these environment variables:
#
#   OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4317
#   OTEL_SERVICE_NAME=<service-name>
#
# The infra-otel crate provides Rust integration for application logs.
# See: crates/infra-otel/README.md
```

#### Acceptance Criteria
- [ ] All services log to stdout/stderr
- [ ] Logging driver is explicitly configured
- [ ] Log rotation is configured (max-size, max-file)
- [ ] OpenTelemetry hooks are documented in comments
- [ ] No log files are written inside containers

#### Verification Steps
```bash
# Verify logs are accessible
docker compose logs postgres | head -20
docker compose logs redis | head -20

# Verify logging configuration
docker inspect infra-postgres --format='{{.HostConfig.LogConfig}}'

# Verify no internal log files
docker exec infra-postgres find /var/log -type f -name "*.log" 2>/dev/null
# Should return empty or only system logs
```

---

### H10: Startup Order Guarantees

#### Objective
Clarify and enforce startup order guarantees to ensure services start in the correct sequence and wait for dependencies to be healthy.

#### Current State
- Both services have health checks
- No `depends_on` declarations
- PostgreSQL and Redis start independently

#### Required Changes
If services have dependencies, add:

```yaml
services:
  postgres:
    # PostgreSQL starts first (no dependencies)

  redis:
    depends_on:
      postgres:
        condition: service_healthy  # Wait for PostgreSQL to be healthy
```

Add documentation for startup order:

```yaml
# Startup Order:
# 1. postgres - Starts first, health check: pg_isready
# 2. redis - Starts after postgres is healthy (if depends_on is set)
#
# Wait for services: docker compose up -d --wait
```

#### Acceptance Criteria
- [ ] Startup order is documented in comments
- [ ] `depends_on` with `condition: service_healthy` for dependent services
- [ ] Health checks complete before dependent services start
- [ ] `docker compose up -d --wait` waits for all health checks

#### Verification Steps
```bash
# Start with wait flag
docker compose up -d --wait

# Verify startup order in logs
docker compose logs --timestamps | head -50

# Verify all services are healthy before returning
docker compose ps --format "table {{.Name}}\t{{.Status}}"
# All should show "(healthy)"
```

---

### H11: README Documentation

#### Objective
Document infrastructure usage and constraints clearly in the README, including operational requirements, security considerations, and deployment guidelines.

#### Current State
Current README focuses on Rust crates and TypeScript SDK. Missing:
- Infrastructure section
- Docker Compose usage
- Environment configuration
- Security considerations

#### Required Changes
Add infrastructure section to README.md:

```markdown
## Infrastructure

### Docker Services

This repository includes Docker Compose configuration for development dependencies:

- **PostgreSQL 16** with pgvector extension for vector storage
- **Redis 7** for caching (optional)

### Quick Start

```bash
# Set up environment (development defaults)
source scripts/env-setup.sh

# Start services
docker compose up -d

# Verify PostgreSQL extensions
./scripts/verify-postgres-extensions.sh
```

### Environment Configuration

All configuration uses shell-exported environment variables. **No .env file is used.**

See `.env.example` for available variables and documentation.

### Security Considerations

- Default passwords are for development only
- Services bind to localhost by default
- Change all passwords before production deployment
- Never commit secrets to the repository

### Production Deployment

For production deployments:

1. Set secure passwords via environment variables
2. Use external secrets management (Vault, AWS Secrets Manager, etc.)
3. Consider managed database services for PostgreSQL
4. Enable TLS for all connections
5. Configure proper network segmentation
```

#### Acceptance Criteria
- [ ] Infrastructure section in README
- [ ] Docker Compose quick start documented
- [ ] Environment variable configuration documented
- [ ] Security considerations section
- [ ] Production deployment guidance
- [ ] Links to verification scripts

#### Verification Steps
```bash
# Verify README contains infrastructure section
grep -q "## Infrastructure" README.md && echo "PASS" || echo "FAIL: Missing Infrastructure section"

# Verify docker compose commands are documented
grep -q "docker compose" README.md && echo "PASS" || echo "FAIL: Missing docker compose docs"

# Verify security section
grep -q "Security" README.md && echo "PASS" || echo "FAIL: Missing security section"
```

---

## 7. Implementation Priorities

### Priority 1: Security (Immediate)

| Item | Risk | Effort |
|------|------|--------|
| H7: Network Exposure | High | Small |
| H8: Secrets Management | Medium | Trivial |

### Priority 2: Reliability (Short-term)

| Item | Risk | Effort |
|------|------|--------|
| H4: Image Pinning | Medium | Small |
| H10: Startup Order | Medium | Small |
| H6: Dependencies | Low | Small |

### Priority 3: Operations (Medium-term)

| Item | Risk | Effort |
|------|------|--------|
| H9: Logging | Low | Small |
| H11: Documentation | Low | Medium |

### Priority 4: Already Compliant (No Action)

| Item | Status |
|------|--------|
| H1: PostgreSQL Extensions | Compliant |
| H2: Named Volumes | Compliant |
| H3: Environment Variables | Compliant |
| H5: Health Checks | Compliant |

---

## 8. Risk Assessment

### 8.1 Change Risks

| Change | Risk Level | Mitigation |
|--------|------------|------------|
| Binding to localhost | Low | Document override procedure |
| Image version pinning | Low | Test before pinning |
| Adding depends_on | Low | Only if truly dependent |
| Logging configuration | Very Low | Default behavior preserved |

### 8.2 Operational Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Breaking existing workflows | High | Low | Backward compatible changes only |
| Development friction | Medium | Low | Defaults preserve dev experience |
| Missing documentation | Low | High | Comprehensive README update |

---

## Appendix A: Verification Commands

### Complete Verification Script

```bash
#!/bin/bash
# Complete infrastructure hardening verification

set -e

echo "=== Infrastructure Hardening Verification ==="

# H1: PostgreSQL Extensions
echo "H1: Checking PostgreSQL extensions..."
docker exec infra-postgres psql -U infra -d infra_vectors -c \
  "SELECT extname FROM pg_extension WHERE extname IN ('vector', 'pgcrypto');" \
  | grep -q vector && echo "  ✓ pgvector" || echo "  ✗ pgvector"
docker exec infra-postgres psql -U infra -d infra_vectors -c \
  "SELECT extname FROM pg_extension WHERE extname IN ('vector', 'pgcrypto');" \
  | grep -q pgcrypto && echo "  ✓ pgcrypto" || echo "  ✗ pgcrypto"

# H2: Named Volumes
echo "H2: Checking named volumes..."
docker volume ls | grep -q postgres-data && echo "  ✓ postgres-data" || echo "  ✗ postgres-data"
docker volume ls | grep -q redis-data && echo "  ✓ redis-data" || echo "  ✗ redis-data"

# H3: No .env file
echo "H3: Checking .env configuration..."
[ ! -f .env ] && echo "  ✓ No .env file" || echo "  ✗ .env file exists"

# H4: Image pinning
echo "H4: Checking image version pinning..."
grep -q "pgvector:pg16" docker-compose.yml && echo "  ✓ PostgreSQL pinned" || echo "  ✗ PostgreSQL not pinned"
grep -q "redis:7" docker-compose.yml && echo "  ✓ Redis pinned" || echo "  ✗ Redis not pinned"

# H5: Health checks
echo "H5: Checking health checks..."
docker inspect infra-postgres --format='{{.State.Health.Status}}' | grep -q healthy && echo "  ✓ PostgreSQL healthy" || echo "  ✗ PostgreSQL unhealthy"
docker inspect infra-redis --format='{{.State.Health.Status}}' | grep -q healthy && echo "  ✓ Redis healthy" || echo "  ✗ Redis unhealthy"

# H6: Networks
echo "H6: Checking Docker networks..."
docker network ls | grep -q infra-network && echo "  ✓ infra-network exists" || echo "  ✗ infra-network missing"

# H7: Port bindings
echo "H7: Checking port bindings..."
docker port infra-postgres 2>/dev/null | grep -q "127.0.0.1" && echo "  ✓ PostgreSQL localhost-only" || echo "  ⚠ PostgreSQL publicly bound"
docker port infra-redis 2>/dev/null | grep -q "127.0.0.1" && echo "  ✓ Redis localhost-only" || echo "  ⚠ Redis publicly bound"

# H8: No committed secrets
echo "H8: Checking for committed secrets..."
git ls-files | grep '\.env$' | grep -v example | wc -l | grep -q "^0$" && echo "  ✓ No .env committed" || echo "  ✗ .env files committed"

# H9: Logging
echo "H9: Checking logging configuration..."
docker logs infra-postgres 2>&1 | head -1 > /dev/null && echo "  ✓ PostgreSQL logs accessible" || echo "  ✗ PostgreSQL logs inaccessible"
docker logs infra-redis 2>&1 | head -1 > /dev/null && echo "  ✓ Redis logs accessible" || echo "  ✗ Redis logs inaccessible"

# H10: Startup order (check compose file)
echo "H10: Checking startup order documentation..."
grep -q "depends_on" docker-compose.yml && echo "  ✓ depends_on configured" || echo "  ⚠ No depends_on (may be intentional)"

# H11: Documentation
echo "H11: Checking documentation..."
grep -q "Infrastructure" README.md && echo "  ✓ Infrastructure section in README" || echo "  ✗ Missing Infrastructure section"

echo ""
echo "=== Verification Complete ==="
```

---

## Appendix B: File Inventory

### Files to Modify

| File | Changes Required |
|------|------------------|
| `docker-compose.yml` | H4 (version pinning), H6 (depends_on), H7 (localhost binding), H9 (logging config) |
| `README.md` | H11 (infrastructure documentation) |
| `.gitignore` | H8 (ensure .env excluded) |

### Files Already Compliant

| File | Status |
|------|--------|
| `docker/postgres/init/01-init-pgvector.sql` | H1 Compliant |
| `docker/postgres/init/99-verify-extensions.sql` | H1 Compliant |
| `scripts/env-setup.sh` | H3 Compliant |
| `scripts/verify-postgres-extensions.sh` | H1 Compliant |
| `scripts/verify-env-config.sh` | H3 Compliant |
| `.env.example` | H3, H8 Compliant |

### Files to Create

**None.** All hardening can be achieved by modifying existing files.

---

**Document Status:** SPECIFICATION COMPLETE

**Next Steps:**
1. Review this specification with team
2. Approve or request modifications
3. Implement changes in priority order
4. Run verification script after each change
5. Update documentation

**Revision History:**
- v1.0.0 (2025-12-17): Initial specification
