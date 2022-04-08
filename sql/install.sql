
-- Create the PL/Python3 extension.
CREATE EXTENSION IF NOT EXISTS plpython3u;

---
--- Extension version.
---
CREATE OR REPLACE FUNCTION pgml_version()
RETURNS TEXT
AS $$
	import pgml
	return pgml.version()
$$ LANGUAGE plpython3u;

---
--- Track table versions.
---
CREATE SCHEMA IF NOT EXISTS pgml;
CREATE TABLE pgml.model_versions(
	id BIGSERIAL PRIMARY KEY,
	name VARCHAR,
	location VARCHAR NULL,
	data_source TEXT,
	y_column VARCHAR,
	started_at TIMESTAMP WITHOUT TIME ZONE DEFAULT CURRENT_TIMESTAMP,
	ended_at TIMESTAMP WITHOUT TIME ZONE NULL,
	mean_squared_error DOUBLE PRECISION,
	r2_score DOUBLE PRECISION,
	successful BOOL NULL
);

---
--- Run some validations on the table/view to make sure
--- it'll work without our package.
---
CREATE OR REPLACE FUNCTION pgml_validate(table_name TEXT)
RETURNS BOOL
AS $$
	from pgml.sql import all_rows
	from pgml.validate import check_type

	for row in all_rows(plpy.cursor(f"SELECT * FROM {table_name}")):
		check_type(row)
	return True
$$ LANGUAGE plpython3u;

---
--- Train the model.
---
CREATE OR REPLACE FUNCTION pgml_train(table_name TEXT, y TEXT)
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

	destination = models_directory(plpy)

	# Train!
	location, msq, r2 = train(plpy.cursor(data_source), y_column=y, name=name, destination=destination)

	plpy.execute(f"""
		UPDATE pgml.model_versions
		SET location = '{location}',
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
DROP FUNCTION pgml_score(model_name TEXT, VARIADIC features DOUBLE PRECISION[]);
CREATE OR REPLACE FUNCTION pgml_score(model_name TEXT, VARIADIC features DOUBLE PRECISION[])
RETURNS DOUBLE PRECISION
AS $$
	from pgml.sql import models_directory
	from pgml.score import load
	import pickle

	if model_name in SD:
		model = SD[model_name]
	else:
		SD[model_name] = load(model_name, models_directory(plpy))
		model = SD[model_name]

	scores = model.predict([features,])
	return scores[0]
$$ LANGUAGE plpython3u;
