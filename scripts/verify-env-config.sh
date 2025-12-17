#!/bin/bash
# LLM-Dev-Ops Infrastructure - Environment Configuration Verification Script
#
# This script verifies that all required environment variables are set
# for shell-export-only configuration mode.
#
# Usage:
#   source scripts/env-setup.sh
#   ./scripts/verify-env-config.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "============================================================"
echo "LLM-Dev-Ops Environment Configuration Verification"
echo "============================================================"
echo ""

# Track if all checks pass
ALL_OK=true

# Function to check if a variable is set
check_var() {
    local var_name="$1"
    local required="$2"
    local default_val="$3"
    local current_val="${!var_name}"

    if [ -n "$current_val" ]; then
        # Mask passwords in output
        if [[ "$var_name" == *PASSWORD* ]] || [[ "$var_name" == *_URL* ]]; then
            echo -e "${GREEN}[OK]${NC} $var_name is set (value masked)"
        else
            echo -e "${GREEN}[OK]${NC} $var_name = $current_val"
        fi
    elif [ "$required" = "true" ]; then
        echo -e "${RED}[MISSING]${NC} $var_name (required, default: $default_val)"
        ALL_OK=false
    else
        echo -e "${YELLOW}[DEFAULT]${NC} $var_name will use default: $default_val"
    fi
}

echo "Checking PostgreSQL Configuration..."
check_var "POSTGRES_HOST" false "localhost"
check_var "POSTGRES_PORT" false "5432"
check_var "POSTGRES_USER" false "infra"
check_var "POSTGRES_PASSWORD" false "infra_password"
check_var "POSTGRES_DB" false "infra_vectors"
check_var "DATABASE_URL" false "(constructed from components)"
echo ""

echo "Checking Redis Configuration..."
check_var "REDIS_HOST" false "localhost"
check_var "REDIS_PORT" false "6379"
check_var "REDIS_PASSWORD" false "infra_redis_password"
check_var "REDIS_URL" false "(constructed from components)"
echo ""

echo "Checking Vector Store Configuration..."
check_var "INFRA_VECTOR_COLLECTION" false "default"
check_var "INFRA_VECTOR_DIMENSIONS" false "1536"
check_var "INFRA_VECTOR_DISTANCE" false "cosine"
check_var "INFRA_RUVECTOR_URL" false "(optional)"
echo ""

echo "Checking OpenTelemetry Configuration..."
check_var "OTEL_SERVICE_NAME" false "infra-vector"
check_var "OTEL_SERVICE_VERSION" false "0.1.0"
check_var "OTEL_EXPORTER_OTLP_ENDPOINT" false "http://localhost:4317"
echo ""

echo "Checking Development Settings..."
check_var "RUST_LOG" false "info,infra_vector=debug"
check_var "RUST_BACKTRACE" false "1"
echo ""

# Check for .env file (should NOT exist)
echo "Checking for .env file presence..."
if [ -f ".env" ]; then
    echo -e "${YELLOW}[WARNING]${NC} .env file found at $(pwd)/.env"
    echo "  This project uses shell-exported variables only."
    echo "  The .env file may cause confusion and is not loaded by the application."
    echo "  Consider removing it: rm .env"
else
    echo -e "${GREEN}[OK]${NC} No .env file present (correct - using shell exports only)"
fi
echo ""

echo "============================================================"
if [ "$ALL_OK" = true ]; then
    echo -e "${GREEN}All environment checks passed!${NC}"
    echo ""
    echo "To start services with current configuration:"
    echo "  docker compose up -d"
    echo ""
    echo "To verify PostgreSQL extensions after startup:"
    echo "  ./scripts/verify-postgres-extensions.sh"
    exit 0
else
    echo -e "${RED}Some required variables are missing!${NC}"
    echo ""
    echo "To set up environment variables, run:"
    echo "  source scripts/env-setup.sh"
    echo ""
    echo "Or export them individually:"
    echo "  export POSTGRES_PASSWORD=your_password"
    exit 1
fi
