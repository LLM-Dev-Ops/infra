-- LLM-Dev-Ops Infrastructure - PostgreSQL Extension Verification
-- This script runs last (99-*) to verify all required extensions are installed
-- It will raise an ERROR if any extension is missing, preventing incomplete setups
-- Supports both RuvVector extension and standard pgvector as fallback

DO $$
DECLARE
    -- Core extensions that must be present
    core_extensions TEXT[] := ARRAY['uuid-ossp', 'pg_trgm', 'pgcrypto'];
    missing_extensions TEXT[] := '{}';
    ext TEXT;
    has_vector_ext BOOLEAN := FALSE;
BEGIN
    -- Check for vector extension (ruvector preferred, pgvector as fallback)
    IF EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'ruvector') THEN
        has_vector_ext := TRUE;
        RAISE NOTICE '✓ ruvector extension found (includes pgvector compatibility)';
    ELSIF EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'vector') THEN
        has_vector_ext := TRUE;
        RAISE NOTICE '✓ pgvector extension found';
    END IF;

    IF NOT has_vector_ext THEN
        missing_extensions := array_append(missing_extensions, 'vector/ruvector');
    END IF;

    -- Check core extensions
    FOREACH ext IN ARRAY core_extensions
    LOOP
        IF NOT EXISTS (SELECT 1 FROM pg_extension WHERE extname = ext) THEN
            missing_extensions := array_append(missing_extensions, ext);
        END IF;
    END LOOP;

    IF array_length(missing_extensions, 1) > 0 THEN
        RAISE EXCEPTION 'Missing required Postgres extensions: %', array_to_string(missing_extensions, ', ');
    END IF;

    RAISE NOTICE '✓ All required extensions verified';
END $$;

-- Extension verification query (can be run manually)
-- This SELECT provides a summary of installed extensions for RuvVector
SELECT
    extname AS extension,
    extversion AS version,
    CASE
        WHEN extname = 'ruvector' THEN 'RuvVector - Advanced vector database with GNN, hyperbolic, graph support'
        WHEN extname = 'vector' THEN 'pgvector - Vector similarity search for embeddings'
        WHEN extname = 'uuid-ossp' THEN 'UUID generation (uuid_generate_v4)'
        WHEN extname = 'pg_trgm' THEN 'Trigram text similarity search'
        WHEN extname = 'pgcrypto' THEN 'Cryptographic functions (digest, encrypt, etc)'
        ELSE 'Other extension'
    END AS description
FROM pg_extension
WHERE extname IN ('ruvector', 'vector', 'uuid-ossp', 'pg_trgm', 'pgcrypto')
ORDER BY extname;

-- Verify vector functionality (works with both ruvector and pgvector)
DO $$
DECLARE
    v_type TEXT;
BEGIN
    -- Detect vector type
    IF EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'ruvector') THEN
        v_type := 'ruvector';
    ELSE
        v_type := 'vector';
    END IF;

    -- Test vector creation using dynamic SQL
    IF v_type = 'ruvector' THEN
        PERFORM '[1,2,3]'::ruvector(3);
        RAISE NOTICE '✓ RuVector type operational';

        PERFORM '[1,2,3]'::ruvector(3) <=> '[4,5,6]'::ruvector(3);
        RAISE NOTICE '✓ Cosine distance operational (ruvector)';

        PERFORM '[1,2,3]'::ruvector(3) <-> '[4,5,6]'::ruvector(3);
        RAISE NOTICE '✓ Euclidean distance operational (ruvector)';
    ELSE
        PERFORM '[1,2,3]'::vector(3);
        RAISE NOTICE '✓ Vector type operational';

        PERFORM '[1,2,3]'::vector(3) <=> '[4,5,6]'::vector(3);
        RAISE NOTICE '✓ Cosine distance operational (pgvector)';

        PERFORM '[1,2,3]'::vector(3) <-> '[4,5,6]'::vector(3);
        RAISE NOTICE '✓ Euclidean distance operational (pgvector)';
    END IF;
END $$;

-- Verify RuvVector-specific functionality (if available)
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'ruvector') THEN
        -- Test RuvVector-specific functions
        BEGIN
            PERFORM ruvector_version();
            RAISE NOTICE '✓ RuvVector: Version function operational';
        EXCEPTION WHEN OTHERS THEN
            RAISE NOTICE '⚠ RuvVector: Version function not available';
        END;
    END IF;
END $$;

-- Verify uuid-ossp functionality
DO $$
DECLARE
    test_uuid UUID;
BEGIN
    test_uuid := uuid_generate_v4();
    IF test_uuid IS NOT NULL THEN
        RAISE NOTICE '✓ uuid-ossp: UUID generation operational (sample: %)', test_uuid;
    END IF;
END $$;

-- Verify pgcrypto functionality
DO $$
DECLARE
    test_digest TEXT;
BEGIN
    test_digest := encode(digest('test', 'sha256'), 'hex');
    IF test_digest IS NOT NULL AND length(test_digest) = 64 THEN
        RAISE NOTICE '✓ pgcrypto: SHA256 digest operational';
    END IF;
END $$;

-- Final summary
DO $$
DECLARE
    vector_ext TEXT;
BEGIN
    SELECT extname INTO vector_ext FROM pg_extension WHERE extname IN ('ruvector', 'vector') LIMIT 1;
    RAISE NOTICE '============================================================';
    RAISE NOTICE 'PostgreSQL extension verification complete for LLM-Dev-Ops';
    RAISE NOTICE 'Vector engine: %', vector_ext;
    RAISE NOTICE 'Core extensions: uuid-ossp, pg_trgm, pgcrypto';
    RAISE NOTICE '============================================================';
END $$;
