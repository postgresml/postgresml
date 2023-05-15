# Algorithm Selection

We currently support regression and classification algorithms from [scikit-learn](https://scikit-learn.org/), [XGBoost](https://xgboost.readthedocs.io/), and [LightGBM](https://lightgbm.readthedocs.io/).

## Algorithms

### Gradient Boosting
Algorithm | Regression | Classification
--- | --- | ---
`xgboost` | [XGBRegressor](https://xgboost.readthedocs.io/en/stable/python/python_api.html#xgboost.XGBRegressor) | [XGBClassifier](https://xgboost.readthedocs.io/en/stable/python/python_api.html#xgboost.XGBClassifier)
`xgboost_random_forest` | [XGBRFRegressor](https://xgboost.readthedocs.io/en/stable/python/python_api.html#xgboost.XGBRFRegressor) | [XGBRFClassifier](https://xgboost.readthedocs.io/en/stable/python/python_api.html#xgboost.XGBRFClassifier)
`lightgbm` | [LGBMRegressor](https://lightgbm.readthedocs.io/en/latest/pythonapi/lightgbm.LGBMRegressor.html#lightgbm.LGBMRegressor) | [LGBMClassifier](https://lightgbm.readthedocs.io/en/latest/pythonapi/lightgbm.LGBMClassifier.html#lightgbm.LGBMClassifier)

### Scikit Ensembles
Algorithm | Regression | Classification
--- | --- | ---
`ada_boost` | [AdaBoostRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.AdaBoostRegressor.html) | [AdaBoostClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.AdaBoostClassifier.html)
`bagging` | [BaggingRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.BaggingRegressor.html) | [BaggingClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.BaggingClassifier.html)
`extra_trees` | [ExtraTreesRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.ExtraTreesRegressor.html) | [ExtraTreesClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.ExtraTreesClassifier.html)
`gradient_boosting_trees` | [GradientBoostingRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.GradientBoostingRegressor.html) | [GradientBoostingClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.GradientBoostingClassifier.html)
`random_forest` | [RandomForestRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.RandomForestRegressor.html) | [RandomForestClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.RandomForestClassifier.html)
`hist_gradient_boosting` | [HistGradientBoostingRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.HistGradientBoostingRegressor.html) | [HistGradientBoostingClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.HistGradientBoostingClassifier.html)

### Support Vector Machines
Algorithm | Regression | Classification
--- | --- | ---
`svm` | [SVR](https://scikit-learn.org/stable/modules/generated/sklearn.svm.SVR.html) | [SVC](https://scikit-learn.org/stable/modules/generated/sklearn.svm.SVC.html)
`nu_svm` | [NuSVR](https://scikit-learn.org/stable/modules/generated/sklearn.svm.NuSVR.html) | [NuSVC](https://scikit-learn.org/stable/modules/generated/sklearn.svm.NuSVC.html)
`linear_svm` | [LinearSVR](https://scikit-learn.org/stable/modules/generated/sklearn.svm.LinearSVR.html) | [LinearSVC](https://scikit-learn.org/stable/modules/generated/sklearn.svm.LinearSVC.html)

### Linear Models
Algorithm | Regression | Classification
--- | --- | ---
`linear` | [LinearRegression](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.LinearRegression.html) | [LogisticRegression](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.LogisticRegression.html)
`ridge` |  [Ridge](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.Ridge.html) | [RidgeClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.RidgeClassifier.html)
`lasso` |  [Lasso](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.Lasso.html) | -
`elastic_net` | [ElasticNet](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.ElasticNet.html) | -
`least_angle` | [LARS](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.Lars.html) | -
`lasso_least_angle` | [LassoLars](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.LassoLars.html) | -
`orthoganl_matching_pursuit` | [OrthogonalMatchingPursuit](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.OrthogonalMatchingPursuit.html) | -
`bayesian_ridge` | [BayesianRidge](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.BayesianRidge.html) | -
`automatic_relevance_determination` | [ARDRegression](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.ARDRegression.html) | -
`stochastic_gradient_descent` | [SGDRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.SGDRegressor.html) | [SGDClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.SGDClassifier.html) 
`perceptron` | - | [Perceptron](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.Perceptron.html) 
`passive_aggressive` | [PassiveAggressiveRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.PassiveAggressiveRegressor.html) | [PassiveAggressiveClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.PassiveAggressiveClassifier.html) 
`ransac` | [RANSACRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.RANSACRegressor.html) | -
`theil_sen` | [TheilSenRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.TheilSenRegressor.html) | -
`huber` | [HuberRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.HuberRegressor.html) | -
`quantile` | [QuantileRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.QuantileRegressor.html) | -

### Other
Algorithm | Regression | Classification
--- | --- | ---
`kernel_ridge` | [KernelRidge](https://scikit-learn.org/stable/modules/generated/sklearn.kernel_ridge.KernelRidge.html) | -
`gaussian_process` | [GaussianProcessRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.gaussian_process.GaussianProcessRegressor.html) | [GaussianProcessClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.gaussian_process.GaussianProcessClassifier.html)

## Comparing Algorithms

Any of the above algorithms can be passed to our `pgml.train()` function using the `algorithm` parameter. If the parameter is omitted, linear regression is used by default.

!!! example

```postgresql
SELECT * FROM pgml.train(
    'My First PostgresML Project',
    task => 'classification',
    relation_name => 'pgml.digits',
    y_column_name => 'target',
    algorithm => 'xgboost',
);
```

!!!


The `hyperparams` argument will pass the hyperparameters on to the algorithm. Take a look at the associated documentation for valid hyperparameters of each algorithm. Our interface uses the scikit-learn notation for all parameters.

!!! example

```postgresql
SELECT * FROM pgml.train(
    'My First PostgresML Project',
    algorithm => 'xgboost',
    hyperparams => '{
        "n_estimators": 25
    }'
);
```

!!!

Once prepared, the training data can be efficiently reused by other PostgresML algorithms for training and predictions. Every time the `pgml.train()` function receives the `relation_name` and `y_column_name` arguments, it will create a new snapshot of the relation (table) and save it in the `pgml` schema.

To train another algorithm on the same dataset, omit the two arguments. PostgresML will reuse the latest snapshot with the new algorithm.

!!! tip

Try experimenting with multiple algorithms to explore their performance characteristics on your dataset. It's often hard to know which algorithm will be the best.

!!!

## Dashboard

The PostgresML dashboard makes it easy to compare various algorithms on your dataset. You can explore individual metrics & compare algorithms to each other, all trained on the same dataset for a fair benchmark.

![Model Selection](/dashboard/static/images/dashboard/models.png)
