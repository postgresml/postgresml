---
--- Test our functions.
---
--- Usage:
---
---   $ cargo pgrx run --release
---   $ psql -h localhost -p 28816 -d pgml -f tests/test.sql -P pager
---
\set ON_ERROR_STOP true
\timing on

-- Reset the test database to a clean state on each run.
DROP EXTENSION IF EXISTS pgml CASCADE;
DROP SCHEMA IF EXISTS pgml CASCADE;
CREATE EXTENSION pgml;

SELECT pgml.load_dataset('breast_cancer');
SELECT pgml.load_dataset('diabetes');
SELECT pgml.load_dataset('digits');
SELECT pgml.load_dataset('iris');
SELECT pgml.load_dataset('linnerud');
SELECT pgml.load_dataset('wine');

\i examples/clustering.sql
\i examples/decomposition.sql
\i examples/binary_classification.sql
\i examples/image_classification.sql
\i examples/joint_regression.sql
\i examples/multi_classification.sql
\i examples/regression.sql
\i examples/vectors.sql
\i examples/chunking.sql
\i examples/preprocessing.sql
-- transformers are generally too slow to run in the test suite
--\i examples/transformers.sql
