---
description: >-
  Statistical method used to model the relationship between a dependent variable
  and one or more independent variables.
---

# Regression

We currently support regression algorithms from [scikit-learn](https://scikit-learn.org/), [XGBoost](https://xgboost.readthedocs.io/), and [LightGBM](https://lightgbm.readthedocs.io/).

### Gradient Boosting

| Algorithm               | Regression                                                                                                              |
| ----------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| `xgboost`               | [XGBRegressor](https://xgboost.readthedocs.io/en/stable/python/python\_api.html#xgboost.XGBRegressor)                   |
| `xgboost_random_forest` | [XGBRFRegressor](https://xgboost.readthedocs.io/en/stable/python/python\_api.html#xgboost.XGBRFRegressor)               |
| `lightgbm`              | [LGBMRegressor](https://lightgbm.readthedocs.io/en/latest/pythonapi/lightgbm.LGBMRegressor.html#lightgbm.LGBMRegressor) |
| `catboost`              | [CatBoostRegressor](https://catboost.ai/en/docs/concepts/python-reference\_catboostregressor)                           |

### Scikit Ensembles

| Algorithm                 | Regression                                                                                                                             |
| ------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `ada_boost`               | [AdaBoostRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.AdaBoostRegressor.html)                         |
| `bagging`                 | [BaggingRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.BaggingRegressor.html)                           |
| `extra_trees`             | [ExtraTreesRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.ExtraTreesRegressor.html)                     |
| `gradient_boosting_trees` | [GradientBoostingRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.GradientBoostingRegressor.html)         |
| `random_forest`           | [RandomForestRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.RandomForestRegressor.html)                 |
| `hist_gradient_boosting`  | [HistGradientBoostingRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.HistGradientBoostingRegressor.html) |

### Support Vector Machines

| Algorithm    | Regression                                                                                |
| ------------ | ----------------------------------------------------------------------------------------- |
| `svm`        | [SVR](https://scikit-learn.org/stable/modules/generated/sklearn.svm.SVR.html)             |
| `nu_svm`     | [NuSVR](https://scikit-learn.org/stable/modules/generated/sklearn.svm.NuSVR.html)         |
| `linear_svm` | [LinearSVR](https://scikit-learn.org/stable/modules/generated/sklearn.svm.LinearSVR.html) |

### Linear Models

| Algorithm                           | Regression                                                                                                                            |
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

### Other

| Algorithm          | Regression                                                                                                                            |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------- |
| `kernel_ridge`     | [KernelRidge](https://scikit-learn.org/stable/modules/generated/sklearn.kernel\_ridge.KernelRidge.html)                               |
| `gaussian_process` | [GaussianProcessRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.gaussian\_process.GaussianProcessRegressor.html) |
