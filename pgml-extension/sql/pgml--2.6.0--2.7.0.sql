ALTER TABLE pgml.snapshots ALTER COLUMN y_column_name DROP NOT NULL;

ALTER FUNCTION pgml.embed(text, text[], jsonb) COST 1000000;
ALTER FUNCTION pgml.embed(text, text, jsonb) COST 1000000;
ALTER FUNCTION pgml.transform(jsonb, jsonb, text[], boolean) COST 10000000;
ALTER FUNCTION pgml.transform(text, jsonb, text[], boolean) COST 10000000;

ALTER TYPE pgml.task ADD VALUE IF NOT EXISTS 'cluster';
ALTER TYPE pgml.algorithm ADD VALUE IF NOT EXISTS 'affinity_propagation';
ALTER TYPE pgml.algorithm ADD VALUE IF NOT EXISTS 'agglomerative';
ALTER TYPE pgml.algorithm ADD VALUE IF NOT EXISTS 'birch';
ALTER TYPE pgml.algorithm ADD VALUE IF NOT EXISTS 'feature_agglomeration';
ALTER TYPE pgml.algorithm ADD VALUE IF NOT EXISTS 'mini_batch_kmeans';
ALTER TYPE pgml.algorithm ADD VALUE IF NOT EXISTS 'mean_shift';
ALTER TYPE pgml.algorithm ADD VALUE IF NOT EXISTS 'optics';
ALTER TYPE pgml.algorithm ADD VALUE IF NOT EXISTS 'spectral';
ALTER TYPE pgml.algorithm ADD VALUE IF NOT EXISTS 'spectral_bi';
ALTER TYPE pgml.algorithm ADD VALUE IF NOT EXISTS 'spectral_co';

