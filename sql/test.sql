---
--- Test our functions.
---
\i sql/install.sql
\i data/winequality-red.sql

SELECT pgml.version();

\timing

SELECT pgml.train('Red Wine', 'regression', 'wine_quality_red', 'quality');
SELECT pgml.predict('Red Wine', 7.4, 0.7, 0, 1.9, 0.076, 11, 34, 0.99, 2, 0.5, 9.4);
SELECT pgml.predict('Red Wine', 6.4, 0.7, 0, 1.9, 0.076, 11, 34, 0.99, 2, 0.5, 9.4);
SELECT pgml.predict('Red Wine', 5.4, 0.7, 0, 1.9, 0.076, 11, 34, 0.99, 2, 0.5, 9.4);
SELECT pgml.predict('Red Wine', 3.4, 0.7, 0, 1.9, 0.076, 11, 34, 0.99, 2, 0.5, 9.4);

SELECT pgml.train('Red Wine Categories', 'classification', 'wine_quality_red', 'quality');
SELECT pgml.predict('Red Wine', 7.4, 0.7, 0, 1.9, 0.076, 11, 34, 0.99, 2, 0.5, 9.4);

