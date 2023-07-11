--
-- This demonstrates using a table with individual columns as features
-- for regression, along with multiple jointly optimized targets.

-- Exit on error (psql)
-- \set ON_ERROR_STOP true
\timing on

SELECT pgml.load_dataset('linnerud');

-- view the dataset
SELECT * FROM pgml.linnerud LIMIT 10;

-- train a simple model on the data
SELECT * FROM pgml.train_joint('Exercise vs Physiology', 'regression', 'pgml.linnerud', ARRAY['weight', 'waist', 'pulse']);

-- check out the predictions
SELECT weight, waist, pulse, pgml.predict_joint('Exercise vs Physiology', ARRAY[chins, situps, jumps]) AS prediction
FROM pgml.linnerud 
LIMIT 10;

-- linear models
SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'ridge');
SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'lasso');
SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'elastic_net');
SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'least_angle');
SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'lasso_least_angle');
SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'orthogonal_matching_pursuit');
SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'bayesian_ridge');
SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'automatic_relevance_determination');
SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'stochastic_gradient_descent');
SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'passive_aggressive');
SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'ransac');
SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'theil_sen', hyperparams => '{"max_iter": 10, "max_subpopulation": 100}');
SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'huber');
-- Quantile Regression too expensive for normal tests on even a toy dataset
-- SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'quantile');

-- support vector machines
SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'svm', hyperparams => '{"max_iter": 100}');
SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'nu_svm', hyperparams => '{"max_iter": 10}');
SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'linear_svm', hyperparams => '{"max_iter": 100}');

-- ensembles
SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'ada_boost', hyperparams => '{"n_estimators": 5}');
SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'bagging', hyperparams => '{"n_estimators": 5}');
SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'extra_trees', hyperparams => '{"n_estimators": 5}');
SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'gradient_boosting_trees', hyperparams => '{"n_estimators": 5}');
SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'random_forest', hyperparams => '{"n_estimators": 5}');

-- other
-- Kernel Ridge is too expensive for normal tests on even a toy dataset
-- SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'kernel_ridge');
-- Gaussian Process is too expensive for normal tests on even a toy dataset
-- SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'gaussian_process');

-- gradient boosting
--SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'xgboost', hyperparams => '{"n_estimators": 10}'); -- too slow
--SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'xgboost_random_forest', hyperparams => '{"n_estimators": 10}');
-- SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'lightgbm', hyperparams => '{"n_estimators": 1}');
-- Histogram Gradient Boosting is too expensive for normal tests on even a toy dataset
-- SELECT * FROM pgml.train_joint('Exercise vs Physiology', algorithm => 'hist_gradient_boosting', hyperparams => '{"max_iter": 10}');

-- check out all that hard work
SELECT trained_models.* FROM pgml.trained_models 
JOIN pgml.models on models.id = trained_models.id
ORDER BY models.metrics->>'mean_squared_error' DESC LIMIT 5;

-- deploy the random_forest model for prediction use
SELECT * FROM pgml.deploy('Exercise vs Physiology', 'most_recent', 'random_forest');
-- check out that throughput
SELECT * FROM pgml.deployed_models ORDER BY deployed_at DESC LIMIT 5;

-- deploy the "best" model for prediction use
SELECT * FROM pgml.deploy('Exercise vs Physiology', 'best_score');
SELECT * FROM pgml.deploy('Exercise vs Physiology', 'most_recent');
SELECT * FROM pgml.deploy('Exercise vs Physiology', 'rollback');
SELECT * FROM pgml.deploy('Exercise vs Physiology', 'best_score', 'svm');

-- check out the improved predictions
SELECT weight, waist, pulse, pgml.predict_joint('Exercise vs Physiology', ARRAY[chins, situps, jumps]) AS prediction
FROM pgml.linnerud 
LIMIT 10;
