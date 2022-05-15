# Hyperparameter tuning

Models can be further refined with the scikit cross validation hyperparameter search libraries. We currently support the [`grid`](https://scikit-learn.org/stable/modules/generated/sklearn.model_selection.GridSearchCV.html) and [`random`](https://scikit-learn.org/stable/modules/generated/sklearn.model_selection.RandomizedSearchCV.html) implementations.

## API
The arguments to pgml.train that begin with `search` are used for hyperparameter turning. 

```sql linenums="1" title="pgml.train" hl_lines="8-10"
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

- `search` can either be [`grid`](https://scikit-learn.org/stable/modules/generated/sklearn.model_selection.GridSearchCV.html) or [`random`](https://scikit-learn.org/stable/modules/generated/sklearn.model_selection.RandomizedSearchCV.html).
- `search_params` is the set of hyperparameters to search for your algorithm
- `search_args` are passed to the scikit learn model selection algorithm for extra configuration

search | description
--- | ---
grid | Trains every permutation of `search_params`
random | Randomly samples `search_params` to train models

You may pass any of the arguments listed in the algorithms documentation as hyperparameters. See [Algorithms](../training/#algorithms) for the complete list of algorithms and their associated documentation.


## Hyperparameter Search
This grid search will train `len(max_depth) * len(n_estimators) * len(learning_rate) = 6 * 4 * 4 = 96` combinations to compare all possible permutations of the `search_params`. It takes a couple of minutes on my computer, but you can delete some values if you want to speed things up. I like to watch all cores operate at 100% utilization in a separate terminal with `htop`.

=== "SQL"

    ```sql
    SELECT * FROM pgml.train(
        'Handwritten Digit Image Classifier', 
        algorithm => 'xgboost', 
        search => 'grid', 
        search_params => '{
            "max_depth": [1, 2, 3, 4, 5, 6], 
            "n_estimators": [20, 40, 80, 160],
            "learning_rate": [0.1, 0.2, 0.3, 0.4]
        }'
    );
    ```

=== "Output"

    ```
                project_name            | objective | algorithm_name |  status
    ------------------------------------+-----------+----------------+----------
     Handwritten Digit Image Classifier |           | xgboost        | deployed
    (1 row)
    ```

As you can see from the output, a new set model has been deployed with a better performance. There will also be a new analysis available on this model visible in the dashboard.

## Visualization

The optimal set of hyperparams is chosen for the model, and that combination is highlighted among all search candidates. The impact of each hyperparameter is measured against the key metric, as well as the training and test times. In this particular case, it's interesting that as `max_depth` increases, the "Test Score" on the key metric trends lower, so the smallest value of `max_depth` is chosen to maximize the "Test Score". Luckily, the smallest `max_depth` values also have the fastest "Fit Time", indicating that we pay less for training these higher quality models. It's a little less obvious how the different values `n_estimators` and `learning_rate` impact the test score. We may want to rerun our search and zoom in our out in the search space to get more insight.

![Hyperparameter Analysis](../images/hyperparams.png) 
