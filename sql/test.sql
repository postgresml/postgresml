---
--- Test our functions.
---
\i sql/install.sql

\timing

SELECT pgml.version();

\i examples/regression/run.sql
\i examples/classification/run.sql
