---
description: A machine learning approach that uses labeled data
---

# Supervised Learning

### Getting Training Data

A large part of the machine learning workflow is acquiring, cleaning, and preparing data for training algorithms. Naturally, we think Postgres is a great place to store your data. For the purpose of this example, we'll load a toy dataset, the classic handwritten digits image collection, from scikit-learn.

```postgresql
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

```postgresql
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

Now that we've got data, we're ready to train a model using an algorithm. We'll start with a classification task to demonstrate the basics. See [pgml.train](/docs/open-source/pgml/api/pgml.train) for a complete list of available algorithms and tasks.

```postgresql
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

The output gives us information about the training run, including the `deployed` status. This is great news indicating training has successfully reached a new high score for the project's key metric and our new model was automatically deployed as the one that will be used to make new predictions for the project.

### Inspecting the results

Now we can inspect some of the artifacts a training run creates.

```postgresql
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

```postgresql
select pgml.predict(
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

If you've executed the commands in this guide, you can see the results of those efforts:

```postgresql
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

```postgresql
SELECT * FROM pgml.deployed_models;
```

```plsql
 id |                name                |      task      | algorithm | runtime |        deployed_at
----+------------------------------------+----------------+-----------+---------+----------------------------
  4 | Handwritten Digit Image Classifier | classification | xgboost   | rust    | 2022-10-11 13:06:26.473489
(1 row)
```

PostgresML will automatically deploy a model only if it has better metrics than existing ones, so it's safe to experiment with different algorithms and hyperparameters.

Take a look at [pgml.deploy](/docs/open-source/pgml/api/pgml.deploy) documentation for more details.

### Specific Models

You may also specify a model\_id to predict rather than a project name, to use a particular training run. You can find model ids by querying the `pgml.models` table.

```postgresql
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

```postgresql
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
