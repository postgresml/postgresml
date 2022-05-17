# Training

The training function is at the heart of PostgresML. It's a powerful single call that can handle the different objectives of training depending on the arguments passed.

## API
Most parameters are optional other than the `project_name` which is a simple human readable identifier to organize your work. 

```sql linenums="1" title="pgml.train"
 pgml.train(
	project_name TEXT,                       -- Human-friendly project name
	objective TEXT DEFAULT NULL,             -- 'regression' or 'classification'
	relation_name TEXT DEFAULT NULL,         -- name of table or view
	y_column_name TEXT DEFAULT NULL,         -- aka "label" or "unknown" or "target"
	algorithm TEXT DEFAULT 'linear',         -- statistical learning method
	hyperparams JSONB DEFAULT '{}'::JSONB,   -- options for the model
	search TEXT DEFAULT NULL,                -- hyperparam tuning, 'grid' or 'random'
	search_params JSONB DEFAULT '{}'::JSONB, -- hyperparam search space
	search_args JSONB DEFAULT '{}'::JSONB,   -- hyperparam options
	test_size REAL DEFAULT 0.25,             -- fraction of the data for the test set
	test_sampling TEXT DEFAULT 'random'      -- 'random', 'first' or 'last'  
)
```

!!! example
    A minimal first call for a project looks like:

    ```SQL
    SELECT * FROM pgml.train(
        'My Classification Project', 
        'classification', 
        'my_table_name', 
        'my_tables_target_column_name'
    );
    ```

The `train` function requires an `objective` the first time a `project_name` is used. That objective is either `regression` or `classification`, which determines the relevant metrics and analysis performed for models trained toward a common goal. It also requires a `relation_name` and `y_column_name` that will be used to establish the first `Snapshot` of training and test data. By default, 25% of the data (specified by `test_size`) will be randomly sampled to measure the performance of the model after the `algorithm` has been fit to the rest. 

!!! tip
    Postgres supports named arguments for function calls, which allows you to pass only the arguments you need.

    ```SQL
        pgml.train('Project Name', algorithm => 'xgboost')
    ```

Future calls to `train` may restate the same `objective` for a project, or omit it, but can't change it. Projects manage their active model using the metrics relevant to a particular objective, so changing it would mean some models in the project are no longer directly comparable. In that case, it's better to start a new project.

!!! note
    If you'd like to train multiple models on the same `Snapshot`, follow up calls to `train` may omit the `relation_name`, `y_column_name`, `test_size` and `test_sampling` arguments to reuse identical data with multiple algorithms or hyperparams. The `Snapshot` is also saved after training runs for any follow up analysis required.


## Algorithms

