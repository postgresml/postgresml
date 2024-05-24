---
description: >-
  Statistical method used to model the relationship between a dependent variable
  and one or more independent variables.
---

# Regression

We currently support regression algorithms from [scikit-learn](https://scikit-learn.org/), [XGBoost](https://xgboost.readthedocs.io/), [LightGBM](https://lightgbm.readthedocs.io/) and [Catboost](https://catboost.ai/).

## Example

This example trains models on the sklean [diabetes dataset](https://scikit-learn.org/stable/modules/generated/sklearn.datasets.load\_diabetes.html#sklearn.datasets.load\_diabetes). This example uses multiple input features to predict a single output variable.

```postgresql
-- load the dataset
SELECT pgml.load_dataset('diabetes');

-- view the dataset
SELECT * FROM pgml.diabetes LIMIT 10;

-- train a simple model on the data
SELECT * FROM pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target');

-- check out the predictions
SELECT target, pgml.predict('Diabetes Progression', ARRAY[age, sex, bmi, bp, s1, s2, s3, s4, s5, s6]) AS prediction
FROM pgml.diabetes 
LIMIT 10;
```

## Algorithms

### Gradient Boosting

| Algorithm               | Reference                                                                                                               |
| ----------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| `xgboost`               | [XGBRegressor](https://xgboost.readthedocs.io/en/stable/python/python\_api.html#xgboost.XGBRegressor)                   |
| `xgboost_random_forest` | [XGBRFRegressor](https://xgboost.readthedocs.io/en/stable/python/python\_api.html#xgboost.XGBRFRegressor)               |
| `lightgbm`              | [LGBMRegressor](https://lightgbm.readthedocs.io/en/latest/pythonapi/lightgbm.LGBMRegressor.html#lightgbm.LGBMRegressor) |
| `catboost`              | [CatBoostRegressor](https://catboost.ai/en/docs/concepts/python-reference\_catboostregressor)                           |

#### Examples

```postgresql
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'xgboost', hyperparams => '{"n_estimators": 10}');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'xgboost_random_forest', hyperparams => '{"n_estimators": 10}');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'lightgbm', hyperparams => '{"n_estimators": 1}');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'catboost', hyperparams => '{"n_estimators": 10}');
```

### Ensembles

| Algorithm                 | Reference                                                                                                                              |
| ------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `ada_boost`               | [AdaBoostRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.AdaBoostRegressor.html)                         |
| `bagging`                 | [BaggingRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.BaggingRegressor.html)                           |
| `extra_trees`             | [ExtraTreesRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.ExtraTreesRegressor.html)                     |
| `gradient_boosting_trees` | [GradientBoostingRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.GradientBoostingRegressor.html)         |
| `random_forest`           | [RandomForestRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.RandomForestRegressor.html)                 |
| `hist_gradient_boosting`  | [HistGradientBoostingRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.HistGradientBoostingRegressor.html) |

#### Examples

```postgresql
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'ada_boost', hyperparams => '{"n_estimators": 5}');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'bagging', hyperparams => '{"n_estimators": 5}');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'extra_trees', hyperparams => '{"n_estimators": 5}');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'gradient_boosting_trees', hyperparams => '{"n_estimators": 5}');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'random_forest', hyperparams => '{"n_estimators": 5}');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'hist_gradient_boosting', hyperparams => '{"max_iter": 10}');
```

### Support Vector Machines

| Algorithm    | Reference                                                                                 |
| ------------ | ----------------------------------------------------------------------------------------- |
| `svm`        | [SVR](https://scikit-learn.org/stable/modules/generated/sklearn.svm.SVR.html)             |
| `nu_svm`     | [NuSVR](https://scikit-learn.org/stable/modules/generated/sklearn.svm.NuSVR.html)         |
| `linear_svm` | [LinearSVR](https://scikit-learn.org/stable/modules/generated/sklearn.svm.LinearSVR.html) |

#### Examples

```postgresql
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'svm', hyperparams => '{"max_iter": 100}');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'nu_svm', hyperparams => '{"max_iter": 10}');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'linear_svm', hyperparams => '{"max_iter": 100}');
```

### Linear

| Algorithm                           | Reference                                                                                                                             |
| ----------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| `linear`                            | [LinearRegression](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.LinearRegression.html)                     |
| `ridge`                             | [Ridge](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.Ridge.html)                                           |
| `lasso`                             | [Lasso](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.Lasso.html)                                           |
| `elastic_net`                       | [ElasticNet](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.ElasticNet.html)                                 |
| `least_angle`                       | [LARS](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.Lars.html)                                             |
| `lasso_least_angle`                 | [LassoLars](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.LassoLars.html)                                   |
| `orthoganl_matching_pursuit`        | [OrthogonalMatchingPursuit](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.OrthogonalMatchingPursuit.html)   |
| `bayesian_ridge`                    | [BayesianRidge](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.BayesianRidge.html)                           |
| `automatic_relevance_determination` | [ARDRegression](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.ARDRegression.html)                           |
| `stochastic_gradient_descent`       | [SGDRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.SGDRegressor.html)                             |
| `passive_aggressive`                | [PassiveAggressiveRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.PassiveAggressiveRegressor.html) |
| `ransac`                            | [RANSACRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.RANSACRegressor.html)                       |
| `theil_sen`                         | [TheilSenRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.TheilSenRegressor.html)                   |
| `huber`                             | [HuberRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.HuberRegressor.html)                         |
| `quantile`                          | [QuantileRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.QuantileRegressor.html)                   |

#### Examples

```postgresql
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'linear');
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
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'quantile');
```

### Other

| Algorithm          | Reference                                                                                                                             |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------- |
| `kernel_ridge`     | [KernelRidge](https://scikit-learn.org/stable/modules/generated/sklearn.kernel\_ridge.KernelRidge.html)                               |
| `gaussian_process` | [GaussianProcessRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.gaussian\_process.GaussianProcessRegressor.html) |

#### Examples

```postgresql
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'kernel_ridge');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'gaussian_process');
```
