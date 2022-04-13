SET client_min_messages TO WARNING;

-- Create the PL/Python3 extension.
CREATE EXTENSION IF NOT EXISTS plpython3u;

---
--- Create schema for models.
---
DROP SCHEMA pgml CASCADE;
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

CREATE TABLE pgml.projects(
	id BIGSERIAL PRIMARY KEY,
	name TEXT NOT NULL,
	created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp()
);
SELECT pgml.auto_updated_at('pgml.projects');
CREATE UNIQUE INDEX projects_name_idx ON pgml.projects(name);

CREATE TABLE pgml.snapshots(
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

CREATE TABLE pgml.models(
	id BIGSERIAL PRIMARY KEY,
	project_id BIGINT NOT NULL,
	snapshot_id BIGINT NOT NULL,
	algorithm TEXT NOT NULL,
	status TEXT NOT NULL,
	created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	mean_squared_error DOUBLE PRECISION,
	r2_score DOUBLE PRECISION,
	pickle BYTEA,
	CONSTRAINT project_id_fk FOREIGN KEY(project_id) REFERENCES pgml.projects(id),
	CONSTRAINT snapshot_id_fk FOREIGN KEY(snapshot_id) REFERENCES pgml.snapshots(id)
);
CREATE INDEX models_project_id_created_at_idx ON pgml.models(project_id, created_at);
SELECT pgml.auto_updated_at('pgml.models');

CREATE TABLE pgml.promotions(
	project_id BIGINT NOT NULL,
	model_id BIGINT NOT NULL,
	created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	CONSTRAINT project_id_fk FOREIGN KEY(project_id) REFERENCES pgml.projects(id),
	CONSTRAINT model_id_fk FOREIGN KEY(model_id) REFERENCES pgml.models(id)
);
CREATE INDEX promotions_project_id_created_at_idx ON pgml.promotions(project_id, created_at);
SELECT pgml.auto_updated_at('pgml.promotions');


---
--- Extension version.
---
CREATE OR REPLACE FUNCTION pgml.version()
RETURNS TEXT
AS $$
	import pgml
	return pgml.version()
$$ LANGUAGE plpython3u;

CREATE OR REPLACE FUNCTION pgml.model_regression(project_name TEXT, relation_name TEXT, y_column_name TEXT)
RETURNS VOID
AS $$
	import pgml
	from pgml.model import Regression
	Regression(project_name, relation_name, y_column_name)
$$ LANGUAGE plpython3u;


---
--- Track table versions.
---
CREATE TABLE pgml.model_versions(
	id BIGSERIAL PRIMARY KEY,
	name VARCHAR NOT NULL,
	data_source TEXT,
	y_column VARCHAR,
	started_at TIMESTAMP WITHOUT TIME ZONE DEFAULT CURRENT_TIMESTAMP,
	ended_at TIMESTAMP WITHOUT TIME ZONE NULL,
	mean_squared_error DOUBLE PRECISION,
	r2_score DOUBLE PRECISION,
	successful BOOL NULL,
	pickle BYTEA 
);

---
--- Train the model.
---
CREATE OR REPLACE FUNCTION pgml.train(table_name TEXT, y TEXT)
RETURNS TEXT
AS $$
	from pgml.train import train
	from pgml.sql import models_directory
	import os

	data_source = f"SELECT * FROM {table_name}"

	# Start training.
	start = plpy.execute(f"""
		INSERT INTO pgml.model_versions
			(name, data_source, y_column)
		VALUES
			('{table_name}', '{data_source}', '{y}')
		RETURNING *""", 1)

	id_ = start[0]["id"]
	name = f"{table_name}_{id_}"

	destination = models_directory()

	# Train!
	pickle, msq, r2 = train(plpy.cursor(data_source), y_column=y, name=name, destination=destination)

	plpy.execute(f"""
		UPDATE pgml.model_versions
		SET pickle = '{pickle}',
			successful = true,
			mean_squared_error = '{msq}',
			r2_score = '{r2}',
			ended_at = clock_timestamp()
		WHERE id = {id_}""")

	return name
$$ LANGUAGE plpython3u;


---
--- Predict
---
CREATE OR REPLACE FUNCTION pgml.score(model_name TEXT, VARIADIC features DOUBLE PRECISION[])
RETURNS DOUBLE PRECISION
AS $$
	from pgml.sql import models_directory
	from pgml.score import load
	import pickle

	if model_name in SD:
		model = SD[model_name]
	else:
		SD[model_name] = load(model_name, models_directory())
		model = SD[model_name]

	scores = model.predict([features,])
	return scores[0]
$$ LANGUAGE plpython3u;
