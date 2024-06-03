-- This example trains models on the sklean digits dataset
-- which is a copy of the test set of the UCI ML hand-written digits datasets
-- https://archive.ics.uci.edu/ml/datasets/Optical+Recognition+of+Handwritten+Digits
--
-- This demonstrates using a table with a single array feature column
-- for clustering. You could do something similar with a vector column
--

-- Exit on error (psql)
-- \set ON_ERROR_STOP true
\timing on

SELECT pgml.load_dataset('digits');

-- create an unlabeled table of the images for unsupervised learning
CREATE VIEW pgml.digit_vectors AS
SELECT image FROM pgml.digits;

-- view the dataset
SELECT left(image::text, 40) || ',...}' FROM pgml.digit_vectors LIMIT 10;

-- train a simple model to classify the data
SELECT * FROM pgml.train('Handwritten Digit Clusters', 'clustering', 'pgml.digit_vectors', hyperparams => '{"n_clusters": 10}');

-- check out the predictions
SELECT target, pgml.predict('Handwritten Digit Clusters', image) AS prediction
FROM pgml.digits
LIMIT 10;

SELECT * FROM pgml.train('Handwritten Digit Clusters', algorithm => 'affinity_propagation');
SELECT * FROM pgml.train('Handwritten Digit Clusters', algorithm => 'birch', hyperparams => '{"n_clusters": 10}');
SELECT * FROM pgml.train('Handwritten Digit Clusters', algorithm => 'kmeans', hyperparams => '{"n_clusters": 10}');
SELECT * FROM pgml.train('Handwritten Digit Clusters', algorithm => 'mini_batch_kmeans', hyperparams => '{"n_clusters": 10}');

-- Offline clustering algorithms are not currently supported
--SELECT * FROM pgml.train('Handwritten Digit Clusters', algorithm => 'dbscan');
--SELECT * FROM pgml.train('Handwritten Digit Clusters', algorithm => 'feature_agglomeration');
--SELECT * FROM pgml.train('Handwritten Digit Clusters', algorithm => 'optics');
--SELECT * FROM pgml.train('Handwritten Digit Clusters', algorithm => 'spectral', hyperparams => '{"n_clusters": 10}');
--SELECT * FROM pgml.train('Handwritten Digit Clusters', algorithm => 'spectral_bi', hyperparams => '{"n_clusters": 10}');
--SELECT * FROM pgml.train('Handwritten Digit Clusters', algorithm => 'spectral_co', hyperparams => '{"n_clusters": 10}');
--SELECT * FROM pgml.train('Handwritten Digit Clusters', algorithm => 'mean_shift');

