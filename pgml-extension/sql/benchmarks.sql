SET client_min_messages TO WARNING;
\pset pager off

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

SELECT set_config('pgml.setting', 'its_set', false);

CREATE OR REPLACE FUNCTION pg_settings()
RETURNS TEXT
AS $$
BEGIN
    RETURN current_setting('pgml.setting');
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION py_settings()
RETURNS TEXT
AS $$
    return plpy.execute("SELECT current_setting('pgml.setting')")[0];
$$ LANGUAGE plpython3u;

CREATE OR REPLACE FUNCTION py_gd()
RETURNS TEXT
AS $$
    return GD.get("pgml.setting", 'its_set');
$$ LANGUAGE plpython3u;

CREATE OR REPLACE FUNCTION py_sd()
RETURNS TEXT
AS $$
    return SD.get("pgml.setting", 'its_set');
$$ LANGUAGE plpython3u;



\timing on
WITH test AS (
    SELECT generate_series(1, 100000), pg_call() -- Time: 20.679 ms
) SELECT count(1) AS pg_call FROM test;

WITH test AS (
    SELECT generate_series(1, 100000), py_call() -- Time: 67.355 ms
) SELECT count(1) AS py_call FROM test;

WITH test AS (
    SELECT generate_series(1, 100000), pg_settings() -- Time: 20.679 ms
) SELECT count(1) AS pg_settings FROM test;

WITH test AS (
    SELECT generate_series(1, 100000), py_settings() -- Time: 67.355 ms
) SELECT count(1) AS py_settings FROM test;

WITH test AS (
    SELECT generate_series(1, 100000), py_gd() -- Time: 67.355 ms
) SELECT count(1) AS py_gd FROM test;

WITH test AS (
    SELECT generate_series(1, 100000), py_sd() -- Time: 67.355 ms
) SELECT count(1) as py_sd FROM test;

