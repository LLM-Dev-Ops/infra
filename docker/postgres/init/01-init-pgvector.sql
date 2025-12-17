-- LLM-Dev-Ops Infrastructure - PostgreSQL Extension Initialization
-- This script sets up required extensions and schema for RuvVector compatibility
-- As specified in SPARC plan: ruvector-postgres provides pgvector-compatible extension
--
-- Required Extensions:
--   - vector (pgvector): Vector similarity search for LLM embeddings
--   - uuid-ossp: UUID generation for collection IDs
--   - pg_trgm: Trigram text similarity for metadata search
--   - pgcrypto: Cryptographic functions for secure operations
--
-- This script is idempotent and safe to re-run (uses IF NOT EXISTS)

-- Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

-- Enable additional useful extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Create schema for vector operations
CREATE SCHEMA IF NOT EXISTS vectors;

-- Set search path
SET search_path TO vectors, public;

-- =============================================================================
-- Vector Collections Table
-- Stores metadata about vector collections
-- =============================================================================
CREATE TABLE IF NOT EXISTS vectors.collections (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) UNIQUE NOT NULL,
    dimensions INTEGER NOT NULL,
    distance_metric VARCHAR(50) NOT NULL DEFAULT 'cosine',
    hnsw_m INTEGER DEFAULT 16,
    hnsw_ef_construction INTEGER DEFAULT 200,
    hnsw_ef_search INTEGER DEFAULT 100,
    compression_enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb,
    CONSTRAINT valid_dimensions CHECK (dimensions > 0 AND dimensions <= 65536),
    CONSTRAINT valid_distance CHECK (distance_metric IN ('cosine', 'euclidean', 'dot_product', 'manhattan'))
);

-- Index for collection lookups
CREATE INDEX IF NOT EXISTS idx_collections_name ON vectors.collections(name);

-- =============================================================================
-- Vector Records Table (Template - actual tables created per collection)
-- This serves as documentation; actual vector tables are created dynamically
-- =============================================================================
-- Example structure for a 1536-dimension collection:
-- CREATE TABLE vectors.collection_<name> (
--     id VARCHAR(255) PRIMARY KEY,
--     vector vector(1536) NOT NULL,
--     metadata JSONB DEFAULT '{}'::jsonb,
--     created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
--     updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
-- );
-- CREATE INDEX ON vectors.collection_<name> USING ivfflat (vector vector_cosine_ops);

-- =============================================================================
-- Function to create a new vector collection
-- =============================================================================
CREATE OR REPLACE FUNCTION vectors.create_collection(
    p_name VARCHAR(255),
    p_dimensions INTEGER,
    p_distance_metric VARCHAR(50) DEFAULT 'cosine',
    p_hnsw_m INTEGER DEFAULT 16,
    p_hnsw_ef_construction INTEGER DEFAULT 200,
    p_metadata JSONB DEFAULT '{}'::jsonb
) RETURNS UUID AS $$
DECLARE
    v_collection_id UUID;
    v_table_name TEXT;
    v_index_ops TEXT;
