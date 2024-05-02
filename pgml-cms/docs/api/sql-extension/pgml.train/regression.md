---
description: >-
  Statistical method used to model the relationship between a dependent variable
  and one or more independent variables.
---

# Regression

We currently support regression algorithms from [scikit-learn](https://scikit-learn.org/ target="_blank"), [XGBoost](https://xgboost.readthedocs.io/ target="_blank"), [LightGBM](https://lightgbm.readthedocs.io/ target="_blank") and [Catboost](https://catboost.ai/ target="_blank").

## Example

This example trains models on the sklean [diabetes dataset](https://scikit-learn.org/stable/modules/generated/sklearn.datasets.load\_diabetes.html#sklearn.datasets.load\_diabetes target="_blank"). This example uses multiple input features to predict a single output variable.

```sql
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
| `xgboost`               | [XGBRegressor](https://xgboost.readthedocs.io/en/stable/python/python\_api.html#xgboost.XGBRegressor target="_blank")                   |
| `xgboost_random_forest` | [XGBRFRegressor](https://xgboost.readthedocs.io/en/stable/python/python\_api.html#xgboost.XGBRFRegressor target="_blank")               |
| `lightgbm`              | [LGBMRegressor](https://lightgbm.readthedocs.io/en/latest/pythonapi/lightgbm.LGBMRegressor.html#lightgbm.LGBMRegressor target="_blank") |
| `catboost`              | [CatBoostRegressor](https://catboost.ai/en/docs/concepts/python-reference\_catboostregressor target="_blank")                           |

#### Examples

```sql
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'xgboost', hyperparams => '{"n_estimators": 10}');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'xgboost_random_forest', hyperparams => '{"n_estimators": 10}');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'lightgbm', hyperparams => '{"n_estimators": 1}');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'catboost', hyperparams => '{"n_estimators": 10}');
```

### Ensembles

| Algorithm                 | Reference                                                                                                                              |
| ------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `ada_boost`               | [AdaBoostRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.AdaBoostRegressor.html target="_blank")                         |
| `bagging`                 | [BaggingRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.BaggingRegressor.html target="_blank")                           |
| `extra_trees`             | [ExtraTreesRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.ExtraTreesRegressor.html target="_blank")                     |
| `gradient_boosting_trees` | [GradientBoostingRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.GradientBoostingRegressor.html target="_blank")         |
| `random_forest`           | [RandomForestRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.RandomForestRegressor.html target="_blank")                 |
| `hist_gradient_boosting`  | [HistGradientBoostingRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.HistGradientBoostingRegressor.html target="_blank") |

#### Examples

```sql
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
| `svm`        | [SVR](https://scikit-learn.org/stable/modules/generated/sklearn.svm.SVR.html target="_blank")             |
| `nu_svm`     | [NuSVR](https://scikit-learn.org/stable/modules/generated/sklearn.svm.NuSVR.html target="_blank")         |
| `linear_svm` | [LinearSVR](https://scikit-learn.org/stable/modules/generated/sklearn.svm.LinearSVR.html target="_blank") |

#### Examples

```sql
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'svm', hyperparams => '{"max_iter": 100}');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'nu_svm', hyperparams => '{"max_iter": 10}');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'linear_svm', hyperparams => '{"max_iter": 100}');
```

### Linear

| Algorithm                           | Reference                                                                                                                             |
| ----------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| `linear`                            | [LinearRegression](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.LinearRegression.html target="_blank")                     |
| `ridge`                             | [Ridge](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.Ridge.html target="_blank")                                           |
| `lasso`                             | [Lasso](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.Lasso.html target="_blank")                                           |
| `elastic_net`                       | [ElasticNet](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.ElasticNet.html target="_blank")                                 |
| `least_angle`                       | [LARS](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.Lars.html target="_blank")                                             |
| `lasso_least_angle`                 | [LassoLars](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.LassoLars.html target="_blank")                                   |
| `orthoganl_matching_pursuit`        | [OrthogonalMatchingPursuit](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.OrthogonalMatchingPursuit.html target="_blank")   |
| `bayesian_ridge`                    | [BayesianRidge](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.BayesianRidge.html target="_blank")                           |
| `automatic_relevance_determination` | [ARDRegression](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.ARDRegression.html target="_blank")                           |
| `stochastic_gradient_descent`       | [SGDRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.SGDRegressor.html target="_blank")                             |
| `passive_aggressive`                | [PassiveAggressiveRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.PassiveAggressiveRegressor.html target="_blank") |
| `ransac`                            | [RANSACRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.RANSACRegressor.html target="_blank")                       |
| `theil_sen`                         | [TheilSenRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.TheilSenRegressor.html target="_blank")                   |
| `huber`                             | [HuberRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.HuberRegressor.html target="_blank")                         |
| `quantile`                          | [QuantileRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.QuantileRegressor.html target="_blank")                   |

#### Examples

```sql
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
| `kernel_ridge`     | [KernelRidge](https://scikit-learn.org/stable/modules/generated/sklearn.kernel\_ridge.KernelRidge.html target="_blank")                               |
| `gaussian_process` | [GaussianProcessRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.gaussian\_process.GaussianProcessRegressor.html target="_blank") |

#### Examples

```sql
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'kernel_ridge');
SELECT * FROM pgml.train('Diabetes Progression', algorithm => 'gaussian_process');
```
