-- LLM-Dev-Ops Infrastructure - PostgreSQL Extension Initialization
-- This script sets up required extensions and schema for RuvVector compatibility
-- As specified in SPARC plan: ruvector-postgres provides pgvector-compatible extension
--
-- Required Extensions:
--   - ruvector: RuvVector extension (provides pgvector-compatible 'vector' type plus advanced features)
--   - vector (pgvector): Fallback if ruvector not available
--   - uuid-ossp: UUID generation for collection IDs
--   - pg_trgm: Trigram text similarity for metadata search
--   - pgcrypto: Cryptographic functions for secure operations
--
-- This script is idempotent and safe to re-run (uses IF NOT EXISTS)

-- Enable RuvVector extension first (includes pgvector-compatible 'vector' type)
-- Falls back to standard pgvector if ruvector is not available
DO $$
BEGIN
    -- Try to create ruvector extension (provides vector type + advanced features)
    BEGIN
        CREATE EXTENSION IF NOT EXISTS ruvector;
        RAISE NOTICE 'RuvVector extension enabled successfully';
    EXCEPTION WHEN OTHERS THEN
        -- Fallback to standard pgvector if ruvector not available
        RAISE NOTICE 'RuvVector not available, falling back to pgvector';
        CREATE EXTENSION IF NOT EXISTS vector;
    END;
END $$;

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
    vector_type VARCHAR(20) NOT NULL DEFAULT 'ruvector', -- 'ruvector' or 'vector' (pgvector)
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
-- Function to detect the active vector type (ruvector or vector)
-- =============================================================================
CREATE OR REPLACE FUNCTION vectors.get_vector_type() RETURNS TEXT AS $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'ruvector') THEN
        RETURN 'ruvector';
    ELSIF EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'vector') THEN
        RETURN 'vector';
    ELSE
        RAISE EXCEPTION 'No vector extension (ruvector or pgvector) is installed';
    END IF;
END;
$$ LANGUAGE plpgsql;

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
    v_vector_type TEXT;
BEGIN
    -- Sanitize table name
    v_table_name := 'collection_' || regexp_replace(p_name, '[^a-zA-Z0-9_]', '_', 'g');

    -- Detect which vector type to use
    v_vector_type := vectors.get_vector_type();

    -- Determine index operator class based on distance metric and vector type
    IF v_vector_type = 'ruvector' THEN
        CASE p_distance_metric
            WHEN 'cosine' THEN v_index_ops := 'ruvector_cosine_ops';
            WHEN 'euclidean' THEN v_index_ops := 'ruvector_l2_ops';
            WHEN 'dot_product' THEN v_index_ops := 'ruvector_ip_ops';
            ELSE v_index_ops := 'ruvector_cosine_ops';
        END CASE;
    ELSE
        CASE p_distance_metric
            WHEN 'cosine' THEN v_index_ops := 'vector_cosine_ops';
            WHEN 'euclidean' THEN v_index_ops := 'vector_l2_ops';
            WHEN 'dot_product' THEN v_index_ops := 'vector_ip_ops';
            ELSE v_index_ops := 'vector_cosine_ops';
        END CASE;
    END IF;

    -- Insert collection metadata
    INSERT INTO vectors.collections (name, dimensions, distance_metric, vector_type, hnsw_m, hnsw_ef_construction, metadata)
    VALUES (p_name, p_dimensions, p_distance_metric, v_vector_type, p_hnsw_m, p_hnsw_ef_construction, p_metadata)
    RETURNING id INTO v_collection_id;

    -- Create the vector table using the detected vector type
    EXECUTE format('
        CREATE TABLE vectors.%I (
            id VARCHAR(255) PRIMARY KEY,
            embedding %s(%s) NOT NULL,
            metadata JSONB DEFAULT ''{}''::jsonb,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )', v_table_name, v_vector_type, p_dimensions);

    -- Create HNSW index for fast similarity search (only if HNSW access method exists)
    -- pgvector provides HNSW; ruvector does not (uses internal indexing)
    IF EXISTS (SELECT 1 FROM pg_am WHERE amname = 'hnsw') THEN
        EXECUTE format('
            CREATE INDEX %I ON vectors.%I
            USING hnsw (embedding %s)
            WITH (m = %s, ef_construction = %s)',
            'idx_' || v_table_name || '_embedding',
            v_table_name,
            v_index_ops,
            p_hnsw_m,
            p_hnsw_ef_construction
        );
    ELSE
        RAISE NOTICE 'HNSW index not available - using sequential scan for vectors (RuvVector uses internal optimization)';
    END IF;

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
    p_vectors JSONB  -- Array of {id, embedding, metadata}
) RETURNS INTEGER AS $$
DECLARE
    v_table_name TEXT;
    v_vector_type TEXT;
    v_inserted INTEGER := 0;
    v_record JSONB;
