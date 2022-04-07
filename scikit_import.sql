CREATE EXTENSION IF NOT EXISTS plpython3u;

CREATE OR REPLACE FUNCTION pgml_version()
RETURNS TEXT
AS $$
    import pgml
    return pgml.version()
$$ LANGUAGE plpython3u;

SELECT pgml_version();
