---
--- Test our functions.
---
\set ON_ERROR_STOP true

\timing

SELECT pgml.load_dataset('breast_cancer');
SELECT pgml.load_dataset('diabetes');
SELECT pgml.load_dataset('digits');
SELECT pgml.load_dataset('iris');
SELECT pgml.load_dataset('linnerud');
SELECT pgml.load_dataset('wine');

\i tests/binary_classification.sql
\i tests/image_classification.sql
\i tests/joint_regression.sql
\i tests/multi_classification.sql
\i tests/regression.sql
\i tests/vectors.sql
