ALTER TABLE pgml.snapshots ADD COLUMN materialized BOOLEAN DEFAULT false;

DROP FUNCTION IF EXISTS pgml.train(
	project_name text,
	task pgml.task,
	relation_name text,
	y_column_name text,
	algorithm pgml.algorithm,
	hyperparams jsonb,
	search pgml.search,
	search_params jsonb,
	search_args jsonb,
	test_size real,
	test_sampling pgml.sampling,
	runtime pgml.runtime,
	automatic_deploy
);

CREATE OR REPLACE FUNCTION pgml.train(
	project_name text,
	task pgml.task DEFAULT NULL::pgml.task,
	relation_name text DEFAULT NULL::text,
	y_column_name text DEFAULT NULL::text,
	algorithm pgml.algorithm DEFAULT 'linear'::pgml.algorithm,
	hyperparams jsonb DEFAULT '{}'::jsonb,
	search pgml.search DEFAULT NULL::pgml.search,
	search_params jsonb DEFAULT '{}'::jsonb,
	search_args jsonb DEFAULT '{}'::jsonb,
	test_size real DEFAULT 0.25,
	test_sampling pgml.sampling DEFAULT 'last'::pgml.sampling,
	runtime pgml.runtime DEFAULT NULL::pgml.runtime,
	automatic_deploy boolean DEFAULT true,
	materialize_snapshot boolean DEFAULT false
)

RETURNS TABLE(project text, task text, algorithm text, deployed boolean)
LANGUAGE c
AS '$libdir/pgml', $function$train_wrapper$function$
