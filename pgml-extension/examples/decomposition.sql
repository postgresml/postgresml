-- This example reduces the dimensionality of images in the sklean digits dataset
-- which is a copy of the test set of the UCI ML hand-written digits datasets
-- https://archive.ics.uci.edu/ml/datasets/Optical+Recognition+of+Handwritten+Digits
--
-- This demonstrates using a table with a single array feature column
-- for decomposition to reduce dimensionality.
--
-- Exit on error (psql)
-- \set ON_ERROR_STOP true
\timing on

SELECT pgml.load_dataset('digits');

-- view the dataset
SELECT left(image::text, 40) || ',...}', target FROM pgml.digits LIMIT 10;

-- create a view of just the vectors for decomposition, without any labels
CREATE VIEW digit_vectors AS
SELECT image FROM pgml.digits;

SELECT * FROM pgml.train('Handwritten Digits Reduction', 'decomposition', 'digit_vectors');

-- check out the decomposed vectors
SELECT target, pgml.decompose('Handwritten Digits Reduction', image) AS pca
FROM pgml.digits
LIMIT 10;

--
-- After a project has been trained, omitted parameters will be reused from previous training runs
-- In these examples we'll reuse the training data snapshots from the initial call.
--

-- We can reduce the image vectors from 64 dimensions to 3 components
SELECT * FROM pgml.train('Handwritten Digits Reduction', hyperparams => '{"n_components": 3}');

-- check out the reduced vectors
SELECT target, pgml.decompose('Handwritten Digits Reduction', image) AS pca
FROM pgml.digits
LIMIT 10;

-- check out all that hard work
SELECT trained_models.* FROM pgml.trained_models
                                 JOIN pgml.models on models.id = trained_models.id
ORDER BY models.metrics->>'cumulative_explained_variance' DESC LIMIT 5;

-- deploy the PCA model for prediction use
SELECT * FROM pgml.deploy('Handwritten Digits Reduction', 'most_recent', 'pca');
-- check out that throughput
SELECT * FROM pgml.deployed_models ORDER BY deployed_at DESC LIMIT 5;

-- deploy the "best" model for prediction use
SELECT * FROM pgml.deploy('Handwritten Digits Reduction', 'best_score');
SELECT * FROM pgml.deploy('Handwritten Digits Reduction', 'most_recent');
SELECT * FROM pgml.deploy('Handwritten Digits Reduction', 'rollback');
SELECT * FROM pgml.deploy('Handwritten Digits Reduction', 'best_score', 'pca');

-- check out the improved predictions
SELECT target, pgml.predict('Handwritten Digits Reduction', image) AS prediction
FROM pgml.digits
LIMIT 10;
