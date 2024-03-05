---
--- Create the extension and load sample data into a database.
---
--- Usage:
---
---   $ cargo pgrx run --release
---   $ psql -P pager-off -h localhost -p 28816 -d pgml -f sql/setup_examples.sql
---
-- \set ON_ERROR_STOP true
\timing on

-- The intention is to only allow setup_examples.sql to run on a database that
-- has not had example data installed before, e.g. docker run. This should
-- error and stop the process if the extension is already present.
CREATE EXTENSION IF NOT EXISTS pgml;

SELECT pgml.load_dataset('breast_cancer');
SELECT pgml.load_dataset('diabetes');
SELECT pgml.load_dataset('digits');
SELECT pgml.load_dataset('iris');
SELECT pgml.load_dataset('linnerud');
SELECT pgml.load_dataset('wine');

\i examples/binary_classification.sql
\i examples/image_classification.sql
\i examples/joint_regression.sql
\i examples/multi_classification.sql
\i examples/regression.sql
