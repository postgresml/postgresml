SET client_min_messages TO WARNING;

-- Create the PL/Python3 extension.
CREATE EXTENSION IF NOT EXISTS plpython3u;

---
--- Create schema for models.
---
DROP SCHEMA IF EXISTS pgml CASCADE;
CREATE SCHEMA IF NOT EXISTS pgml;

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

CREATE TABLE IF NOT EXISTS pgml.projects(
	id BIGSERIAL PRIMARY KEY,
	name TEXT NOT NULL,
	objective TEXT NOT NULL,
	created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp()
);
SELECT pgml.auto_updated_at('pgml.projects');
CREATE UNIQUE INDEX IF NOT EXISTS projects_name_idx ON pgml.projects(name);

CREATE TABLE IF NOT EXISTS pgml.snapshots(
	id BIGSERIAL PRIMARY KEY,
	relation_name TEXT NOT NULL,
	y_column_name TEXT NOT NULL,
	test_size FLOAT4 NOT NULL,
	test_sampling TEXT NOT NULL,
	status TEXT NOT NULL,
	created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp()
);
SELECT pgml.auto_updated_at('pgml.snapshots');

CREATE TABLE IF NOT EXISTS pgml.models(
	id BIGSERIAL PRIMARY KEY,
	project_id BIGINT NOT NULL,
	snapshot_id BIGINT NOT NULL,
	algorithm_name TEXT NOT NULL,
	status TEXT NOT NULL,
	created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	metrics JSONB,
	pickle BYTEA,
	CONSTRAINT project_id_fk FOREIGN KEY(project_id) REFERENCES pgml.projects(id),
	CONSTRAINT snapshot_id_fk FOREIGN KEY(snapshot_id) REFERENCES pgml.snapshots(id)
);
CREATE INDEX IF NOT EXISTS models_project_id_created_at_idx ON pgml.models(project_id, created_at);
SELECT pgml.auto_updated_at('pgml.models');

CREATE TABLE IF NOT EXISTS pgml.deployments(
	project_id BIGINT NOT NULL,
	model_id BIGINT NOT NULL,
	created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	CONSTRAINT project_id_fk FOREIGN KEY(project_id) REFERENCES pgml.projects(id),
	CONSTRAINT model_id_fk FOREIGN KEY(model_id) REFERENCES pgml.models(id)
);
CREATE INDEX IF NOT EXISTS deployments_project_id_created_at_idx ON pgml.deployments(project_id, created_at);
SELECT pgml.auto_updated_at('pgml.deployments');


---
--- Extension version
---
CREATE OR REPLACE FUNCTION pgml.version()
RETURNS TEXT
AS $$
	import pgml
	return pgml.version()
$$ LANGUAGE plpython3u;

---
--- Load data
---
CREATE OR REPLACE FUNCTION pgml.load_dataset(source TEXT)
RETURNS TEXT
AS $$
	from pgml.datasets import load
	return load(source)
$$ LANGUAGE plpython3u;

---
--- Train
---
CREATE OR REPLACE FUNCTION pgml.train(project_name TEXT, objective TEXT, relation_name TEXT, y_column_name TEXT, algorithm TEXT DEFAULT NULL)
RETURNS TABLE(project_name TEXT, objective TEXT, status TEXT)
AS $$
	from pgml.model import train

	status = train(project_name, objective, relation_name, y_column_name, algorithm)

	return [(project_name, objective, status)]
$$ LANGUAGE plpython3u;

---
--- Deploy
---
CREATE OR REPLACE FUNCTION pgml.deploy(project_name TEXT, algorithm_name TEXT)
RETURNS TABLE(project_name TEXT, objective TEXT, algorithm_name TEXT)
AS $$
	from pgml.model import Project
	model = Project.find_by_name(project_name).deploy(algorithm_name)
	return [(model.project.name, model.project.objective, model.algorithm_name)]
$$ LANGUAGE plpython3u;

---
--- Predict
---
CREATE OR REPLACE FUNCTION pgml.predict(project_name TEXT, features DOUBLE PRECISION[])
RETURNS DOUBLE PRECISION
AS $$
	from pgml.model import Project
	return Project.find_by_name(project_name).deployed_model.predict([features,])[0]
$$ LANGUAGE plpython3u;

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
       m.metrics,
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
	   p.name,
       p.objective,
       m.algorithm_name,
       m.metrics,
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
	   p.name,
       p.objective,
       m.algorithm_name,
       m.metrics,
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
