-- This example trains models on the sklean california_housing dataset
-- which is a copy of the test set from the StatLib repository
-- https://www.dcc.fc.up.pt/~ltorgo/Regression/cal_housing.html
--
-- This demonstrates using a table with individual columns as features
-- for regression.
SELECT pgml.load_dataset('california_housing');

-- view the dataset
SELECT * from pgml.california_housing;

-- train a simple model to classify the data
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target');

-- check out the predictions
SELECT target, pgml.predict('California Housing Prediction', ARRAY[median_income, house_age, avg_rooms, avg_bedrooms, population, avg_occupants, latitude, longitude]) AS prediction
FROM pgml.california_housing 
LIMIT 10;

-- -- train some more models with different algorithms
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'svm');
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'random_forest');
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'gradient_boosting_trees');
-- TODO SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'dense_neural_network');
-- -- check out all that hard work
SELECT * FROM pgml.trained_models;

-- deploy the random_forest model for prediction use
SELECT pgml.deploy('California Housing Prediction', 'random_forest');
-- check out that throughput
SELECT * FROM pgml.deployed_models;

-- do some hyper param tuning
-- TODO SELECT pgml.hypertune(100, 'California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'gradient_boosted_trees');
-- deploy the "best" model for prediction use
SELECT pgml.deploy('California Housing Prediction', 'best_fit');

-- check out the improved predictions
SELECT target, pgml.predict('California Housing Prediction', ARRAY[median_income, house_age, avg_rooms, avg_bedrooms, population, avg_occupants, latitude, longitude]) AS prediction
FROM pgml.california_housing 
LIMIT 10;
