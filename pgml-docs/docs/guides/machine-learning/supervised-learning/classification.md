---
description: >-
  Technique that assigns new observations to categorical labels or classes based
  on a model built from labeled training data.
---

# Classification

We currently support classification algorithms from [scikit-learn](https://scikit-learn.org/), [XGBoost](https://xgboost.readthedocs.io/), and [LightGBM](https://lightgbm.readthedocs.io/).

### Gradient Boosting

| Algorithm               | Classification                                                                                                             |
| ----------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| `xgboost`               | [XGBClassifier](https://xgboost.readthedocs.io/en/stable/python/python\_api.html#xgboost.XGBClassifier)                    |
| `xgboost_random_forest` | [XGBRFClassifier](https://xgboost.readthedocs.io/en/stable/python/python\_api.html#xgboost.XGBRFClassifier)                |
| `lightgbm`              | [LGBMClassifier](https://lightgbm.readthedocs.io/en/latest/pythonapi/lightgbm.LGBMClassifier.html#lightgbm.LGBMClassifier) |
| `catboost`              | [CatBoostClassifier](https://catboost.ai/en/docs/concepts/python-reference\_catboostclassifier)                            |

### Scikit Ensembles

| Algorithm                 | Classification                                                                                                                           |
| ------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| `ada_boost`               | [AdaBoostClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.AdaBoostClassifier.html)                         |
| `bagging`                 | [BaggingClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.BaggingClassifier.html)                           |
| `extra_trees`             | [ExtraTreesClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.ExtraTreesClassifier.html)                     |
| `gradient_boosting_trees` | [GradientBoostingClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.GradientBoostingClassifier.html)         |
| `random_forest`           | [RandomForestClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.RandomForestClassifier.html)                 |
| `hist_gradient_boosting`  | [HistGradientBoostingClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.HistGradientBoostingClassifier.html) |

### Support Vector Machines

| Algorithm    | Classification                                                                            |
| ------------ | ----------------------------------------------------------------------------------------- |
| `svm`        | [SVC](https://scikit-learn.org/stable/modules/generated/sklearn.svm.SVC.html)             |
| `nu_svm`     | [NuSVC](https://scikit-learn.org/stable/modules/generated/sklearn.svm.NuSVC.html)         |
| `linear_svm` | [LinearSVC](https://scikit-learn.org/stable/modules/generated/sklearn.svm.LinearSVC.html) |

### Linear Models

| Algorithm                     | Classification                                                                                                                          |
| ----------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| `linear`                      | [LogisticRegression](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.LogisticRegression.html)                   |
| `ridge`                       | [RidgeClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.RidgeClassifier.html)                         |
| `stochastic_gradient_descent` | [SGDClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.SGDClassifier.html)                             |
| `perceptron`                  | [Perceptron](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.Perceptron.html)                                   |
| `passive_aggressive`          | [PassiveAggressiveClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.PassiveAggressiveClassifier.html) |

### Other

| Algorithm          | Classification                                                                                                                          |
| ------------------ | --------------------------------------------------------------------------------------------------------------------------------------- |
| `gaussian_process` | [GaussianProcessClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.gaussian\_process.GaussianProcessClassifier.html) |