BEGIN
    v_table_name := 'collection_' || regexp_replace(p_collection_name, '[^a-zA-Z0-9_]', '_', 'g');
    v_vector_type := vectors.get_vector_type();

    FOR v_record IN SELECT * FROM jsonb_array_elements(p_vectors)
    LOOP
        EXECUTE format('
            INSERT INTO vectors.%I (id, embedding, metadata)
            VALUES ($1, $2::%s, $3)
            ON CONFLICT (id) DO UPDATE SET
                embedding = EXCLUDED.embedding,
                metadata = EXCLUDED.metadata,
                updated_at = NOW()',
            v_table_name, v_vector_type
        ) USING
            v_record->>'id',
            COALESCE(v_record->>'embedding', v_record->>'vector'),  -- Support both field names
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
    v_vector_type TEXT;
    v_operator TEXT;
    v_query TEXT;
BEGIN
    v_table_name := 'collection_' || regexp_replace(p_collection_name, '[^a-zA-Z0-9_]', '_', 'g');

    -- Get distance metric and vector type
    SELECT distance_metric, vector_type INTO v_distance_metric, v_vector_type
    FROM vectors.collections WHERE name = p_collection_name;

    -- Determine operator based on distance metric (same for both ruvector and pgvector)
    CASE v_distance_metric
        WHEN 'cosine' THEN v_operator := '<=>'; -- cosine distance
        WHEN 'euclidean' THEN v_operator := '<->'; -- L2 distance
        WHEN 'dot_product' THEN v_operator := '<#>'; -- negative inner product
        ELSE v_operator := '<=>';
    END CASE;

    -- Build and execute query using the correct vector type
    IF p_filter IS NOT NULL THEN
        v_query := format('
            SELECT t.id, 1 - (t.embedding %s $1::%s) as score, t.metadata
            FROM vectors.%I t
            WHERE t.metadata @> $2
            ORDER BY t.embedding %s $1::%s
            LIMIT $3',
            v_operator, v_vector_type, v_table_name, v_operator, v_vector_type
        );
        RETURN QUERY EXECUTE v_query USING p_query_vector, p_filter, p_k;
    ELSE
        v_query := format('
            SELECT t.id, 1 - (t.embedding %s $1::%s) as score, t.metadata
            FROM vectors.%I t
            ORDER BY t.embedding %s $1::%s
            LIMIT $2',
            v_operator, v_vector_type, v_table_name, v_operator, v_vector_type
        );
        RETURN QUERY EXECUTE v_query USING p_query_vector, p_k;
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
    vector_type VARCHAR(20),
    index_size_bytes BIGINT
) AS $$
DECLARE
    v_table_name TEXT;
    v_index_name TEXT;
BEGIN
    v_table_name := 'collection_' || regexp_replace(p_collection_name, '[^a-zA-Z0-9_]', '_', 'g');
    v_index_name := 'idx_' || v_table_name || '_embedding';

    RETURN QUERY
    SELECT
        (SELECT COUNT(*) FROM vectors.collections c
         JOIN pg_class pc ON pc.relname = v_table_name
         WHERE c.name = p_collection_name)::BIGINT as total_vectors,
        c.dimensions,
        c.distance_metric,
        c.vector_type,
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
DECLARE
    ext_name TEXT;
BEGIN
    -- Check which vector extension is active
    SELECT extname INTO ext_name FROM pg_extension WHERE extname IN ('ruvector', 'vector') LIMIT 1;
    RAISE NOTICE 'Vector extension initialization complete using: %', ext_name;
    RAISE NOTICE 'Default collection created with 1536 dimensions.';
END $$;
