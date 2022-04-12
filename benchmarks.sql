--
-- CREATE EXTENSION
--
CREATE EXTENSION IF NOT EXISTS plpython3u;

CREATE OR REPLACE FUNCTION pg_call()
RETURNS INT
AS $$
BEGIN
    RETURN 1;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION py_call()
RETURNS INT
AS $$
    return 1;
$$ LANGUAGE plpython3u;

\timing on
SELECT generate_series(1, 50000), pg_call(); -- Time: 20.679 ms
SELECT generate_series(1, 50000), py_call(); -- Time: 67.355 ms

