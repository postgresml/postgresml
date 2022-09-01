CREATE SCHEMA IF NOT EXISTS pgml_rust;

--- 
--- Track of updates to data
---
CREATE OR REPLACE FUNCTION pgml_rust.auto_updated_at(tbl regclass) 
RETURNS VOID 
AS $$
    DECLARE name_parts TEXT[];
    DECLARE name TEXT; 
BEGIN
    name_parts := string_to_array(tbl::TEXT, '.');
    name := name_parts[array_upper(name_parts, 1)];

    EXECUTE format('DROP TRIGGER IF EXISTS %s_auto_updated_at ON %s', name, tbl);
    EXECUTE format('CREATE TRIGGER %s_auto_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE pgml_rust.set_updated_at()', name, tbl);
END;
$$
LANGUAGE plpgsql;


---
--- Called via trigger whenever a row changes
---
CREATE OR REPLACE FUNCTION pgml_rust.set_updated_at() 
RETURNS TRIGGER 
AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
        AND NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at
    ) THEN
        NEW.updated_at := clock_timestamp();
    END IF;
    RETURN NEW;
END;
$$
LANGUAGE plpgsql;

---
--- Projects organize work
---
CREATE TABLE IF NOT EXISTS pgml_rust.projects(
	id BIGSERIAL PRIMARY KEY,
	name TEXT NOT NULL UNIQUE,
	task TEXT NOT NULL,
	created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp()
);
SELECT pgml_rust.auto_updated_at('pgml_rust.projects');


CREATE TABLE IF NOT EXISTS pgml_rust.models (
	id BIGSERIAL PRIMARY KEY,
	project_id BIGINT NOT NULL REFERENCES pgml_rust.projects(id),
	algorithm VARCHAR,
	data BYTEA
);

---
--- Deployments determine which model is live
---
CREATE TABLE IF NOT EXISTS pgml_rust.deployments(
	id BIGSERIAL PRIMARY KEY,
	project_id BIGINT NOT NULL,
	model_id BIGINT NOT NULL,
	strategy TEXT NOT NULL,
	created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	CONSTRAINT project_id_fk FOREIGN KEY(project_id) REFERENCES pgml_rust.projects(id),
	CONSTRAINT model_id_fk FOREIGN KEY(model_id) REFERENCES pgml_rust.models(id)
);
CREATE INDEX IF NOT EXISTS deployments_project_id_created_at_idx ON pgml_rust.deployments(project_id);
CREATE INDEX IF NOT EXISTS deployments_model_id_created_at_idx ON pgml_rust.deployments(model_id);
SELECT pgml_rust.auto_updated_at('pgml_rust.deployments');
