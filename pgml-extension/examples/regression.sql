-- This example trains models on the sklearn diabetes dataset
-- Source URL: https://www4.stat.ncsu.edu/~boos/var.select/diabetes.html
-- For more information see:
--   Bradley Efron, Trevor Hastie, Iain Johnstone and Robert Tibshirani (2004)
--   "Least Angle Regression," Annals of Statistics (with discussion), 407-499
--   https://web.stanford.edu/~hastie/Papers/LARS/LeastAngle_2002.pdf

--
-- This demonstrates using a table with individual columns as features
-- for regression.

-- Exit on error (psql)
-- \set ON_ERROR_STOP true
\timing on

SELECT pgml.load_dataset('diabetes');

-- view the dataset
SELECT * FROM pgml.diabetes LIMIT 10;

-- train a simple model on the data
SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target');

-- check out the predictions
SELECT target, pgml.predict('Diabetes Progression', ARRAY[age, sex, bmi, bp, s1, s2, s3, s4, s5, s6]) AS prediction
FROM pgml.diabetes 
LIMIT 10;

-- Check predictions against a specific model id
SELECT model_id, target, pgml.predict(model_id, ARRAY[age, sex, bmi, bp, s1, s2, s3, s4, s5, s6]) AS prediction
FROM pgml.diabetes
CROSS JOIN LATERAL (
    SELECT pgml.models.id AS model_id FROM pgml.models
    INNER JOIN pgml.projects
    ON pgml.models.project_id = pgml.projects.id
    WHERE pgml.projects.name = 'Diabetes Progression'
    LIMIT 1
) models
LIMIT 10;

--
-- After a project has been trained, omitted parameters will be reused from previous training runs
-- In these examples we'll reuse the training data snapshots from the initial call.
--

-- linear models
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'ridge');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'lasso');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'elastic_net');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'least_angle');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'lasso_least_angle');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'orthogonal_matching_pursuit');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'bayesian_ridge');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'automatic_relevance_determination');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'stochastic_gradient_descent');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'passive_aggressive');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'ransac');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'theil_sen', hyperparams => '{"max_iter": 10, "max_subpopulation": 100}');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'huber');
-- Quantile Regression too expensive for normal tests on even a toy dataset
-- SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'quantile');

-- support vector machines
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'svm', hyperparams => '{"max_iter": 100}');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'nu_svm', hyperparams => '{"max_iter": 10}');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'linear_svm', hyperparams => '{"max_iter": 100}');

-- ensembles
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'ada_boost', hyperparams => '{"n_estimators": 5}');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'bagging', hyperparams => '{"n_estimators": 5}');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'extra_trees', hyperparams => '{"n_estimators": 5}');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'gradient_boosting_trees', hyperparams => '{"n_estimators": 5}');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'random_forest', hyperparams => '{"n_estimators": 5}');

-- other
-- Kernel Ridge is too expensive for normal tests on even a toy dataset
-- SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'kernel_ridge');
-- Gaussian Process is too expensive for normal tests on even a toy dataset
-- SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'gaussian_process');

-- gradient boosting
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'xgboost', hyperparams => '{"n_estimators": 10}');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'catboost', hyperparams => '{"n_estimators": 10}');
-- SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'xgboost_random_forest', hyperparams => '{"n_estimators": 10}');
-- SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'lightgbm', hyperparams => '{"n_estimators": 1}');
-- Histogram Gradient Boosting is too expensive for normal tests on even a toy dataset
-- SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'hist_gradient_boosting', hyperparams => '{"max_iter": 10}');

-- runtimes
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'linear', runtime => 'python');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'linear', runtime => 'rust');

--SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'xgboost', runtime => 'python', hyperparams => '{"n_estimators": 1}'); -- too slow
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'xgboost', runtime => 'rust', hyperparams => '{"n_estimators": 10}');

-- check out all that hard work
SELECT trained_models.* FROM pgml.trained_models 
JOIN pgml.models on models.id = trained_models.id
ORDER BY models.metrics->>'mean_squared_error' DESC LIMIT 5;

-- deploy the random_forest model for prediction use
SELECT * FROM pgml.deploy('Diabetes Progression', 'most_recent', 'random_forest');
-- check out that throughput
SELECT * FROM pgml.deployed_models ORDER BY deployed_at DESC LIMIT 5;

-- do a hyperparam search on your favorite algorithm
SELECT pgml.train(
    'Diabetes Progression', 
    algorithm => 'xgboost',
    hyperparams => '{"eval_metric": "rmse"}'::JSONB,
    search => 'grid', 
    search_params => '{
        "max_depth": [1, 2], 
        "n_estimators": [20, 40]
    }'
);

-- deploy the "best" model for prediction use
SELECT * FROM pgml.deploy('Diabetes Progression', 'best_score');
SELECT * FROM pgml.deploy('Diabetes Progression', 'most_recent');
SELECT * FROM pgml.deploy('Diabetes Progression', 'rollback');
SELECT * FROM pgml.deploy('Diabetes Progression', 'best_score', 'svm');

-- check out the improved predictions
SELECT target, pgml.predict('Diabetes Progression', ARRAY[age, sex, bmi, bp, s1, s2, s3, s4, s5, s6]) AS prediction
FROM pgml.diabetes 
LIMIT 10;
