# Training Models

The training function is at the heart of PostgresML. It's a powerful single mechanism that can handle many different training tasks which are configurable with the function parameters.

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
    test_sampling TEXT DEFAULT 'random'  
)
```

### Parameters

| **Parameter** | **Description** | **Example** |
----------------|-----------------|-------------|
| `project_name` | An easily recognizable identifier to organize your work. | `My First PostgresML Project` |
| `task` | The objective of the experiment: `regression` or `classification`. | `classification` |
| `relation_name` | The Postgres table or view where the training data is stored or defined. | `public.users` |
| `y_column_name` | The name of the label (aka "target" or "unknown") column in the training table. | `is_bot` |
| `algorithm` | The algorithm to train on the dataset, see [Algorithm Selection](/docs/guides/training/algorithm_selection/) for details. | `xgboost` |
| `hyperparams ` | The hyperparameters to pass to the algorithm for training, JSON formatted. | `{ "n_estimators": 25 }` |
| `search` | If set, PostgresML will perform a hyperparameter search to find the best hyperparameters for the algorithm. See [Hyperparameter Search](/docs/guides/training/hyperparameter_search/) for details. | `grid` |
| `search_params` | Search parameters used in the hyperparameter search, using the scikit-learn notation, JSON formatted. | ```{ "n_estimators": [5, 10, 25, 100] }``` |
| `search_args` | Configuration parameters for the search, JSON formatted. Currently only `n_iter` is supported for `random` search. | `{ "n_iter": 10 }` |
| `test_size ` | Fraction of the dataset to use for the test set and algorithm validation. | `0.25` |
| `test_sampling` | Algorithm used to fetch test data from the dataset: `random`, `first`, or `last`. | `random` |

!!! example

```postgresql
SELECT * FROM pgml.train(
    project_name => 'My Classification Project', 
    task => 'classification', 
    relation_name => 'pgml.digits',
    y_column_name => 'target'
);
```

This will create a "My Classification Project", copy the <code>pgml.digits</code> table into the <code>pgml</code> schema, naming it <code>pgml.snapshot_{id}</code> where <code>id</code> is the primary key of the snapshot, and train a linear classification model on the snapshot using the <code>target</code> column as the label.

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

If you'd like to train multiple models on the same snapshot, follow up calls to <code>pgml.train()</code> may omit the <code>relation_name</code>, <code>y_column_name</code>, <code>test_size</code> and <code>test_sampling</code> arguments to reuse identical data with multiple algorithms or hyperparameters.

!!!



## Getting Training Data

A large part of the machine learning workflow is acquiring, cleaning, and preparing data for training algorithms. Naturally, we think Postgres is a great place to store your data. For the purpose of this example, we'll load a toy dataset, the classic handwritten digits image collection, from scikit-learn.

=== "SQL"

```postgresql
SELECT * FROM pgml.load_dataset('digits');
```

=== "Output"

```
pgml=# SELECT * FROM pgml.load_dataset('digits');
NOTICE:  table "digits" does not exist, skipping
 table_name  | rows
-------------+------
 pgml.digits | 1797
(1 row)
```

This `NOTICE` can safely be ignored. PostgresML attempts to do a clean reload by dropping the `pgml.digits` table if it exists. The first time this command is run, the table does not exist.

===


PostgresML loaded the Digits dataset into the `pgml.digits` table. You can examine the 2D arrays of image data, as well as the label in the `target` column:

=== "SQL"

```postgresql
SELECT
    target,
    image
FROM pgml.digits LIMIT 5;

