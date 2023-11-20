---
description: A machine learning approach that uses labeled data
---

# Supervised Learning

PostgresML is a machine learning extension for PostgreSQL that enables you to perform training and inference using SQL queries.&#x20;

## Training

The training function is at the heart of PostgresML. It's a powerful single mechanism that can handle many different training tasks which are configurable with the function parameters.

### API

Most parameters are optional and have configured defaults. The `project_name` parameter is required and is an easily recognizable identifier to organize your work.

```sql
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

#### Parameters

<table data-full-width="false"><thead><tr><th></th><th></th><th></th></tr></thead><tbody><tr><td><strong>Parameter</strong></td><td><strong>Description</strong></td><td><strong>Example</strong></td></tr><tr><td><code>project_name</code></td><td>An easily recognizable identifier to organize your work.</td><td><code>My First PostgresML Project</code></td></tr><tr><td><code>task</code></td><td>The objective of the experiment: <code>regression</code> or <code>classification</code>.</td><td><code>classification</code></td></tr><tr><td><code>relation_name</code></td><td>The Postgres table or view where the training data is stored or defined.</td><td><code>public.users</code></td></tr><tr><td><code>y_column_name</code></td><td>The name of the label (aka "target" or "unknown") column in the training table.</td><td><code>is_bot</code></td></tr><tr><td><code>algorithm</code></td><td>The algorithm to train on the dataset, see <a data-mention href="regression.md">regression.md</a> and <a data-mention href="classification.md">classification.md</a> sections for supported algorithms</td><td><code>xgboost</code></td></tr><tr><td><code>hyperparams</code></td><td>The hyperparameters to pass to the algorithm for training, JSON formatted.</td><td><code>{ "n_estimators": 25 }</code></td></tr><tr><td><code>search</code></td><td>If set, PostgresML will perform a hyperparameter search to find the best hyperparameters for the algorithm. See <a href="../../../../../docs/training/hyperparameter_search">Hyperparameter Search</a> for details.</td><td><code>grid</code></td></tr><tr><td><code>search_params</code></td><td>Search parameters used in the hyperparameter search, using the scikit-learn notation, JSON formatted.</td><td><code>{ "n_estimators": [5, 10, 25, 100] }</code></td></tr><tr><td><code>search_args</code></td><td>Configuration parameters for the search, JSON formatted. Currently only <code>n_iter</code> is supported for <code>random</code> search.</td><td><code>{ "n_iter": 10 }</code></td></tr><tr><td><code>test_size</code></td><td>Fraction of the dataset to use for the test set and algorithm validation.</td><td><code>0.25</code></td></tr><tr><td><code>test_sampling</code></td><td>Algorithm used to fetch test data from the dataset: <code>random</code>, <code>first</code>, or <code>last</code>.</td><td><code>random</code></td></tr></tbody></table>

### Example

```sql
SELECT * FROM pgml.train(
    project_name => 'My Classification Project', 
    task => 'classification', 
    relation_name => 'pgml.digits',
    y_column_name => 'target'
);
```

This will create a **My Classification Project**, copy the `pgml.digits` table into the `pgml` schema, naming it `pgml.snapshot_{id}` where `id` is the primary key of the snapshot, and train a linear classification model on the snapshot using the `target` column as the label.



When used for the first time in a project, `pgml.train()` function requires the `task` parameter, which can be either `regression` or `classification`. The task determines the relevant metrics and analysis performed on the data. All models trained within the project will refer to those metrics and analysis for benchmarking and deployment.

The first time it's called, the function will also require a `relation_name` and `y_column_name`. The two arguments will be used to create the first snapshot of training and test data. By default, 25% of the data (specified by the `test_size` parameter) will be randomly sampled to measure the performance of the model after the `algorithm` has been trained on the 75% of the data.

{% hint style="info" %}
```sql
SELECT * FROM pgml.train(
    'My Classification Project',
    algorithm => 'xgboost'
);
```
{% endhint %}

!

Future calls to `pgml.train()` may restate the same `task` for a project or omit it, but they can't change it. Projects manage their deployed model using the metrics relevant to a particular task (e.g. `r2` or `f1`), so changing it would mean some models in the project are no longer directly comparable. In that case, it's better to start a new project.

{% hint style="info" %}
If you'd like to train multiple models on the same snapshot, follow up calls to `pgml.train()` may omit the `relation_name`, `y_column_name`, `test_size` and `test_sampling` arguments to reuse identical data with multiple algorithms or hyperparameters.
{% endhint %}

### Getting Training Data

A large part of the machine learning workflow is acquiring, cleaning, and preparing data for training algorithms. Naturally, we think Postgres is a great place to store your data. For the purpose of this example, we'll load a toy dataset, the classic handwritten digits image collection, from scikit-learn.



```sql
SELECT * FROM pgml.load_dataset('digits');
```

```plsql
pgml=# SELECT * FROM pgml.load_dataset('digits');
NOTICE:  table "digits" does not exist, skipping
 table_name  | rows
