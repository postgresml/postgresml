---
--- Transformer Pipeline w/ Binary Inputs
---
CREATE OR REPLACE FUNCTION pgml.transform(
	task JSONB DEFAULT '{}'::JSONB,         -- pipeline constructor arguments
	call JSONB DEFAULT '{}'::JSONB,         -- pipeline call arguments
	inputs BYTEA[] DEFAULT ARRAY[]::BYTEA[] -- pipeline inputs
)
RETURNS JSONB
AS $$
	from pgml_extension.transformers import transform
	import json

	result = transform(
		json.loads(task),
		json.loads(call),
		inputs,
	)

	return json.dumps(result)
$$ LANGUAGE plpython3u;

---
--- Transformer Pipeline w/ Binary Inputs
---
CREATE OR REPLACE FUNCTION pgml.transform(
	task TEXT,                              -- pipeline task name
	call JSONB DEFAULT '{}'::JSONB,         -- pipeline call arguments
	inputs BYTEA[] DEFAULT ARRAY[]::BYTEA[] -- pipeline inputs
)
RETURNS JSONB
AS $$
	from pgml_extension.transformers import transform
	import json

	result = transform(
		task,
		json.loads(call),
		inputs,
	)

	return json.dumps(result)
$$ LANGUAGE plpython3u;

---
--- Transformer Pipeline w/ Text Inputs
---
CREATE OR REPLACE FUNCTION pgml.transform(
	task JSONB DEFAULT '{}'::JSONB,       -- pipeline constructor arguments
	call JSONB DEFAULT '{}'::JSONB,       -- pipeline call arguments
	inputs TEXT[] DEFAULT ARRAY[]::TEXT[] -- pipeline inputs
)
RETURNS JSONB
AS $$
	from pgml_extension.transformers import transform
	import json

	result = transform(
		json.loads(task),
		json.loads(call),
		inputs,
	)

	return json.dumps(result)
$$ LANGUAGE plpython3u;

---
--- Transformer Pipeline w/ Text Inputs
---
CREATE OR REPLACE FUNCTION pgml.transform(
	task TEXT,                            -- pipeline task name
	call JSONB DEFAULT '{}'::JSONB,       -- pipeline call arguments
	inputs TEXT[] DEFAULT ARRAY[]::TEXT[] -- pipeline inputs
)
RETURNS JSONB
AS $$
	from pgml_extension.transformers import transform
	import json

	result = transform(
		task,
		json.loads(call),
		inputs,
	)

	return json.dumps(result)
$$ LANGUAGE plpython3u;

---
--- Fine tune a Transformer
---
CREATE OR REPLACE FUNCTION pgml.tune(
	project_name TEXT, 							-- Human-friendly project name
	task TEXT DEFAULT NULL,                     -- See hugging face documentation for task types
	relation_name TEXT DEFAULT NULL,            -- name of table or view
	y_column_name TEXT DEFAULT NULL,            -- aka "label" or "unknown" or "target"
	model_name TEXT DEFAULT NULL,               -- pre-trained model
	hyperparams JSONB DEFAULT '{}'::JSONB,      -- options for the model
	search TEXT DEFAULT NULL,                   -- hyperparam tuning, 'grid' or 'random'
	search_params JSONB DEFAULT '{}'::JSONB,    -- hyperparam search space
	search_args JSONB DEFAULT '{}'::JSONB,      -- hyperparam options
	test_size REAL DEFAULT 0.25,                -- fraction of the data for the test set
	test_sampling TEXT DEFAULT 'random'         -- 'random', 'first' or 'last'
)
RETURNS TABLE(project_name TEXT, task TEXT, model_name TEXT, status TEXT)
AS $$
	from pgml_extension.transformers import tune
	import json
	status = tune(
		project_name, 
		task, 
		relation_name, 
		[y_column_name],
		model_name, 
		json.loads(hyperparams),
		search,
		json.loads(search_params),
		json.loads(search_args),
		test_size,
		test_sampling
	)

	return [(project_name, task, model_name, status)]
$$ LANGUAGE plpython3u;

---
--- Predict
---
CREATE OR REPLACE FUNCTION pgml.predict(
	project_name TEXT, -- Human-friendly project name
	inputs TEXT 
)
RETURNS JSONB
AS $$
	from pgml_extension.model import Project

	return Project.find_by_name(project_name).deployed_model.predict(inputs)
$$ LANGUAGE plpython3u;

---
--- Predict Batch
---
CREATE OR REPLACE FUNCTION pgml.predict(
	project_name TEXT, -- Human-friendly project name
	inputs TEXT[] 
)
RETURNS JSONB
AS $$
	from pgml_extension.model import Project

	return Project.find_by_name(project_name).deployed_model.predict(inputs)
$$ LANGUAGE plpython3u;

---
--- Predict Probability
---
CREATE OR REPLACE FUNCTION pgml.predict_proba(
	project_name TEXT, -- Human-friendly project name
	inputs TEXT 
)
RETURNS SETOF JSONB
AS $$
	from pgml_extension.model import Project

	return Project.find_by_name(project_name).deployed_model.predict_proba(inputs)
$$ LANGUAGE plpython3u;

---
--- Predict Probability Batch
---
CREATE OR REPLACE FUNCTION pgml.predict_proba(
	project_name TEXT, -- Human-friendly project name
	inputs TEXT[] 
)
RETURNS SETOF JSONB
AS $$
	from pgml_extension.model import Project

	return Project.find_by_name(project_name).deployed_model.predict_proba(inputs)
$$ LANGUAGE plpython3u;

---
--- Generate
---
CREATE OR REPLACE FUNCTION pgml.generate(
	project_name TEXT, -- Human-friendly project name
	inputs TEXT 
)
RETURNS TEXT
AS $$
	import plpy
	from pgml_extension.model import Project

	return Project.find_by_name(project_name).deployed_model.generate([inputs])[0]
$$ LANGUAGE plpython3u;

---
--- Generate Batch
---
CREATE OR REPLACE FUNCTION pgml.generate(
	project_name TEXT, -- Human-friendly project name
	inputs TEXT[] 
)
RETURNS SETOF TEXT
AS $$
	from pgml_extension.model import Project
	return Project.find_by_name(project_name).deployed_model.generate(inputs)
$$ LANGUAGE plpython3u;
