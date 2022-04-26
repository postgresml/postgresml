-- This example trains models on the sklean diabetes dataset
-- Source URL: https://www4.stat.ncsu.edu/~boos/var.select/diabetes.html
-- For more information see:
--   Bradley Efron, Trevor Hastie, Iain Johnstone and Robert Tibshirani (2004)
--   "Least Angle Regression," Annals of Statistics (with discussion), 407-499
--   https://web.stanford.edu/~hastie/Papers/LARS/LeastAngle_2002.pdf

--
-- This demonstrates using a table with individual columns as features
-- for regression.

-- Exit on error (psql)
\set ON_ERROR_STOP true

SELECT pgml.load_dataset('diabetes');

-- view the dataset
SELECT * FROM pgml.diabetes LIMIT 10;

-- train a simple model on the data
SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target');

-- check out the predictions
SELECT target, pgml.predict('Diabetes Progression', ARRAY[age, sex, bmi, bp, s1, s2, s3, s4, s5, s6]) AS prediction
FROM pgml.diabetes 
LIMIT 10;

-- linear models
SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target', 'ridge');
SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target', 'lasso');
SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target', 'elastic_net');
SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target', 'least_angle');
SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target', 'lasso_least_angle');
SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target', 'orthoganl_matching_pursuit');
SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target', 'bayesian_ridge');
SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target', 'automatic_relevance_determination');
SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target', 'stochastic_gradient_descent');
SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target', 'passive_aggressive');
SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target', 'ransac');
SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target', 'theil_sen', '{"max_iter": 10, "max_subpopulation": 100}');
SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target', 'huber');
-- Quantile Regression too expensive for normal tests on even a toy dataset
-- SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target', 'quantile');
--- support vector machines
SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target', 'svm', '{"max_iter": 100}');
SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target', 'nu_svm', '{"max_iter": 10}');
SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target', 'linear_svm', '{"max_iter": 100}');
-- ensembles
SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target', 'ada_boost', '{"n_estimators": 5}');
SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target', 'bagging', '{"n_estimators": 5}');
SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target', 'extra_trees', '{"n_estimators": 5}');
SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target', 'gradient_boosting_trees', '{"n_estimators": 5}');
-- Histogram Gradient Boosting is too expensive for normal tests on even a toy dataset
-- SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target', 'hist_gradient_boosting', '{"max_iter": 10}');
SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target', 'random_forest', '{"n_estimators": 5}');
-- other
SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target', 'kernel_ridge');
SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target', 'xgboost');
-- Gaussian Process is too expensive for normal tests on even a toy dataset
-- SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target', 'gaussian_process');

-- check out all that hard work
SELECT trained_models.* FROM pgml.trained_models 
JOIN pgml.models on models.id = trained_models.id
ORDER BY models.metrics->>'mean_squared_error' DESC LIMIT 5;

-- deploy the random_forest model for prediction use
SELECT * FROM pgml.deploy('Diabetes Progression', 'most_recent', 'random_forest');
-- check out that throughput
SELECT * FROM pgml.deployed_models ORDER BY deployed_at DESC LIMIT 5;

-- do some hyper param tuning
-- TODO SELECT pgml.hypertune(100, 'Diabetes Progression', 'regression', 'pgml.diabetes', 'target', 'gradient_boosted_trees');

-- deploy the "best" model for prediction use
SELECT * FROM pgml.deploy('Diabetes Progression', 'best_score');
SELECT * FROM pgml.deploy('Diabetes Progression', 'most_recent');
SELECT * FROM pgml.deploy('Diabetes Progression', 'rollback');
SELECT * FROM pgml.deploy('Diabetes Progression', 'best_score', 'svm');

-- check out the improved predictions
SELECT target, pgml.predict('Diabetes Progression', ARRAY[age, sex, bmi, bp, s1, s2, s3, s4, s5, s6]) AS prediction
FROM pgml.diabetes 
LIMIT 10;