-------------+------
 pgml.digits | 1797
(1 row)
```

This `NOTICE` can safely be ignored. PostgresML attempts to do a clean reload by dropping the `pgml.digits` table if it exists. The first time this command is run, the table does not exist.



PostgresML loaded the Digits dataset into the `pgml.digits` table. You can examine the 2D arrays of image data, as well as the label in the `target` column:

```sql
SELECT
    target,
    image
FROM pgml.digits LIMIT 5;

```

```plsql
target |                                                                                image
-------+----------------------------------------------------------------------------------------------------------------------------------------------------------------------
     0 | {{0,0,5,13,9,1,0,0},{0,0,13,15,10,15,5,0},{0,3,15,2,0,11,8,0},{0,4,12,0,0,8,8,0},{0,5,8,0,0,9,8,0},{0,4,11,0,1,12,7,0},{0,2,14,5,10,12,0,0},{0,0,6,13,10,0,0,0}}
     1 | {{0,0,0,12,13,5,0,0},{0,0,0,11,16,9,0,0},{0,0,3,15,16,6,0,0},{0,7,15,16,16,2,0,0},{0,0,1,16,16,3,0,0},{0,0,1,16,16,6,0,0},{0,0,1,16,16,6,0,0},{0,0,0,11,16,10,0,0}}
     2 | {{0,0,0,4,15,12,0,0},{0,0,3,16,15,14,0,0},{0,0,8,13,8,16,0,0},{0,0,1,6,15,11,0,0},{0,1,8,13,15,1,0,0},{0,9,16,16,5,0,0,0},{0,3,13,16,16,11,5,0},{0,0,0,3,11,16,9,0}}
     3 | {{0,0,7,15,13,1,0,0},{0,8,13,6,15,4,0,0},{0,2,1,13,13,0,0,0},{0,0,2,15,11,1,0,0},{0,0,0,1,12,12,1,0},{0,0,0,0,1,10,8,0},{0,0,8,4,5,14,9,0},{0,0,7,13,13,9,0,0}}
     4 | {{0,0,0,1,11,0,0,0},{0,0,0,7,8,0,0,0},{0,0,1,13,6,2,2,0},{0,0,7,15,0,9,8,0},{0,5,16,10,0,16,6,0},{0,4,15,16,13,16,1,0},{0,0,0,3,15,10,0,0},{0,0,0,2,16,4,0,0}}
(5 rows)
```

### Training a Model

Now that we've got data, we're ready to train a model using an algorithm. We'll start with the default `linear` algorithm to demonstrate the basics. See the [Algorithms](../../../../../docs/training/algorithm\_selection) for a complete list of available algorithms.

```sql
SELECT * FROM pgml.train(
    'Handwritten Digit Image Classifier',
    'classification',
    'pgml.digits',
    'target'
);
```

```plsql
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

The output gives us information about the training run, including the `deployed` status. This is great news indicating training has successfully reached a new high score for the project's key metric and our new model was automatically deployed as the one that will be used to make new predictions for the project. See [Deployments](../../../../../docs/predictions/deployments) for a guide to managing the active model.

### Inspecting the results

Now we can inspect some of the artifacts a training run creates.

```sql
SELECT * FROM pgml.overview;
```

