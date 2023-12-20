-- Exit on error (psql)
-- \set ON_ERROR_STOP true
\timing on

SELECT pgml.load_dataset('iris');

-- view the dataset
SELECT * FROM pgml.iris LIMIT 10;

-- snapshots are automatically random ordered at creation, so this view is just for fun
DROP VIEW IF EXISTS pgml.iris_view;
CREATE VIEW pgml.iris_view AS SELECT * FROM pgml.iris ORDER BY random() LIMIT 100;

-- train a simple model to classify the data
SELECT * FROM pgml.train('Iris Flower Types', 'classification', 'pgml.iris_view', 'target');

-- check out the predictions
SELECT target, pgml.predict('Iris Flower Types', ARRAY[sepal_length, sepal_width, petal_length, petal_width]) AS prediction
FROM pgml.iris_view 
LIMIT 10;

-- view raw class probabilities
SELECT target, pgml.predict_proba('Iris Flower Types', ARRAY[sepal_length, sepal_width, petal_length, petal_width]) AS prediction
FROM pgml.iris_view
LIMIT 10;

--
-- After a project has been trained, omitted parameters will be reused from previous training runs
-- In these examples we'll reuse the training data snapshots from the initial call.
--

-- linear models
SELECT * FROM pgml.train('Iris Flower Types', algorithm => 'ridge');
--SELECT * FROM pgml.train('Iris Flower Types', algorithm => 'stochastic_gradient_descent');
--SELECT * FROM pgml.train('Iris Flower Types', algorithm => 'perceptron');
--SELECT * FROM pgml.train('Iris Flower Types', algorithm => 'passive_aggressive');

-- support vector machines
SELECT * FROM pgml.train('Iris Flower Types', algorithm => 'svm');
SELECT * FROM pgml.train('Iris Flower Types', algorithm => 'nu_svm');
SELECT * FROM pgml.train('Iris Flower Types', algorithm => 'linear_svm');

-- ensembles
SELECT * FROM pgml.train('Iris Flower Types', algorithm => 'ada_boost');
SELECT * FROM pgml.train('Iris Flower Types', algorithm => 'bagging');
SELECT * FROM pgml.train('Iris Flower Types', algorithm => 'extra_trees', hyperparams => '{"n_estimators": 10}');
SELECT * FROM pgml.train('Iris Flower Types', algorithm => 'gradient_boosting_trees', hyperparams => '{"n_estimators": 10}');
SELECT * FROM pgml.train('Iris Flower Types', algorithm => 'random_forest', hyperparams => '{"n_estimators": 10}');

-- other
-- Gaussian Process is too expensive for normal tests on even a toy dataset
-- SELECT * FROM pgml.train('Iris Flower Types', algorithm => 'gaussian_process', hyperparams => '{"max_iter_predict": 100, "warm_start": true}');

-- gradient boosting
SELECT * FROM pgml.train('Iris Flower Types', algorithm => 'xgboost', hyperparams => '{"n_estimators": 10}');
SELECT * FROM pgml.train('Iris Flower Types', algorithm => 'catboost', hyperparams => '{"n_estimators": 10}');
--SELECT * FROM pgml.train('Iris Flower Types', algorithm => 'xgboost_random_forest', hyperparams => '{"n_estimators": 10}');
-- SELECT * FROM pgml.train('Iris Flower Types', algorithm => 'lightgbm', hyperparams => '{"n_estimators": 1}');
-- Histogram Gradient Boosting is too expensive for normal tests on even a toy dataset
-- SELECT * FROM pgml.train('Iris Flower Types', algorithm => 'hist_gradient_boosting', hyperparams => '{"max_iter": 2}');


-- check out all that hard work
SELECT trained_models.* FROM pgml.trained_models 
JOIN pgml.models on models.id = trained_models.id
ORDER BY models.metrics->>'f1' DESC LIMIT 5;

-- deploy the random_forest model for prediction use
SELECT * FROM pgml.deploy('Iris Flower Types', 'most_recent', 'random_forest');
-- check out that throughput
SELECT * FROM pgml.deployed_models ORDER BY deployed_at DESC LIMIT 5;

-- do a hyperparam search on your favorite algorithm
SELECT pgml.train(
    'Iris Flower Types', 
    algorithm => 'gradient_boosting_trees', 
    hyperparams => '{"random_state": 0}',
    search => 'grid', 
    search_params => '{
        "n_estimators": [10, 20],
        "max_leaf_nodes": [2, 4],
        "criterion": ["friedman_mse", "squared_error"]
    }'
);

-- deploy the "best" model for prediction use
SELECT * FROM pgml.deploy('Iris Flower Types', 'best_score');
SELECT * FROM pgml.deploy('Iris Flower Types', 'most_recent');
SELECT * FROM pgml.deploy('Iris Flower Types', 'rollback');
SELECT * FROM pgml.deploy('Iris Flower Types', 'best_score', 'svm');

-- check out the improved predictions
SELECT target, pgml.predict('Iris Flower Types', ARRAY[sepal_length, sepal_width, petal_length, petal_width]) AS prediction
FROM pgml.iris_view 
LIMIT 10;

