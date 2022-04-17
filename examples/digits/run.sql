-- This example trains models on the sklean digits dataset
-- which is a copy of the test set of the UCI ML hand-written digits datasets
-- https://archive.ics.uci.edu/ml/datasets/Optical+Recognition+of+Handwritten+Digits
--
-- This demonstrates using a table with a single array feature column
-- for classification.
--
-- The final result after a few seconds of training is not terrible. Maybe not perfect
-- enough for mission critical applications, but it's telling how quickly "off the shelf" 
-- solutions can solve problems these days.
SELECT pgml.load_dataset('digits');

-- view the dataset
SELECT * from pgml.digits;

-- train a simple model to classify the data
SELECT pgml.train('Handwritten Digit Image Classifier', 'classification', 'pgml.digits', 'target');

-- check out the predictions
SELECT target, pgml.predict('Handwritten Digit Image Classifier', image) AS prediction
FROM pgml.digits 
LIMIT 10;

-- -- train some more models with different algorithms
SELECT pgml.train('Handwritten Digit Image Classifier', 'classification', 'pgml.digits', 'target', 'svm');
SELECT pgml.train('Handwritten Digit Image Classifier', 'classification', 'pgml.digits', 'target', 'random_forest');
SELECT pgml.train('Handwritten Digit Image Classifier', 'classification', 'pgml.digits', 'target', 'gradient_boosting_trees');
-- TODO SELECT pgml.train('Handwritten Digit Image Classifier', 'classification', 'pgml.digits', 'target', 'dense_neural_network');
-- -- check out all that hard work
SELECT * FROM pgml.trained_models;

-- deploy the random_forest model for prediction use
SELECT pgml.deploy('Handwritten Digit Image Classifier', 'random_forest');
-- check out that throughput
SELECT * FROM pgml.deployed_models;

-- do some hyper param tuning
-- TODO SELECT pgml.hypertune(100, 'Handwritten Digit Image Classifier', 'classification', 'pgml.digits', 'target', 'gradient_boosted_trees');
-- deploy the "best" model for prediction use
SELECT pgml.deploy('Handwritten Digit Image Classifier', 'best_fit');

-- check out the improved predictions
SELECT target, pgml.predict('Handwritten Digit Image Classifier', image) AS prediction
FROM pgml.digits 
LIMIT 10;