We currently support the following regression and classification algorithms from [Scikit-Learn](https://scikit-learn.org/) and [XGBoost](https://xgboost.readthedocs.io/). You may pass any of these to the training function. The `hyperparams` argument will pass parameters on. See the associated documentation for valid hyperparameters of each algorithm.

### XGBoost
Algorithm | Regression | Classification
--- | --- | ---
`xgboost` | [XGBRegressor](https://xgboost.readthedocs.io/en/stable/python/python_api.html#xgboost.XGBRegressor) | [XGBClassifier](https://xgboost.readthedocs.io/en/stable/python/python_api.html#xgboost.XGBClassifier)
`xgboost_random_forest` | [XGBRFRegressor](https://xgboost.readthedocs.io/en/stable/python/python_api.html#xgboost.XGBRFRegressor) | [XGBRFClassifier](https://xgboost.readthedocs.io/en/stable/python/python_api.html#xgboost.XGBRFClassifier)

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

## Getting training data
A large part of machine learning is acquiring, cleaning and preparing data for algorithms. Naturally, we think Postgres is a great place to store your data. For the purpose of this example, we'll load a toy dataset, a classic handwritten digits image collection from scikit-learn.

=== "SQL"

    ```sql linenums="1"
    pgml_development=# SELECT pgml.load_dataset('digits');
    ```

=== "Output"

    ```sql linenums="1"
    NOTICE:  table "digits" does not exist, skipping -- (1)
    load_dataset
    --------------
    OK
    (1 row)
    ```

    1. This NOTICE can safely be ignored. PostgresML attempts to do a clean reload by dropping the `pgml.digits` table if it exists. The first time this command is run, the table does not exist.


PostgresML loads this into a fixed table `pgml.digits`. You can examine the 2D arrays of image data, as well as the label in the `target` column.

=== "SQL"

    ```sql linenums="1"
    pgml_development=# SELECT target, image FROM pgml.digits LIMIT 5;
    ```

=== "Output"

    ```sql linenums="1"
    target |                                                                                image
    --------+----------------------------------------------------------------------------------------------------------------------------------------------------------------------
         0 | {{0,0,5,13,9,1,0,0},{0,0,13,15,10,15,5,0},{0,3,15,2,0,11,8,0},{0,4,12,0,0,8,8,0},{0,5,8,0,0,9,8,0},{0,4,11,0,1,12,7,0},{0,2,14,5,10,12,0,0},{0,0,6,13,10,0,0,0}}
         1 | {{0,0,0,12,13,5,0,0},{0,0,0,11,16,9,0,0},{0,0,3,15,16,6,0,0},{0,7,15,16,16,2,0,0},{0,0,1,16,16,3,0,0},{0,0,1,16,16,6,0,0},{0,0,1,16,16,6,0,0},{0,0,0,11,16,10,0,0}}
         2 | {{0,0,0,4,15,12,0,0},{0,0,3,16,15,14,0,0},{0,0,8,13,8,16,0,0},{0,0,1,6,15,11,0,0},{0,1,8,13,15,1,0,0},{0,9,16,16,5,0,0,0},{0,3,13,16,16,11,5,0},{0,0,0,3,11,16,9,0}}
         3 | {{0,0,7,15,13,1,0,0},{0,8,13,6,15,4,0,0},{0,2,1,13,13,0,0,0},{0,0,2,15,11,1,0,0},{0,0,0,1,12,12,1,0},{0,0,0,0,1,10,8,0},{0,0,8,4,5,14,9,0},{0,0,7,13,13,9,0,0}}
         4 | {{0,0,0,1,11,0,0,0},{0,0,0,7,8,0,0,0},{0,0,1,13,6,2,2,0},{0,0,7,15,0,9,8,0},{0,5,16,10,0,16,6,0},{0,4,15,16,13,16,1,0},{0,0,0,3,15,10,0,0},{0,0,0,2,16,4,0,0}}
    (5 rows)
    ```

## Training the model
Now that we've got data, we're ready to train a model using an algorithm. We'll start with the default `linear` algorithm to demonstrate the basics. See the [Algorithms](#algorithms) reference for a complete list of available choices.

=== "SQL"

    ```sql linenums="1"
    SELECT * FROM pgml.train('Handwritten Digit Image Classifier', 'classification', 'pgml.digits', 'target');
    ```

=== "Output"

    ```sql linenumes="1"
                project_name            |   objective    | algorithm_name |  status
    ------------------------------------+----------------+----------------+----------
     Handwritten Digit Image Classifier | classification | linear         | deployed
    (1 row)
    ```

The output gives us information about the training run, including the `deployed` status. This is great news indicating training has successfully reached a new high score for the project's key metric and our new model was automatically deployed as the one that will be used to make new predictions for the project. See [Deployments](../guides/deployments/) for a guide to managing the active model.

## Inspecting the results
Now we can inspect some of the artifacts a training run creates. 

=== "SQL"

    ```sql linenums="1"
    SELECT * FROM pgml.overview;
    ```

=== "Output"

    ```sql linenums="1"
                    name                |        deployed_at         |   objective    | algorithm_name | relation_name | y_column_name | test_sampling | test_size
    ------------------------------------+----------------------------+----------------+----------------+---------------+---------------+---------------+-----------
     Handwritten Digit Image Classifier | 2022-05-10 15:06:32.824305 | classification | linear         | pgml.digits   | {target}      | random        |      0.25
    (1 row)
    ```

See the [Models](../../references/models/) reference for a complete description of the artifacts.
