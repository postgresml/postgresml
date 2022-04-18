---
--- Test our functions.
---
\i sql/install.sql
\i data/winequality-red.sql

SELECT pgml.version();

\timing

SELECT * FROM pgml.train('Red Wine Scores', 'regression', 'wine_quality_red', 'quality');
SELECT pgml.predict('Red Wine Scores', '{7.4, 0.7, 0, 1.9, 0.076, 11, 34, 0.99, 2, 0.5, 9.4}');
SELECT pgml.predict('Red Wine Scores', '{6.4, 0.7, 0, 1.9, 0.076, 11, 34, 0.99, 2, 0.5, 9.4}');
SELECT pgml.predict('Red Wine Scores', '{5.4, 0.7, 0, 1.9, 0.076, 11, 34, 0.99, 2, 0.5, 9.4}');
SELECT pgml.predict('Red Wine Scores', '{3.4, 0.7, 0, 1.9, 0.076, 11, 34, 0.99, 2, 0.5, 9.4}');

SELECT * FROM pgml.train('Red Wine Categories', 'classification', 'wine_quality_red', 'quality', 'svm');
SELECT pgml.predict('Red Wine Categories', '{7.4, 0.7, 0, 1.9, 0.076, 11, 34, 0.99, 2, 0.5, 9.4}');

