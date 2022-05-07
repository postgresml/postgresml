-- Create the PL/Python3 extension.
CREATE EXTENSION IF NOT EXISTS plpython3u;


---
--- Extension version
---
CREATE OR REPLACE FUNCTION pgml.version()
RETURNS TEXT
AS $$
	import pgml_extension
	return pgml_extension.version()
$$ LANGUAGE plpython3u;


---
--- Load toy datasets for practice
---
CREATE OR REPLACE FUNCTION pgml.load_dataset(
	source TEXT -- diabetes, digits, iris, linnerud, wine, breast_cancer, california_housing
)
RETURNS TEXT
AS $$
	from pgml_extension.datasets import load
	return load(source)
$$ LANGUAGE plpython3u;


---
--- Train a model
---
CREATE OR REPLACE FUNCTION pgml.train(
	project_name TEXT, 							-- Human-friendly project name
	objective TEXT DEFAULT NULL,                -- 'regression' or 'classification'
	relation_name TEXT DEFAULT NULL,            -- name of table or view
	y_column_name TEXT DEFAULT NULL,            -- aka "label" or "unknown" or "target"
	algorithm TEXT DEFAULT 'linear',            -- statistical learning method
	hyperparams JSONB DEFAULT '{}'::JSONB,      -- options for the model
	search TEXT DEFAULT NULL,                   -- hyperparam tuning, 'grid' or 'random'
	search_params JSONB DEFAULT '{}'::JSONB,    -- hyperparam search space
	search_args JSONB DEFAULT '{}'::JSONB,      -- hyperparam options
	test_size REAL DEFAULT 0.25,                -- fraction of the data for the test set
	test_sampling TEXT DEFAULT 'random'         -- 'random', 'first' or 'last'  
)
RETURNS TABLE(project_name TEXT, objective TEXT, algorithm_name TEXT, status TEXT)
AS $$
	from pgml_extension.model import train
	import json
	status = train(
		project_name, 
		objective, 
		relation_name, 
		[y_column_name], 
		algorithm, 
		json.loads(hyperparams),
		search,
		json.loads(search_params),
		json.loads(search_args),
		test_size,
		test_sampling
	)

	if "projects" in GD:
		if project_name in GD["projects"]:
	 		del GD["projects"][project_name]

	return [(project_name, objective, algorithm, status)]
$$ LANGUAGE plpython3u;


--
-- Train a model w/ multiple outputs
--
CREATE OR REPLACE FUNCTION pgml.train_joint(
	project_name TEXT, 							-- Human-friendly project name
	objective TEXT DEFAULT NULL,                -- 'regression' or 'classification'
	relation_name TEXT DEFAULT NULL,            -- name of table or view
	y_column_name TEXT[] DEFAULT NULL,          -- multiple "labels" or "unknowns" or "targets"
	algorithm TEXT DEFAULT 'linear',            -- statistical learning method
	hyperparams JSONB DEFAULT '{}'::JSONB,      -- options for the model
	search TEXT DEFAULT NULL,                   -- hyperparam tuning, 'grid' or 'random'
	search_params JSONB DEFAULT '{}'::JSONB,    -- hyperparam search space
	search_args JSONB DEFAULT '{}'::JSONB,      -- hyperparam options
	test_size REAL DEFAULT 0.25,                -- fraction of the data for the test set
	test_sampling TEXT DEFAULT 'random'         -- 'random', 'first' or 'last'  
)
RETURNS TABLE(project_name TEXT, objective TEXT, algorithm_name TEXT, status TEXT)
AS $$
	from pgml_extension.model import train
	import json
	status = train(
		project_name, 
		objective, 
		relation_name, 
		y_column_name, 
		algorithm, 
		json.loads(hyperparams),
		search,
		json.loads(search_params),
		json.loads(search_args),
		test_size,
		test_sampling
	)

	if "projects" in GD:
		if project_name in GD["projects"]:
	 		del GD["projects"][project_name]

	return [(project_name, objective, algorithm, status)]
$$ LANGUAGE plpython3u;


---
--- Deploy a specific model
---
CREATE OR REPLACE FUNCTION pgml.deploy(
	project_name TEXT,                  -- Human-friendly project name
	strategy TEXT DEFAULT 'best_score', -- 'rollback', 'best_score', or 'most_recent'
	algorithm_name TEXT DEFAULT NULL    -- filter candidates to a particular algorithm, NULL = all qualify
)
RETURNS TABLE(project_name TEXT, strategy TEXT, algorithm_name TEXT)
AS $$
	from pgml_extension.model import Project
	model = Project.find_by_name(project_name).deploy(strategy, algorithm_name)

	if "projects" in GD:
		if project_name in GD["projects"]:
	 		del GD["projects"][project_name]

	return [(model.project.name, model.project.objective, model.algorithm_name)]
$$ LANGUAGE plpython3u;


---
--- Predict
---
CREATE OR REPLACE FUNCTION pgml.predict(
	project_name TEXT,          -- Human-friendly project name
	features DOUBLE PRECISION[] -- Must match the training data column order
)
RETURNS DOUBLE PRECISION
AS $$
	from pgml_extension.model import Project

	if "projects" not in GD:
		GD["projects"] = {}

	project = GD["projects"].get(project_name)
	if project is None:
		project = Project.find_by_name(project_name)
		GD["projects"][project_name] = project

	return project.deployed_model.predict(features)
$$ LANGUAGE plpython3u;


---
--- Predict w/ multiple outputs
---
CREATE OR REPLACE FUNCTION pgml.predict_joint(
	project_name TEXT,          -- Human-friendly project name
	features DOUBLE PRECISION[] -- Must match the training data column order
)
RETURNS DOUBLE PRECISION[]
AS $$
	from pgml_extension.model import Project

	if "projects" not in GD:
		GD["projects"] = {}

	project = GD["projects"].get(project_name)
	if project is None:
		project = Project.find_by_name(project_name)
		GD["projects"][project_name] = project

	return project.deployed_model.predict(features)
$$ LANGUAGE plpython3u;
