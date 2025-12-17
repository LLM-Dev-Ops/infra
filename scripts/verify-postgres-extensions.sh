#!/bin/bash
# LLM-Dev-Ops Infrastructure - PostgreSQL Extension Verification Script
# Usage: ./scripts/verify-postgres-extensions.sh
#
# This script verifies that all required PostgreSQL extensions are installed
# and operational for the RuvVector integration.
#
# ENVIRONMENT CONFIGURATION:
# All configuration is read from shell-exported environment variables.
# No .env file is used. Export variables before running:
#
#   export POSTGRES_HOST=localhost
#   export POSTGRES_PASSWORD=your_password
#   ./scripts/verify-postgres-extensions.sh
#
# Or source the environment setup script first:
#   source scripts/env-setup.sh
#   ./scripts/verify-postgres-extensions.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Connection parameters from shell environment (with defaults for development)
POSTGRES_HOST="${POSTGRES_HOST:-localhost}"
POSTGRES_PORT="${POSTGRES_PORT:-5432}"
POSTGRES_USER="${POSTGRES_USER:-infra}"
POSTGRES_PASSWORD="${POSTGRES_PASSWORD:-infra_password}"
POSTGRES_DB="${POSTGRES_DB:-infra_vectors}"

echo "============================================================"
echo "PostgreSQL Extension Verification for LLM-Dev-Ops"
echo "============================================================"
echo ""
echo "Connection: ${POSTGRES_USER}@${POSTGRES_HOST}:${POSTGRES_PORT}/${POSTGRES_DB}"
echo ""

# Core extensions (always required)
CORE_EXTENSIONS=("uuid-ossp" "pg_trgm" "pgcrypto")

# Check if psql is available
if ! command -v psql &> /dev/null; then
    echo -e "${RED}Error: psql command not found. Install postgresql-client or use docker exec.${NC}"
    echo ""
    echo "Alternative: Run via Docker:"
    echo "  docker exec infra-ruvector psql -U \$POSTGRES_USER -d \$POSTGRES_DB -c \"SELECT extname, extversion FROM pg_extension;\""
    exit 1
fi

# Function to run SQL query
run_sql() {
    PGPASSWORD="${POSTGRES_PASSWORD}" psql -h "${POSTGRES_HOST}" -p "${POSTGRES_PORT}" -U "${POSTGRES_USER}" -d "${POSTGRES_DB}" -t -A -c "$1" 2>/dev/null
}

# Check connection
echo "Checking database connection..."
if ! run_sql "SELECT 1;" > /dev/null 2>&1; then
    echo -e "${RED}✗ Cannot connect to PostgreSQL${NC}"
    echo ""
    echo "Ensure the database is running:"
    echo "  docker compose up -d ruvector"
    exit 1
fi
echo -e "${GREEN}✓ Database connection successful${NC}"
echo ""

# Check for vector extension (ruvector or pgvector)
echo "Checking vector extension..."
ALL_OK=true
VECTOR_EXT=""

ruvector_result=$(run_sql "SELECT extversion FROM pg_extension WHERE extname = 'ruvector';")
if [ -n "$ruvector_result" ]; then
    echo -e "${GREEN}✓ ruvector${NC} (version: ${ruvector_result}) - Advanced vector database"
    VECTOR_EXT="ruvector"
else
    pgvector_result=$(run_sql "SELECT extversion FROM pg_extension WHERE extname = 'vector';")
    if [ -n "$pgvector_result" ]; then
        echo -e "${GREEN}✓ pgvector${NC} (version: ${pgvector_result})"
        VECTOR_EXT="pgvector"
    else
        echo -e "${RED}✗ No vector extension (ruvector or pgvector) found${NC}"
        ALL_OK=false
    fi
fi

echo ""

# Check core extensions
echo "Checking core extensions..."
for ext in "${CORE_EXTENSIONS[@]}"; do
    result=$(run_sql "SELECT extversion FROM pg_extension WHERE extname = '${ext}';")
    if [ -n "$result" ]; then
        echo -e "${GREEN}✓ ${ext}${NC} (version: ${result})"
    else
        echo -e "${RED}✗ ${ext} - NOT INSTALLED${NC}"
        ALL_OK=false
    fi
done

echo ""

# Verify vector functionality (works with both ruvector and pgvector)
echo "Verifying vector functionality..."
if run_sql "SELECT '[1,2,3]'::vector(3);" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Vector type operational${NC}"
else
    echo -e "${RED}✗ Vector type failed${NC}"
    ALL_OK=false
fi

if run_sql "SELECT '[1,2,3]'::vector(3) <=> '[4,5,6]'::vector(3);" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Cosine distance operational${NC}"
else
    echo -e "${RED}✗ Cosine distance failed${NC}"
    ALL_OK=false
fi

echo ""

# Verify uuid-ossp functionality
echo "Verifying uuid-ossp functionality..."
uuid_result=$(run_sql "SELECT uuid_generate_v4();")
if [ -n "$uuid_result" ]; then
    echo -e "${GREEN}✓ UUID generation operational (sample: ${uuid_result})${NC}"
else
    echo -e "${RED}✗ UUID generation failed${NC}"
    ALL_OK=false
fi

echo ""

# Verify pgcrypto functionality
echo "Verifying pgcrypto functionality..."
digest_result=$(run_sql "SELECT encode(digest('test', 'sha256'), 'hex');")
if [ -n "$digest_result" ] && [ ${#digest_result} -eq 64 ]; then
    echo -e "${GREEN}✓ SHA256 digest operational${NC}"
else
    echo -e "${RED}✗ SHA256 digest failed${NC}"
    ALL_OK=false
fi

echo ""

# Check vectors schema
echo "Checking vectors schema..."
if run_sql "SELECT 1 FROM information_schema.schemata WHERE schema_name = 'vectors';" | grep -q 1; then
    echo -e "${GREEN}✓ 'vectors' schema exists${NC}"

    # Check collections table
    if run_sql "SELECT 1 FROM information_schema.tables WHERE table_schema = 'vectors' AND table_name = 'collections';" | grep -q 1; then
        echo -e "${GREEN}✓ 'vectors.collections' table exists${NC}"
    else
        echo -e "${YELLOW}⚠ 'vectors.collections' table not found${NC}"
    fi
else
    echo -e "${YELLOW}⚠ 'vectors' schema not found${NC}"
fi

# Check for RuvVector-specific features (if ruvector is installed)
if [ "$VECTOR_EXT" = "ruvector" ]; then
    echo ""
    echo "Checking RuvVector-specific features..."
    if run_sql "SELECT ruvector_version();" > /dev/null 2>&1; then
        version=$(run_sql "SELECT ruvector_version();")
        echo -e "${GREEN}✓ RuvVector version: ${version}${NC}"
    else
        echo -e "${YELLOW}⚠ RuvVector version function not available${NC}"
    fi
fi

echo ""
echo "============================================================"

if [ "$ALL_OK" = true ]; then
    echo -e "${GREEN}All PostgreSQL extensions verified successfully!${NC}"
    echo "Vector engine: ${VECTOR_EXT}"
    echo "The database is ready for RuvVector integration."
    exit 0
else
    echo -e "${RED}Some extensions or features are missing!${NC}"
    echo ""
    echo "To fix, ensure Docker init scripts ran correctly:"
    echo "  1. docker compose down -v  # Remove volumes to reset"
    echo "  2. docker compose up -d ruvector"
    echo "  3. ./scripts/verify-postgres-extensions.sh"
    exit 1
fi
