--
-- Run this with psql.
--

DROP TABLE IF EXISTS wine_quality_red;
CREATE TABLE wine_quality_red (
	fixed_acidity DOUBLE PRECISION,
	volatile_acidity DOUBLE PRECISION,
	citric_acid DOUBLE PRECISION,
	residual_sugar DOUBLE PRECISION,
	chlorides DOUBLE PRECISION,
	free_sulfur_dioxide DOUBLE PRECISION,
	total_sulfur_dioxide DOUBLE PRECISION,
	density DOUBLE PRECISION,
	ph DOUBLE PRECISION,
	sulphates DOUBLE PRECISION,
	alcohol DOUBLE PRECISION,
	quality DOUBLE PRECISION
);

\copy wine_quality_red FROM 'data/winequality-red.csv' CSV HEADER 