```plsql
pgml=# SELECT * FROM pgml.overview;
                name                |        deployed_at         |      task      | algorithm | runtime | relation_name | y_column_name | test_sampling | test_size
------------------------------------+----------------------------+----------------+-----------+---------+---------------+---------------+---------------+-----------
 Handwritten Digit Image Classifier | 2022-10-11 12:43:15.346482 | classification | linear    | python  | pgml.digits   | {target}      | last          |      0.25
(1 row)
```

## Inference

The `pgml.predict()` function is the key value proposition of PostgresML. It provides online predictions using the best, automatically deployed model for a project.

### API

The API for predictions is very simple and only requires two arguments: the project name and the features used for prediction.

```sql
select pgml.predict (
	project_name TEXT,
	features REAL[]
)
```

#### Parameters

| Parameter      | Description                                              | Example                       |
| -------------- | -------------------------------------------------------- | ----------------------------- |
| `project_name` | The project name used to train models in `pgml.train()`. | `My First PostgresML Project` |
| `features`     | The feature vector used to predict a novel data point.   | `ARRAY[0.1, 0.45, 1.0]`       |

#### Example

```
SELECT pgml.predict(
    'My Classification Project', 
    ARRAY[0.1, 2.0, 5.0]
) AS prediction;
```



where `ARRAY[0.1, 2.0, 5.0]` is the same type of features used in training, in the same order as in the training data table or view. This score can be used in other regular queries.

!!! example

```
SELECT *,
    pgml.predict(
        'Buy it Again',
        ARRAY[
            user.location_id,
            NOW() - user.created_at,
            user.total_purchases_in_dollars
        ]
    ) AS buying_score
FROM users
WHERE tenant_id = 5
ORDER BY buying_score
LIMIT 25;
```

!!!

### Example

If you've already been through the [Training Overview](../../../../../docs/training/overview), you can see the results of those efforts:

```sql
SELECT
    target,
    pgml.predict('Handwritten Digit Image Classifier', image) AS prediction
FROM pgml.digits 
LIMIT 10;
```

```plsql
 target | prediction
--------+------------
      0 |          0
      1 |          1
      2 |          2
      3 |          3
      4 |          4
      5 |          5
      6 |          6
      7 |          7
      8 |          8
      9 |          9
(10 rows)
```

### Active Model

Since it's so easy to train multiple algorithms with different hyperparameters, sometimes it's a good idea to know which deployed model is used to make predictions. You can find that out by querying the `pgml.deployed_models` view:

```sql
SELECT * FROM pgml.deployed_models;
```

```plsql
 id |                name                |      task      | algorithm | runtime |        deployed_at
----+------------------------------------+----------------+-----------+---------+----------------------------
  4 | Handwritten Digit Image Classifier | classification | xgboost   | rust    | 2022-10-11 13:06:26.473489
(1 row)
```

PostgresML will automatically deploy a model only if it has better metrics than existing ones, so it's safe to experiment with different algorithms and hyperparameters.

Take a look at [Deploying Models](../../../../../docs/predictions/deployments) documentation for more details.

### Specific Models

You may also specify a model\_id to predict rather than a project name, to use a particular training run. You can find model ids by querying the `pgml.models` table.

```sql
SELECT models.id, models.algorithm, models.metrics
FROM pgml.models
JOIN pgml.projects 
  ON projects.id = models.project_id
WHERE projects.name = 'Handwritten Digit Image Classifier';
```

```
 id | algorithm |                                                                                                         metrics

----+-----------+-------------------------------------------------------------------------------------------------------------------------------------------------------
-------------------------------------------------------------------
  1 | linear    | {"f1": 0.9190376400947571, "mcc": 0.9086633324623108, "recall": 0.9205743074417114, "accuracy": 0.9175946712493896, "fit_time": 0.8388963937759399, "p
recision": 0.9175060987472534, "score_time": 0.019625699147582054}
```

For example, making predictions with `model_id = 1`:

```sql
SELECT
    target,
    pgml.predict(1, image) AS prediction
FROM pgml.digits 
LIMIT 10;
```

```plsql
 target | prediction
--------+------------
      0 |          0
      1 |          1
      2 |          2
      3 |          3
      4 |          4
      5 |          5
      6 |          6
      7 |          7
      8 |          8
      9 |          9
(10 rows)
```
