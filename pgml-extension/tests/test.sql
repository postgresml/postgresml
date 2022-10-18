---
--- Test our functions.
---
\set ON_ERROR_STOP true

\timing

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
\i examples/vectors.sql
