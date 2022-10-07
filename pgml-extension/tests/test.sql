---
--- Test our functions.
---
\set ON_ERROR_STOP true

\timing

SELECT pgml.load_dataset('diabetes');
SELECT pgml.load_dataset('digits');
SELECT pgml.load_dataset('iris');
SELECT pgml.load_dataset('breast_cancer');

\i pgml_rust/tests/regression.sql
\i pgml_rust/tests/binary_classification.sql
\i pgml_rust/tests/image_classification.sql
