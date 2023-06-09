
# Upgrade a v1.0 installation to v2.0

The API is identical between v1.0 and v2.0, and models trained with v1.0 can be imported into v2.0. 

!!! note

Make sure you've set up the system requirements in [v2.0 installation](/docs/guides/setup/v2/installation/), so that the v2.0 extension may be installed.

!!!

## Migration
You may run this migration to install the v2.0 extension and copy all existing assets from an existing v1.0 installation.

```postgresql
-- Run this migration as an atomic step
BEGIN;

-- Move the existing installation to a temporary schema
ALTER SCHEMA pgml RENAME to pgml_tmp;

-- Create the v2.0 extension
CREATE EXTENSION pgml;

-- Copy v1.0 projects into v2.0
INSERT INTO pgml.projects (id, name, task, created_at, updated_at)
SELECT id, name, task::pgml.task, created_at, updated_at 
FROM pgml_tmp.projects;
SELECT setval('pgml.projects_id_seq', COALESCE((SELECT MAX(id)+1 FROM pgml.projects), 1), false);

-- Copy v1.0 snapshots into v2.0
INSERT INTO pgml.snapshots (id, relation_name, y_column_name, test_size, test_sampling, status, columns, analysis, created_at, updated_at)
SELECT id, relation_name, y_column_name, test_size, test_sampling::pgml.sampling, status, columns, analysis, created_at, updated_at
FROM pgml_tmp.snapshots;
SELECT setval('pgml.snapshots_id_seq', COALESCE((SELECT MAX(id)+1 FROM pgml.snapshots), 1), false);

-- Copy v1.0 models into v2.0
INSERT INTO pgml.models (id, project_id, snapshot_id, num_features, algorithm, hyperparams, status, metrics, search, search_params, search_args, created_at, updated_at)
SELECT 
  models.id, 
  project_id, 
  snapshot_id, 
  (SELECT count(*) FROM jsonb_object_keys(snapshots.columns)) - array_length(snapshots.y_column_name, 1) num_features,  
  case when algorithm_name = 'orthoganl_matching_pursuit' then 'orthogonal_matching_pursuit'::pgml.algorithm else algorithm_name::pgml.algorithm end, 
  hyperparams, 
  models.status, 
  metrics, 
  search, 
  search_params, 
  search_args, 
  models.created_at, 
  models.updated_at
FROM pgml_tmp.models 
JOIN pgml_tmp.snapshots 
  ON snapshots.id = models.snapshot_id;
SELECT setval('pgml.models_id_seq', COALESCE((SELECT MAX(id)+1 FROM pgml.models), 1), false);

-- Copy v1.0 deployments into v2.0
INSERT INTO pgml.deployments
SELECT id, project_id, model_id, strategy::pgml.strategy, created_at
FROM pgml_tmp.deployments;
SELECT setval('pgml.deployments_id_seq', COALESCE((SELECT MAX(id)+1 FROM pgml.deployments), 1), false);

-- Copy v1.0 files into v2.0
INSERT INTO pgml.files (id, model_id, path, part, created_at, updated_at, data)
SELECT id, model_id, path, part, created_at, updated_at, data
FROM pgml_tmp.files;
SELECT setval('pgml.files_id_seq', COALESCE((SELECT MAX(id)+1 FROM pgml.files), 1), false);

-- Complete the migration
COMMIT;
```

## Cleanup v1.0
Make sure you validate the v2.0 installation first by running some predictions with existing models, before removing the v1.0 installation completely.

```postgresql
DROP SCHEMA pgml_tmp;
```


