---
description: >-
  Technique that assigns new observations to categorical labels or classes based
  on a model built from labeled training data.
---

# Classification

## Example

This example trains models on the sklean digits dataset which is a copy of the test set of the [UCI ML hand-written digits datasets](https://archive.ics.uci.edu/ml/datasets/Optical+Recognition+of+Handwritten+Digits). This demonstrates using a table with a single array feature column for classification. You could do something similar with a vector column.

```postgresql
-- load the sklearn digits dataset
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
```

## Algorithms

We currently support classification algorithms from [scikit-learn](https://scikit-learn.org/), [XGBoost](https://xgboost.readthedocs.io/), [LightGBM](https://lightgbm.readthedocs.io/) and [Catboost](https://catboost.ai/).

### Gradient Boosting

| Algorithm               | Reference                                                                                                                  |
| ----------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| `xgboost`               | [XGBClassifier](https://xgboost.readthedocs.io/en/stable/python/python\_api.html#xgboost.XGBClassifier)                    |
| `xgboost_random_forest` | [XGBRFClassifier](https://xgboost.readthedocs.io/en/stable/python/python\_api.html#xgboost.XGBRFClassifier)                |
| `lightgbm`              | [LGBMClassifier](https://lightgbm.readthedocs.io/en/latest/pythonapi/lightgbm.LGBMClassifier.html#lightgbm.LGBMClassifier) |
| `catboost`              | [CatBoostClassifier](https://catboost.ai/en/docs/concepts/python-reference\_catboostclassifier)                            |

#### Examples

```postgresql
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'xgboost', hyperparams => '{"n_estimators": 10}');
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'xgboost_random_forest', hyperparams => '{"n_estimators": 10}');
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'lightgbm', hyperparams => '{"n_estimators": 1}');
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'catboost', hyperparams => '{"n_estimators": 1}');
```

### Scikit Ensembles

| Algorithm                 | Reference                                                                                                                                |
| ------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| `ada_boost`               | [AdaBoostClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.AdaBoostClassifier.html)                         |
| `bagging`                 | [BaggingClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.BaggingClassifier.html)                           |
| `extra_trees`             | [ExtraTreesClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.ExtraTreesClassifier.html)                     |
| `gradient_boosting_trees` | [GradientBoostingClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.GradientBoostingClassifier.html)         |
| `random_forest`           | [RandomForestClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.RandomForestClassifier.html)                 |
| `hist_gradient_boosting`  | [HistGradientBoostingClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.HistGradientBoostingClassifier.html) |

#### Examples

```postgresql
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'ada_boost');
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'bagging');
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'extra_trees', hyperparams => '{"n_estimators": 10}');
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'gradient_boosting_trees', hyperparams => '{"n_estimators": 10}');
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'random_forest', hyperparams => '{"n_estimators": 10}');
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'hist_gradient_boosting', hyperparams => '{"max_iter": 2}');
```

### Support Vector Machines

| Algorithm    | Reference                                                                                 |
| ------------ | ----------------------------------------------------------------------------------------- |
| `svm`        | [SVC](https://scikit-learn.org/stable/modules/generated/sklearn.svm.SVC.html)             |
| `nu_svm`     | [NuSVC](https://scikit-learn.org/stable/modules/generated/sklearn.svm.NuSVC.html)         |
| `linear_svm` | [LinearSVC](https://scikit-learn.org/stable/modules/generated/sklearn.svm.LinearSVC.html) |

#### Examples

```postgresql
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'svm');
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'nu_svm');
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'linear_svm');
```

### Linear Models

| Algorithm                     | Reference                                                                                                                               |
| ----------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| `linear`                      | [LogisticRegression](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.LogisticRegression.html)                   |
| `ridge`                       | [RidgeClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.RidgeClassifier.html)                         |
| `stochastic_gradient_descent` | [SGDClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.SGDClassifier.html)                             |
| `perceptron`                  | [Perceptron](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.Perceptron.html)                                   |
| `passive_aggressive`          | [PassiveAggressiveClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.linear\_model.PassiveAggressiveClassifier.html) |

#### Examples

```postgresql
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'ridge');
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'stochastic_gradient_descent');
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'perceptron');
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'passive_aggressive');
```

### Other

| Algorithm          | Reference                                                                                                                               |
| ------------------ | --------------------------------------------------------------------------------------------------------------------------------------- |
| `gaussian_process` | [GaussianProcessClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.gaussian\_process.GaussianProcessClassifier.html) |

#### Examples

```postgresql
SELECT * FROM pgml.train('Handwritten Digits', algorithm => 'gaussian_process', hyperparams => '{"max_iter_predict": 100, "warm_start": true}');
```