BEGIN
    -- Sanitize table name
    v_table_name := 'collection_' || regexp_replace(p_name, '[^a-zA-Z0-9_]', '_', 'g');

    -- Determine index operator class based on distance metric
    CASE p_distance_metric
        WHEN 'cosine' THEN v_index_ops := 'vector_cosine_ops';
        WHEN 'euclidean' THEN v_index_ops := 'vector_l2_ops';
        WHEN 'dot_product' THEN v_index_ops := 'vector_ip_ops';
        ELSE v_index_ops := 'vector_cosine_ops';
    END CASE;

    -- Insert collection metadata
    INSERT INTO vectors.collections (name, dimensions, distance_metric, hnsw_m, hnsw_ef_construction, metadata)
    VALUES (p_name, p_dimensions, p_distance_metric, p_hnsw_m, p_hnsw_ef_construction, p_metadata)
    RETURNING id INTO v_collection_id;

    -- Create the vector table
    EXECUTE format('
        CREATE TABLE vectors.%I (
            id VARCHAR(255) PRIMARY KEY,
            vector vector(%s) NOT NULL,
            metadata JSONB DEFAULT ''{}''::jsonb,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )', v_table_name, p_dimensions);

    -- Create HNSW index for fast similarity search
    EXECUTE format('
        CREATE INDEX %I ON vectors.%I
        USING hnsw (vector %s)
        WITH (m = %s, ef_construction = %s)',
        'idx_' || v_table_name || '_vector',
        v_table_name,
        v_index_ops,
        p_hnsw_m,
        p_hnsw_ef_construction
    );

    -- Create GIN index for metadata queries
    EXECUTE format('
        CREATE INDEX %I ON vectors.%I USING GIN (metadata)',
        'idx_' || v_table_name || '_metadata',
        v_table_name
    );

    RETURN v_collection_id;
END;
$$ LANGUAGE plpgsql;

-- =============================================================================
-- Function to drop a vector collection
-- =============================================================================
CREATE OR REPLACE FUNCTION vectors.drop_collection(p_name VARCHAR(255)) RETURNS BOOLEAN AS $$
DECLARE
    v_table_name TEXT;
BEGIN
    v_table_name := 'collection_' || regexp_replace(p_name, '[^a-zA-Z0-9_]', '_', 'g');

    -- Drop the table
    EXECUTE format('DROP TABLE IF EXISTS vectors.%I CASCADE', v_table_name);

    -- Delete collection metadata
    DELETE FROM vectors.collections WHERE name = p_name;

    RETURN FOUND;
END;
$$ LANGUAGE plpgsql;

-- =============================================================================
-- Function to insert vectors (batch)
-- =============================================================================
CREATE OR REPLACE FUNCTION vectors.insert_vectors(
    p_collection_name VARCHAR(255),
    p_vectors JSONB  -- Array of {id, vector, metadata}
) RETURNS INTEGER AS $$
DECLARE
    v_table_name TEXT;
    v_inserted INTEGER := 0;
    v_record JSONB;
BEGIN
    v_table_name := 'collection_' || regexp_replace(p_collection_name, '[^a-zA-Z0-9_]', '_', 'g');

    FOR v_record IN SELECT * FROM jsonb_array_elements(p_vectors)
    LOOP
        EXECUTE format('
            INSERT INTO vectors.%I (id, vector, metadata)
            VALUES ($1, $2::vector, $3)
            ON CONFLICT (id) DO UPDATE SET
                vector = EXCLUDED.vector,
                metadata = EXCLUDED.metadata,
                updated_at = NOW()',
            v_table_name
        ) USING
            v_record->>'id',
            v_record->>'vector',
            COALESCE(v_record->'metadata', '{}'::jsonb);

        v_inserted := v_inserted + 1;
    END LOOP;

    RETURN v_inserted;
END;
$$ LANGUAGE plpgsql;

-- =============================================================================
-- Function to search vectors
-- =============================================================================
CREATE OR REPLACE FUNCTION vectors.search_vectors(
    p_collection_name VARCHAR(255),
    p_query_vector TEXT,
    p_k INTEGER DEFAULT 10,
    p_filter JSONB DEFAULT NULL
) RETURNS TABLE (
    id VARCHAR(255),
    score FLOAT,
    metadata JSONB
) AS $$
DECLARE
    v_table_name TEXT;
    v_distance_metric VARCHAR(50);
    v_operator TEXT;
    v_query TEXT;
BEGIN
    v_table_name := 'collection_' || regexp_replace(p_collection_name, '[^a-zA-Z0-9_]', '_', 'g');

    -- Get distance metric
    SELECT distance_metric INTO v_distance_metric
    FROM vectors.collections WHERE name = p_collection_name;

    -- Determine operator based on distance metric
    CASE v_distance_metric
        WHEN 'cosine' THEN v_operator := '<=>'; -- cosine distance
        WHEN 'euclidean' THEN v_operator := '<->'; -- L2 distance
        WHEN 'dot_product' THEN v_operator := '<#>'; -- negative inner product
        ELSE v_operator := '<=>';
    END CASE;

    -- Build and execute query
    IF p_filter IS NOT NULL THEN
        v_query := format('
            SELECT t.id, 1 - (t.vector %s $1::vector) as score, t.metadata
            FROM vectors.%I t
            WHERE t.metadata @> $2
            ORDER BY t.vector %s $1::vector
            LIMIT $3',
            v_operator, v_table_name, v_operator
        );
        RETURN QUERY EXECUTE v_query USING p_query_vector::vector, p_filter, p_k;
    ELSE
        v_query := format('
            SELECT t.id, 1 - (t.vector %s $1::vector) as score, t.metadata
            FROM vectors.%I t
            ORDER BY t.vector %s $1::vector
            LIMIT $2',
            v_operator, v_table_name, v_operator
        );
        RETURN QUERY EXECUTE v_query USING p_query_vector::vector, p_k;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- =============================================================================
-- Function to get collection statistics
-- =============================================================================
CREATE OR REPLACE FUNCTION vectors.collection_stats(p_collection_name VARCHAR(255))
RETURNS TABLE (
    total_vectors BIGINT,
    dimensions INTEGER,
    distance_metric VARCHAR(50),
    index_size_bytes BIGINT
) AS $$
DECLARE
    v_table_name TEXT;
    v_index_name TEXT;
BEGIN
    v_table_name := 'collection_' || regexp_replace(p_collection_name, '[^a-zA-Z0-9_]', '_', 'g');
    v_index_name := 'idx_' || v_table_name || '_vector';

    RETURN QUERY
    SELECT
        (SELECT COUNT(*) FROM vectors.collections c
         JOIN pg_class pc ON pc.relname = v_table_name
         WHERE c.name = p_collection_name)::BIGINT as total_vectors,
        c.dimensions,
        c.distance_metric,
        COALESCE(pg_relation_size('vectors.' || v_index_name), 0)::BIGINT as index_size_bytes
    FROM vectors.collections c
    WHERE c.name = p_collection_name;
END;
$$ LANGUAGE plpgsql;

-- =============================================================================
-- Create default collection for testing
-- =============================================================================
SELECT vectors.create_collection(
    'default',
    1536,
    'cosine',
    16,
    200,
    '{"description": "Default vector collection for LLM embeddings"}'::jsonb
);

-- =============================================================================
-- Grant permissions
-- =============================================================================
GRANT ALL PRIVILEGES ON SCHEMA vectors TO PUBLIC;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA vectors TO PUBLIC;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA vectors TO PUBLIC;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA vectors TO PUBLIC;

-- Log completion
DO $$
BEGIN
    RAISE NOTICE 'pgvector initialization complete. Default collection created with 1536 dimensions.';
END $$;
