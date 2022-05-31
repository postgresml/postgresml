---
--- Create schema for models.
---
CREATE SCHEMA IF NOT EXISTS pgml;


--- 
--- Track of updates to data
---
CREATE OR REPLACE FUNCTION pgml.auto_updated_at(tbl regclass) 
RETURNS VOID 
AS $$
    DECLARE name_parts TEXT[];
    DECLARE name TEXT; 
BEGIN
    name_parts := string_to_array(tbl::TEXT, '.');
    name := name_parts[array_upper(name_parts, 1)];

    EXECUTE format('DROP TRIGGER IF EXISTS %s_auto_updated_at ON %s', name, tbl);
    EXECUTE format('CREATE TRIGGER %s_auto_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE pgml.set_updated_at()', name, tbl);
END;
$$
LANGUAGE plpgsql;


---
--- Called via trigger whenever a row changes
---
CREATE OR REPLACE FUNCTION pgml.set_updated_at() 
RETURNS TRIGGER 
AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
        AND NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at
    ) THEN
        NEW.updated_at := clock_timestamp();
    END IF;
    RETURN new;
END;
$$
LANGUAGE plpgsql;


---
--- Projects organize work
---
CREATE TABLE IF NOT EXISTS pgml.projects(
	id BIGSERIAL PRIMARY KEY,
	name TEXT NOT NULL,
	objective TEXT NOT NULL,
	created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp()
);
SELECT pgml.auto_updated_at('pgml.projects');
CREATE UNIQUE INDEX IF NOT EXISTS projects_name_idx ON pgml.projects(name);


---
--- Snapshots freeze data for training
---
CREATE TABLE IF NOT EXISTS pgml.snapshots(
	id BIGSERIAL PRIMARY KEY,
	relation_name TEXT NOT NULL,
	y_column_name TEXT[] NOT NULL,
	test_size FLOAT4 NOT NULL,
	test_sampling TEXT NOT NULL,
	status TEXT NOT NULL,
	columns JSONB,
	analysis JSONB,
	created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp()
);
SELECT pgml.auto_updated_at('pgml.snapshots');


---
--- Models save the learned parameters
---
CREATE TABLE IF NOT EXISTS pgml.models(
	id BIGSERIAL PRIMARY KEY,
	project_id BIGINT NOT NULL,
	snapshot_id BIGINT NOT NULL,
	algorithm_name TEXT NOT NULL,
	hyperparams JSONB NOT NULL,
	status TEXT NOT NULL,
	search TEXT,
	search_params JSONB NOT NULL,
	search_args JSONB NOT NULL,
	created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	metrics JSONB,
	pickle BYTEA,
	CONSTRAINT project_id_fk FOREIGN KEY(project_id) REFERENCES pgml.projects(id),
	CONSTRAINT snapshot_id_fk FOREIGN KEY(snapshot_id) REFERENCES pgml.snapshots(id)
);
CREATE INDEX IF NOT EXISTS models_project_id_idx ON pgml.models(project_id);
CREATE INDEX IF NOT EXISTS models_snapshot_id_idx ON pgml.models(snapshot_id);
SELECT pgml.auto_updated_at('pgml.models');


---
--- Deployements determine which model is live
---
CREATE TABLE IF NOT EXISTS pgml.deployments(
	id BIGSERIAL PRIMARY KEY,
	project_id BIGINT NOT NULL,
	model_id BIGINT NOT NULL,
	strategy TEXT NOT NULL,
	created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	CONSTRAINT project_id_fk FOREIGN KEY(project_id) REFERENCES pgml.projects(id),
	CONSTRAINT model_id_fk FOREIGN KEY(model_id) REFERENCES pgml.models(id)
);
CREATE INDEX IF NOT EXISTS deployments_project_id_created_at_idx ON pgml.deployments(project_id);
CREATE INDEX IF NOT EXISTS deployments_model_id_created_at_idx ON pgml.deployments(model_id);
SELECT pgml.auto_updated_at('pgml.deployments');


---
--- Quick status check on the system.
---
DROP VIEW IF EXISTS pgml.overview;
CREATE VIEW pgml.overview AS
SELECT
	   p.name,
	   d.created_at AS deployed_at,
       p.objective,
       m.algorithm_name,
       s.relation_name,
       s.y_column_name,
       s.test_sampling,
       s.test_size
FROM pgml.projects p
INNER JOIN pgml.models m ON p.id = m.project_id
INNER JOIN pgml.deployments d ON d.project_id = p.id
AND d.model_id = m.id
INNER JOIN pgml.snapshots s ON s.id = m.snapshot_id
ORDER BY d.created_at DESC;


---
--- List details of trained models.
---
DROP VIEW IF EXISTS pgml.trained_models;
CREATE VIEW pgml.trained_models AS
SELECT
	m.id,	
	p.name,
	p.objective,
	m.algorithm_name,
	m.created_at,
	s.test_sampling,
	s.test_size,
	d.model_id IS NOT NULL AS deployed
FROM pgml.projects p
INNER JOIN pgml.models m ON p.id = m.project_id
INNER JOIN pgml.snapshots s ON s.id = m.snapshot_id
LEFT JOIN (
	SELECT DISTINCT ON(project_id)
		project_id, model_id, created_at
	FROM pgml.deployments
	ORDER BY project_id, created_at desc
) d ON d.model_id = m.id
ORDER BY m.created_at DESC;


---
--- List details of deployed models.
---
DROP VIEW IF EXISTS pgml.deployed_models;
CREATE VIEW pgml.deployed_models AS
SELECT
	m.id,
	p.name,
	p.objective,
	m.algorithm_name,
	d.created_at as deployed_at 
FROM pgml.projects p
INNER JOIN (
	SELECT DISTINCT ON(project_id)
		project_id, model_id, created_at
	FROM pgml.deployments
	ORDER BY project_id, created_at desc
) d ON d.project_id = p.id
INNER JOIN pgml.models m ON m.id = d.model_id
ORDER BY p.name ASC;
