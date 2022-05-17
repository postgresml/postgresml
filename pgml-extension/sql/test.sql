---
--- Test our functions.
---
\set ON_ERROR_STOP true

\i sql/install.sql

\timing

SELECT pgml.version();

SELECT pgml.load_dataset('diabetes');
SELECT pgml.load_dataset('digits');
SELECT pgml.load_dataset('iris');
SELECT pgml.load_dataset('linnerud');
SELECT pgml.load_dataset('wine');
SELECT pgml.load_dataset('breast_cancer');

SELECT pgml.load_dataset('california_housing');

\i examples/joint_regression.sql
\i examples/vectors.sql
\i examples/regression.sql
\i examples/binary_classification.sql
\i examples/multi_classification.sql
\i examples/image_classification.sql

