-- SELECT ARRAY_AGG(random()) AS vector
-- FROM generate_series(1, 1280000) i
-- GROUP BY i % 10000;

SELECT 1 FROM (
SELECT ARRAY_AGG(random()) AS vector
FROM generate_series(1, 1280000) i
GROUP BY i % 10000
) f LIMIT 0;

-- CREATE TABLE embeddings AS
-- SELECT ARRAY_AGG(random()) AS vector
-- FROM generate_series(1, 1280000) i
-- GROUP BY i % 10000;

-- COPY embeddings TO '/tmp/embeddings.csv' DELIMITER ',' CSV HEADER;
