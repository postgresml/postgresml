-- This example trains models on the sklean california_housing dataset
-- which is a copy of the test set from the StatLib repository
-- https://www.dcc.fc.up.pt/~ltorgo/Regression/cal_housing.html
--
-- This demonstrates using a table with individual columns as features
-- for regression.
SELECT pgml.load_dataset('california_housing');

-- view the dataset
SELECT * FROM pgml.california_housing LIMIT 10;

-- train a simple model to classify the data
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target');

-- check out the predictions
SELECT target, pgml.predict('California Housing Prediction', ARRAY[median_income, house_age, avg_rooms, avg_bedrooms, population, avg_occupants, latitude, longitude]) AS prediction
FROM pgml.california_housing 
LIMIT 10;

-- linear models
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'ridge');
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'lasso');
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'elastic_net');
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'least_angle');
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'lasso_least_angle');
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'orthoganl_matching_pursuit');
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'bayesian_ridge');
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'automatic_relevance_determination');
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'stochastic_gradient_descent');
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'passive_aggressive');
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'ransac');
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'theil_sen');
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'huber');
-- quantile regression is too slow for tests
-- SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'quantile');
--- support vector machines
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'svm');
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'nu_svm');
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'linear_svm');
-- ensembles
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'ada_boost');
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'bagging');
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'extra_trees');
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'gradient_boosting_trees');
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'hist_gradient_boosting');
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'random_forest');
-- other
SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'kernel_ridge');
-- guassian process goes OOM on this dataset
-- SELECT pgml.train('California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'gaussian_process');

-- check out all that hard work
SELECT * FROM pgml.trained_models 
JOIN pgml.models on models.id = trained_models.id
ORDER BY models.metrics->>'mean_squared_error' DESC LIMIT 5;

-- deploy the random_forest model for prediction use
SELECT pgml.deploy('California Housing Prediction', 'random_forest');
-- check out that throughput
SELECT * FROM pgml.deployed_models ORDER BY deployed_at DESC LIMIT 5;

-- do some hyper param tuning
-- TODO SELECT pgml.hypertune(100, 'California Housing Prediction', 'regression', 'pgml.california_housing', 'target', 'gradient_boosted_trees');
-- deploy the "best" model for prediction use
SELECT pgml.deploy('California Housing Prediction', 'best_fit');

-- check out the improved predictions
SELECT target, pgml.predict('California Housing Prediction', ARRAY[median_income, house_age, avg_rooms, avg_bedrooms, population, avg_occupants, latitude, longitude]) AS prediction
FROM pgml.california_housing 
LIMIT 10;