```

=== "Output"

```
target |                                                                                image
-------+----------------------------------------------------------------------------------------------------------------------------------------------------------------------
     0 | {{0,0,5,13,9,1,0,0},{0,0,13,15,10,15,5,0},{0,3,15,2,0,11,8,0},{0,4,12,0,0,8,8,0},{0,5,8,0,0,9,8,0},{0,4,11,0,1,12,7,0},{0,2,14,5,10,12,0,0},{0,0,6,13,10,0,0,0}}
     1 | {{0,0,0,12,13,5,0,0},{0,0,0,11,16,9,0,0},{0,0,3,15,16,6,0,0},{0,7,15,16,16,2,0,0},{0,0,1,16,16,3,0,0},{0,0,1,16,16,6,0,0},{0,0,1,16,16,6,0,0},{0,0,0,11,16,10,0,0}}
     2 | {{0,0,0,4,15,12,0,0},{0,0,3,16,15,14,0,0},{0,0,8,13,8,16,0,0},{0,0,1,6,15,11,0,0},{0,1,8,13,15,1,0,0},{0,9,16,16,5,0,0,0},{0,3,13,16,16,11,5,0},{0,0,0,3,11,16,9,0}}
     3 | {{0,0,7,15,13,1,0,0},{0,8,13,6,15,4,0,0},{0,2,1,13,13,0,0,0},{0,0,2,15,11,1,0,0},{0,0,0,1,12,12,1,0},{0,0,0,0,1,10,8,0},{0,0,8,4,5,14,9,0},{0,0,7,13,13,9,0,0}}
     4 | {{0,0,0,1,11,0,0,0},{0,0,0,7,8,0,0,0},{0,0,1,13,6,2,2,0},{0,0,7,15,0,9,8,0},{0,5,16,10,0,16,6,0},{0,4,15,16,13,16,1,0},{0,0,0,3,15,10,0,0},{0,0,0,2,16,4,0,0}}
(5 rows)
```

===

## Training a Model

Now that we've got data, we're ready to train a model using an algorithm. We'll start with the default `linear` algorithm to demonstrate the basics. See the [Algorithms](/docs/guides/training/algorithm_selection/) for a complete list of available algorithms.


=== "SQL"

```postgresql
SELECT * FROM pgml.train(
    'Handwritten Digit Image Classifier',
    'classification',
    'pgml.digits',
    'target'
);
```

=== "Output"

```
INFO:  Snapshotting table "pgml.digits", this may take a little while...
INFO:  Snapshot of table "pgml.digits" created and saved in "pgml"."snapshot_1"
INFO:  Dataset { num_features: 64, num_labels: 1, num_rows: 1797, num_train_rows: 1348, num_test_rows: 449 }
INFO:  Training Model { id: 1, algorithm: linear, runtime: python }
INFO:  Hyperparameter searches: 1, cross validation folds: 1
INFO:  Hyperparams: {}
INFO:  Metrics: {
  "f1": 0.91903764,
  "precision": 0.9175061,
  "recall": 0.9205743,
  "accuracy": 0.9175947,
  "mcc": 0.90866333,
  "fit_time": 0.17586434,
  "score_time": 0.01282608
}
              project               |      task      | algorithm | deployed
------------------------------------+----------------+-----------+----------
 Handwritten Digit Image Classifier | classification | linear    | t
(1 row)
```

===


The output gives us information about the training run, including the `deployed` status. This is great news indicating training has successfully reached a new high score for the project's key metric and our new model was automatically deployed as the one that will be used to make new predictions for the project. See [Deployments](/docs/guides/predictions/deployments/) for a guide to managing the active model.

## Inspecting the results
Now we can inspect some of the artifacts a training run creates. 

=== "SQL"

```postgresql
SELECT * FROM pgml.overview;
```

=== "Output"

```
pgml=# SELECT * FROM pgml.overview;
                name                |        deployed_at         |      task      | algorithm | runtime | relation_name | y_column_name | test_sampling | test_size
------------------------------------+----------------------------+----------------+-----------+---------+---------------+---------------+---------------+-----------
 Handwritten Digit Image Classifier | 2022-10-11 12:43:15.346482 | classification | linear    | python  | pgml.digits   | {target}      | last          |      0.25
(1 row)
```

===

## More Examples

See [examples](https://github.com/postgresml/postgresml/tree/master/pgml-extension/examples) in our git repository for more kinds of training with different types of features, algorithms and tasks.
