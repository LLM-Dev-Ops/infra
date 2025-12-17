-- LLM-Dev-Ops Infrastructure - PostgreSQL Extension Verification
-- This script runs last (99-*) to verify all required extensions are installed
-- It will raise an ERROR if any extension is missing, preventing incomplete setups

DO $$
DECLARE
    required_extensions TEXT[] := ARRAY['vector', 'uuid-ossp', 'pg_trgm', 'pgcrypto'];
    missing_extensions TEXT[] := '{}';
    ext TEXT;
BEGIN
    FOREACH ext IN ARRAY required_extensions
    LOOP
        IF NOT EXISTS (SELECT 1 FROM pg_extension WHERE extname = ext) THEN
            missing_extensions := array_append(missing_extensions, ext);
        END IF;
    END LOOP;

    IF array_length(missing_extensions, 1) > 0 THEN
        RAISE EXCEPTION 'Missing required Postgres extensions: %', array_to_string(missing_extensions, ', ');
    END IF;

    RAISE NOTICE '✓ All required extensions verified: %', array_to_string(required_extensions, ', ');
END $$;

-- Extension verification query (can be run manually)
-- This SELECT provides a summary of installed extensions for RuvVector
SELECT
    extname AS extension,
    extversion AS version,
    CASE
        WHEN extname = 'vector' THEN 'pgvector - Vector similarity search for embeddings'
        WHEN extname = 'uuid-ossp' THEN 'UUID generation (uuid_generate_v4)'
        WHEN extname = 'pg_trgm' THEN 'Trigram text similarity search'
        WHEN extname = 'pgcrypto' THEN 'Cryptographic functions (digest, encrypt, etc)'
        ELSE 'Other extension'
    END AS description
FROM pg_extension
WHERE extname IN ('vector', 'uuid-ossp', 'pg_trgm', 'pgcrypto')
ORDER BY extname;

-- Verify pgvector functionality
DO $$
BEGIN
    -- Test vector creation
    PERFORM '[1,2,3]'::vector(3);
    RAISE NOTICE '✓ pgvector: Vector type operational';

    -- Test distance operations
    PERFORM '[1,2,3]'::vector(3) <=> '[4,5,6]'::vector(3);
    RAISE NOTICE '✓ pgvector: Cosine distance operational';

    PERFORM '[1,2,3]'::vector(3) <-> '[4,5,6]'::vector(3);
    RAISE NOTICE '✓ pgvector: Euclidean distance operational';
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
BEGIN
    RAISE NOTICE '============================================================';
    RAISE NOTICE 'PostgreSQL extension verification complete for LLM-Dev-Ops';
    RAISE NOTICE 'Extensions: vector, uuid-ossp, pg_trgm, pgcrypto';
    RAISE NOTICE '============================================================';
END $$;
