---
description: >-
  Pre-process and pull data to train a model using any of 50 different ML
  algorithms.
---

# pgml.train()

The training function is at the heart of PostgresML. It's a powerful single mechanism that can create models for regression, classification or clustering tasks. It also facilitates preprocessing inputs and hyperparam search.

## API

Most parameters are optional and have configured defaults. The `project_name` parameter is required and is an easily recognizable identifier to organize your work.

```postgresql
pgml.train(
    project_name TEXT,
    task TEXT DEFAULT NULL,
    relation_name TEXT DEFAULT NULL,
    y_column_name TEXT DEFAULT NULL,
    algorithm TEXT DEFAULT 'linear',
    hyperparams JSONB DEFAULT '{}'::JSONB,
    search TEXT DEFAULT NULL,
    search_params JSONB DEFAULT '{}'::JSONB,
    search_args JSONB DEFAULT '{}'::JSONB,
    test_size REAL DEFAULT 0.25,
    test_sampling TEXT DEFAULT 'random',
    preprocess JSONB DEFAULT '{}'::JSONB
)
```

### Parameters

| Parameter       | Example                                               | Description                                                                                                                                                                                                                                                                                  |
| --------------- | ----------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `project_name`  | `'Search Results Ranker'`                             | An easily recognizable identifier to organize your work.                                                                                                                                                                                                                                     |
| `task`          | `'regression'`                                        | The objective of the experiment: `regression`, `classification` or `cluster`                                                                                                                                                                                                                 |
| `relation_name` | `'public.search_logs'`                                | The Postgres table or view where the training data is stored or defined.                                                                                                                                                                                                                     |
| `y_column_name` | `'clicked'`                                           | The name of the label (aka "target" or "unknown") column in the training table.                                                                                                                                                                                                              |
| `algorithm`     | `'xgboost'`                                           | <p>The algorithm to train on the dataset, see the task specific pages for available algorithms:<br><a data-mention href="regression.md">regression.md</a></p><p><a data-mention href="classification.md">classification.md</a><br><a data-mention href="clustering.md">clustering.md</a></p> |
| `hyperparams`   | `{ "n_estimators": 25 }`                              | The hyperparameters to pass to the algorithm for training, JSON formatted.                                                                                                                                                                                                                   |
| `search`        | `grid`                                                | If set, PostgresML will perform a hyperparameter search to find the best hyperparameters for the algorithm. See [hyperparameter-search.md](../guides/supervised-learning/hyperparameter-search.md "mention") for details.                                                                                                  |
| `search_params` | `{ "n_estimators": [5, 10, 25, 100] }`                | Search parameters used in the hyperparameter search, using the scikit-learn notation, JSON formatted.                                                                                                                                                                                        |
| `search_args`   | `{ "n_iter": 10 }`                                    | Configuration parameters for the search, JSON formatted. Currently only `n_iter` is supported for `random` search.                                                                                                                                                                           |
| `test_size`     | `0.25`                                                | Fraction of the dataset to use for the test set and algorithm validation.                                                                                                                                                                                                                    |
| `test_sampling` | `random`                                              | Algorithm used to fetch test data from the dataset: `random`, `first`, or `last`.                                                                                                                                                                                                            |
| `preprocess`    | `{"col_name": {"impute": "mean", scale: "standard"}}` | Preprocessing steps to impute NULLS, encode categoricals and scale inputs. See [data-pre-processing.md](../guides/supervised-learning/data-pre-processing.md "mention") for details.                                                                                                                                       |

!!! example

```postgresql
SELECT * FROM pgml.train(
    project_name => 'My Classification Project', 
    task => 'classification', 
    relation_name => 'pgml.digits',
    y_column_name => 'target'
);
```

This will create a "My Classification Project", copy the `pgml.digits` table into the `pgml` schema, naming it `pgml.snapshot_{id}` where `id` is the primary key of the snapshot, and train a linear classification model on the snapshot using the `target` column as the label.

!!!

When used for the first time in a project, `pgml.train()` function requires the `task` parameter, which can be either `regression` or `classification`. The task determines the relevant metrics and analysis performed on the data. All models trained within the project will refer to those metrics and analysis for benchmarking and deployment.

The first time it's called, the function will also require a `relation_name` and `y_column_name`. The two arguments will be used to create the first snapshot of training and test data. By default, 25% of the data (specified by the `test_size` parameter) will be randomly sampled to measure the performance of the model after the `algorithm` has been trained on the 75% of the data.

!!! tip

```postgresql
SELECT * FROM pgml.train(
    'My Classification Project',
    algorithm => 'xgboost'
);
```

!!!

Future calls to `pgml.train()` may restate the same `task` for a project or omit it, but they can't change it. Projects manage their deployed model using the metrics relevant to a particular task (e.g. `r2` or `f1`), so changing it would mean some models in the project are no longer directly comparable. In that case, it's better to start a new project.

!!! tip

If you'd like to train multiple models on the same snapshot, follow up calls to `pgml.train()` may omit the `relation_name`, `y_column_name`, `test_size` and `test_sampling` arguments to reuse identical data with multiple algorithms or hyperparameters.

!!!
