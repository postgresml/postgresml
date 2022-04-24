SET client_min_messages TO WARNING;

-- Create the PL/Python3 extension.
CREATE EXTENSION IF NOT EXISTS plpython3u;

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
CREATE OR REPLACE FUNCTION pgml.train(project_name TEXT, objective TEXT, relation_name TEXT, y_column_name TEXT, algorithm TEXT DEFAULT 'linear', hyperparams JSONB DEFAULT '{}'::JSONB)
RETURNS TABLE(project_name TEXT, objective TEXT, algorithm_name TEXT, status TEXT)
AS $$
	from pgml.model import train
	import json
	status = train(project_name, objective, relation_name, y_column_name, algorithm, json.loads(hyperparams))
	return [(project_name, objective, algorithm, status)]
$$ LANGUAGE plpython3u;

---
--- Deploy
---
CREATE OR REPLACE FUNCTION pgml.deploy(project_name TEXT, qualifier TEXT DEFAULT 'best_score', algorithm_name TEXT DEFAULT NULL)
RETURNS TABLE(project_name TEXT, objective TEXT, algorithm_name TEXT)
AS $$
	from pgml.model import Project
	model = Project.find_by_name(project_name).deploy(qualifier, algorithm_name)
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
