# Hyperparameter Search

Models can be further refined by using hyperparameter search and cross validation. We currently support `random` and `grid` search algorithms, and k-fold cross validation.

## API

The parameters passed to `pgml.train()` easily allow one to perform hyperparameter tuning. The three parameters relevant to this are: `search`, `search_params` and `search_args`.

| **Parameter**   | **Example**                   |
| --------------- | ----------------------------- |
| `search`        | `grid`                        |
| `search_params` | `{"alpha": [0.1, 0.2, 0.5] }` |
| `search_args`   | `{"n_iter": 10 }`             |

```postgresql
SELECT * FROM pgml.train(
    'Handwritten Digit Image Classifier', 
    algorithm => 'xgboost', 
    search => 'grid', 
    search_params => '{
        "max_depth": [1, 2, 3, 4, 5, 6], 
        "n_estimators": [20, 40, 80, 160]
    }'
);
```

You may pass any of the arguments listed in the algorithms documentation as hyperparameters. See [Algorithms](../../../../../../docs/training/algorithm\_selection/) for the complete list of algorithms and their associated hyperparameters.

### Search Algorithms

We currently support two search algorithms: `random` and `grid`.

| Algorithm | Description                                                                                     |
| --------- | ----------------------------------------------------------------------------------------------- |
| `grid`    | Trains every permutation of `search_params` using a cartesian product.                          |
| `random`  | Randomly samples `search_params` up to `n_iter` number of iterations provided in `search_args`. |

### Analysis

PostgresML automatically selects the optimal set of hyperparameters for the model, and that combination is highlighted in the Dashboard, among all other search candidates.

The impact of each hyperparameter is measured against the key metric (`r2` for regression and `f1` for classification), as well as the training and test times.

{% hint style="info" %}
In our example case, it's interesting that as \`max\_depth\` increases, the "Test Score" on the key metric trends lower, so the smallest value of max\_depth is chosen to maximize the "Test Score".

Luckily, the smallest `max_depth` values also have the fastest "Fit Time", indicating that we pay less for training these higher quality models.

It's a little less obvious how the different values \`n\_estimators\` and `learning_rate` impact the test score. We may want to rerun our search and zoom in on our the search space to get more insight.
{% endhint %}

### Performance

In our example above, the grid search will train `len(max_depth) * len(n_estimators) * len(learning_rate) = 6 * 4 * 4 = 96` combinations to compare all possible permutations of `search_params`.

It only took about a minute on my computer because we're using optimized Rust/C++ XGBoost bindings, but you can delete some values if you want to speed things up even further. I like to watch all cores operate at 100% utilization in a separate terminal with `htop`:

In the end, we get the following output:

```plsql
              project               |      task      | algorithm | deployed
------------------------------------+----------------+-----------+----------
 Handwritten Digit Image Classifier | classification | xgboost   | t
(1 row)
```

A new model has been deployed with better performance and metrics. There will also be a new analysis available for this model, viewable in the dashboard.
