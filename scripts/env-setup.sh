#!/bin/bash
# LLM-Dev-Ops Infrastructure - Environment Variable Setup Script
#
# This script exports all required environment variables for the infra project.
# Source this script before running docker compose or the application:
#
#   source scripts/env-setup.sh
#   docker compose up -d
#
# IMPORTANT: This script is designed to be sourced, not executed directly.
# All variables are exported to the current shell session.
#
# For production, override these values with your own secure credentials:
#   export POSTGRES_PASSWORD=your_secure_password
#   source scripts/env-setup.sh  # Will not override existing exports

# =============================================================================
# RuvVector/PostgreSQL Configuration
# =============================================================================
# Uses ruvnet/ruvector image (PostgreSQL 17 with RuvVector extension)
# RuvVector provides pgvector-compatible 'vector' type plus advanced features:
#   - Hyperbolic embeddings (Poincaré ball)
#   - GNN operations (GCN, GraphSAGE)
#   - Graph operations (Cypher queries)
#   - Self-learning capabilities

# PostgreSQL connection parameters
export POSTGRES_HOST="${POSTGRES_HOST:-localhost}"
export POSTGRES_PORT="${POSTGRES_PORT:-5432}"
export POSTGRES_USER="${POSTGRES_USER:-infra}"
export POSTGRES_PASSWORD="${POSTGRES_PASSWORD:-infra_password}"
export POSTGRES_DB="${POSTGRES_DB:-infra_vectors}"

# Full connection URL (constructed from components)
export DATABASE_URL="${DATABASE_URL:-postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@${POSTGRES_HOST}:${POSTGRES_PORT}/${POSTGRES_DB}}"

# PostgreSQL connection for vector operations (same as DATABASE_URL by default)
export INFRA_POSTGRES_URL="${INFRA_POSTGRES_URL:-${DATABASE_URL}}"

# =============================================================================
# Redis Configuration
# =============================================================================

export REDIS_HOST="${REDIS_HOST:-localhost}"
export REDIS_PORT="${REDIS_PORT:-6379}"
export REDIS_PASSWORD="${REDIS_PASSWORD:-infra_redis_password}"

# Full Redis URL (constructed from components)
export REDIS_URL="${REDIS_URL:-redis://:${REDIS_PASSWORD}@${REDIS_HOST}:${REDIS_PORT}}"

# =============================================================================
# Infra Vector Crate Configuration
# =============================================================================

# Default collection name for vector storage
export INFRA_VECTOR_COLLECTION="${INFRA_VECTOR_COLLECTION:-default}"

# Vector dimensions (1536 for OpenAI ada-002 embeddings)
export INFRA_VECTOR_DIMENSIONS="${INFRA_VECTOR_DIMENSIONS:-1536}"

# Distance metric: cosine, euclidean, dot_product, manhattan
export INFRA_VECTOR_DISTANCE="${INFRA_VECTOR_DISTANCE:-cosine}"

# RuvVector HTTP endpoint (optional, for RuvVector backend)
# export INFRA_RUVECTOR_URL="${INFRA_RUVECTOR_URL:-http://localhost:8100}"

# =============================================================================
# OpenTelemetry Configuration
# =============================================================================

export OTEL_SERVICE_NAME="${OTEL_SERVICE_NAME:-infra-vector}"
export OTEL_SERVICE_VERSION="${OTEL_SERVICE_VERSION:-0.1.0}"
export OTEL_EXPORTER_OTLP_ENDPOINT="${OTEL_EXPORTER_OTLP_ENDPOINT:-http://localhost:4317}"
export OTEL_TRACES_SAMPLER_ARG="${OTEL_TRACES_SAMPLER_ARG:-1.0}"

# =============================================================================
# Development Settings
# =============================================================================

# Rust logging configuration
export RUST_LOG="${RUST_LOG:-info,infra_vector=debug}"

# Enable Rust backtraces for debugging
export RUST_BACKTRACE="${RUST_BACKTRACE:-1}"

# =============================================================================
# Verification Output
# =============================================================================

echo "Environment variables exported for LLM-Dev-Ops Infrastructure"
echo ""
echo "RuvVector/PostgreSQL:"
echo "  POSTGRES_HOST=${POSTGRES_HOST}"
echo "  POSTGRES_PORT=${POSTGRES_PORT}"
echo "  POSTGRES_USER=${POSTGRES_USER}"
echo "  POSTGRES_DB=${POSTGRES_DB}"
echo "  DATABASE_URL=postgres://***:***@${POSTGRES_HOST}:${POSTGRES_PORT}/${POSTGRES_DB}"
echo ""
echo "Redis:"
echo "  REDIS_HOST=${REDIS_HOST}"
echo "  REDIS_PORT=${REDIS_PORT}"
echo "  REDIS_URL=redis://***@${REDIS_HOST}:${REDIS_PORT}"
echo ""
echo "Vector Store:"
echo "  INFRA_VECTOR_COLLECTION=${INFRA_VECTOR_COLLECTION}"
echo "  INFRA_VECTOR_DIMENSIONS=${INFRA_VECTOR_DIMENSIONS}"
echo "  INFRA_VECTOR_DISTANCE=${INFRA_VECTOR_DISTANCE}"
echo ""
echo "OpenTelemetry:"
echo "  OTEL_SERVICE_NAME=${OTEL_SERVICE_NAME}"
echo "  OTEL_SERVICE_VERSION=${OTEL_SERVICE_VERSION}"
echo ""
echo "Development:"
echo "  RUST_LOG=${RUST_LOG}"
echo "  RUST_BACKTRACE=${RUST_BACKTRACE}"
echo ""
echo "To start services: docker compose up -d"
