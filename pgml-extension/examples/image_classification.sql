-- This example trains models on the sklean digits dataset
-- which is a copy of the test set of the UCI ML hand-written digits datasets
-- https://archive.ics.uci.edu/ml/datasets/Optical+Recognition+of+Handwritten+Digits
--
-- This demonstrates using a table with a single array feature column
-- for classification.
--
-- Some algorithms converge on this trivial dataset in under a second, demonstrating the
-- speed with which modern machines can "learn" from example data.

-- Exit on error (psql)
-- \set ON_ERROR_STOP true
\timing on

SELECT pgml.load_dataset('digits');

-- view the dataset
SELECT left(image::text, 40) || ',...}', target FROM pgml.digits LIMIT 10;

-- train a simple model to classify the data
SELECT * FROM pgml.train('Handwritten Digits', 'classification', 'pgml.digits', 'target');

-- check out the predictions
SELECT target, pgml.predict('Handwritten Digits', image) AS prediction
FROM pgml.digits 
LIMIT 10;

-- view raw class probabilities
SELECT target, pgml.predict_proba('Handwritten Digits', image) AS prediction
FROM pgml.digits
LIMIT 10;

--
-- After a project has been trained, omitted parameters will be reused from previous training runs
-- In these examples we'll reuse the training data snapshots from the initial call.
--

-- linear models
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'ridge');
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'stochastic_gradient_descent');
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'perceptron');
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'passive_aggressive');

-- support vector machines
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'svm');
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'nu_svm');
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'linear_svm');

-- ensembles
-- SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'ada_boost'); -- adaboost no longer converges? f1 is missing...
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'bagging');
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'extra_trees', hyperparams => '{"n_estimators": 10}');
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'gradient_boosting_trees', hyperparams => '{"n_estimators": 10}');
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'random_forest', hyperparams => '{"n_estimators": 10}');

-- other
-- Gaussian Process is too expensive for normal tests on even a toy dataset
-- SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'gaussian_process', hyperparams => '{"max_iter_predict": 100, "warm_start": true}');

-- gradient boosting
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'xgboost', hyperparams => '{"n_estimators": 10}');
-- SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'xgboost_random_forest', hyperparams => '{"n_estimators": 10}');
-- SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'lightgbm', hyperparams => '{"n_estimators": 1}');
-- Histogram Gradient Boosting is too expensive for normal tests on even a toy dataset
-- SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'hist_gradient_boosting', hyperparams => '{"max_iter": 2}');

-- runtimes
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'linear', runtime => 'python');
--SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'linear', runtime => 'rust');

--SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'xgboost', runtime => 'python', hyperparams => '{"n_estimators": 10}'); -- too slow
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'xgboost', runtime => 'rust', hyperparams => '{"n_estimators": 10}');

-- check out all that hard work
SELECT trained_models.* FROM pgml.trained_models 
JOIN pgml.models on models.id = trained_models.id
ORDER BY models.metrics->>'f1' DESC LIMIT 5;

-- deploy the random_forest model for prediction use
SELECT * FROM pgml.deploy('Handwritten Digits', 'most_recent', 'random_forest');
-- check out that throughput
SELECT * FROM pgml.deployed_models ORDER BY deployed_at DESC LIMIT 5;

-- do a hyperparam search on your favorite algorithm
SELECT pgml.train(
    'Handwritten Digits', 
    algorithm => 'svm', 
    hyperparams => '{"random_state": 0}',
    search => 'grid', 
    search_params => '{
        "kernel": ["linear", "poly", "sigmoid"], 
        "shrinking": [true, false]
    }'
);

-- deploy the "best" model for prediction use
SELECT * FROM pgml.deploy('Handwritten Digits', 'best_score');
SELECT * FROM pgml.deploy('Handwritten Digits', 'most_recent');
SELECT * FROM pgml.deploy('Handwritten Digits', 'rollback');
SELECT * FROM pgml.deploy('Handwritten Digits', 'best_score', 'svm');

-- check out the improved predictions
SELECT target, pgml.predict('Handwritten Digits', image) AS prediction
FROM pgml.digits 
LIMIT 10;
