---
--- Test our functions.
---
\i sql/install.sql
\i data/winequality-red.sql

SELECT pgml.version();

-- Valiate our wine data.
SELECT pgml.validate('wine_quality_red');

-- Train twice
SELECT pgml.train('wine_quality_red', 'quality');

SELECT * FROM pgml.model_versions;

\timing
WITH latest_model AS (
	SELECT name || '_' || id AS model_name FROM pgml.model_versions ORDER BY id DESC LIMIT 1
)
SELECT pgml.score(
	(SELECT model_name FROM latest_model), -- last model we just trained

	-- features as variadic arguments
	7.4, 0.7, 0, 1.9, 0.076, 11, 34, 0.99, 2, 0.5, 9.4) AS score;
