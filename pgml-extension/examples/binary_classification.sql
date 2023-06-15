-- Exit on error (psql)
-- \set ON_ERROR_STOP true
\timing on

SELECT pgml.load_dataset('breast_cancer');

-- view the dataset
SELECT * FROM pgml.breast_cancer LIMIT 10;

-- train a simple model to classify the data
SELECT * FROM pgml.train('Breast Cancer Detection', 'classification', 'pgml.breast_cancer', 'malignant');

-- check out the predictions
SELECT malignant, pgml.predict(
    'Breast Cancer Detection', 
    ARRAY[
        "mean radius", 
        "mean texture", 
        "mean perimeter", 
        "mean area",
        "mean smoothness",
        "mean compactness",
        "mean concavity",
        "mean concave points",
        "mean symmetry",
        "mean fractal dimension",
        "radius error",
        "texture error",
        "perimeter error",
        "area error",
        "smoothness error",
        "compactness error",
        "concavity error",
        "concave points error",
        "symmetry error",
        "fractal dimension error",
        "worst radius",
        "worst texture",
        "worst perimeter",
        "worst area",
        "worst smoothness",
        "worst compactness",
        "worst concavity",
        "worst concave points",
        "worst symmetry",
        "worst fractal dimension"
    ]
) AS prediction
FROM pgml.breast_cancer
ORDER BY random()
LIMIT 10;

-- view raw class probabilities
SELECT malignant, pgml.predict_proba(
    'Breast Cancer Detection',
    ARRAY[
        "mean radius",
        "mean texture",
        "mean perimeter",
        "mean area",
        "mean smoothness",
        "mean compactness",
        "mean concavity",
        "mean concave points",
        "mean symmetry",
        "mean fractal dimension",
        "radius error",
        "texture error",
        "perimeter error",
        "area error",
        "smoothness error",
        "compactness error",
        "concavity error",
        "concave points error",
        "symmetry error",
        "fractal dimension error",
        "worst radius",
        "worst texture",
        "worst perimeter",
        "worst area",
        "worst smoothness",
        "worst compactness",
        "worst concavity",
        "worst concave points",
        "worst symmetry",
        "worst fractal dimension"
    ]
) AS prediction
FROM pgml.breast_cancer
ORDER BY random()
LIMIT 10;

--
-- After a project has been trained, omitted parameters will be reused from previous training runs
-- In these examples we'll reuse the training data snapshots from the initial call.
--

-- linear models
SELECT * FROM pgml.train('Breast Cancer Detection', algorithm => 'ridge');
SELECT * FROM pgml.train('Breast Cancer Detection', algorithm => 'stochastic_gradient_descent');
SELECT * FROM pgml.train('Breast Cancer Detection', algorithm => 'perceptron');
SELECT * FROM pgml.train('Breast Cancer Detection', algorithm => 'passive_aggressive');

-- support vector machines
SELECT * FROM pgml.train('Breast Cancer Detection', algorithm => 'svm');
SELECT * FROM pgml.train('Breast Cancer Detection', algorithm => 'nu_svm');
SELECT * FROM pgml.train('Breast Cancer Detection', algorithm => 'linear_svm');

-- ensembles
SELECT * FROM pgml.train('Breast Cancer Detection', algorithm => 'ada_boost');
SELECT * FROM pgml.train('Breast Cancer Detection', algorithm => 'bagging');
SELECT * FROM pgml.train('Breast Cancer Detection', algorithm => 'extra_trees', hyperparams => '{"n_estimators": 10}');
SELECT * FROM pgml.train('Breast Cancer Detection', algorithm => 'gradient_boosting_trees', hyperparams => '{"n_estimators": 10}');
SELECT * FROM pgml.train('Breast Cancer Detection', algorithm => 'random_forest', hyperparams => '{"n_estimators": 10}');

-- other
-- Gaussian Process is too expensive for normal tests on even a toy dataset
-- SELECT * FROM pgml.train('Breast Cancer Detection', algorithm => 'gaussian_process', hyperparams => '{"max_iter_predict": 100, "warm_start": true}');

-- Gradient Boosting
SELECT * FROM pgml.train('Breast Cancer Detection', algorithm => 'xgboost', hyperparams => '{"n_estimators": 10}');
-- SELECT * FROM pgml.train('Breast Cancer Detection', algorithm => 'xgboost_random_forest', hyperparams => '{"n_estimators": 10}');
-- SELECT * FROM pgml.train('Breast Cancer Detection', algorithm => 'lightgbm', hyperparams => '{"n_estimators": 1}');
-- Histogram Gradient Boosting is too expensive for normal tests on even a toy dataset
-- SELECT * FROM pgml.train('Breast Cancer Detection', algorithm => 'hist_gradient_boosting', hyperparams => '{"max_iter": 2}');


-- check out all that hard work
SELECT trained_models.* FROM pgml.trained_models 
JOIN pgml.models on models.id = trained_models.id
ORDER BY models.metrics->>'f1' DESC LIMIT 5;

-- deploy the random_forest model for prediction use
SELECT * FROM pgml.deploy('Breast Cancer Detection', 'most_recent', 'random_forest');
-- check out that throughput
SELECT * FROM pgml.deployed_models ORDER BY deployed_at DESC LIMIT 5;

-- do a hyperparam search on your favorite algorithm
SELECT pgml.train(
    'Breast Cancer Detection', 
    algorithm => 'xgboost', 
    search => 'grid', 
    search_params => '{
        "n_estimators": [2, 4],
        "max_depth": [1, 2, 3]
    }'
);

-- deploy the "best" model for prediction use
SELECT * FROM pgml.deploy('Breast Cancer Detection', 'best_score');
SELECT * FROM pgml.deploy('Breast Cancer Detection', 'most_recent');
SELECT * FROM pgml.deploy('Breast Cancer Detection', 'rollback');
SELECT * FROM pgml.deploy('Breast Cancer Detection', 'best_score', 'svm');

-- check out the improved predictions
SELECT malignant, pgml.predict(
    'Breast Cancer Detection', 
    ARRAY[
        "mean radius", 
        "mean texture", 
        "mean perimeter", 
        "mean area",
        "mean smoothness",
        "mean compactness",
        "mean concavity",
        "mean concave points",
        "mean symmetry",
        "mean fractal dimension",
        "radius error",
        "texture error",
        "perimeter error",
        "area error",
        "smoothness error",
        "compactness error",
        "concavity error",
        "concave points error",
        "symmetry error",
        "fractal dimension error",
        "worst radius",
        "worst texture",
        "worst perimeter",
        "worst area",
        "worst smoothness",
        "worst compactness",
        "worst concavity",
        "worst concave points",
        "worst symmetry",
        "worst fractal dimension"
    ]
) AS prediction
FROM pgml.breast_cancer 
LIMIT 10;
