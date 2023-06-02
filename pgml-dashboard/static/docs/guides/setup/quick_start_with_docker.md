# Quick Start w/ Docker

We've prepared a Docker image that will allow you to quickly spin up a new PostgreSQL database with PostgresML already installed. It also includes some Scikit toy datasets so you can easily experiment with PostgresML workflows without having to import your own data.

You can skip to [Installation](/docs/guides/setup/v2/installation/) for production installation instructions.


=== "OS X"

<p><a href="https://docs.docker.com/desktop/mac/install/">Install Docker for OS X</a>.</p>

=== "Linux"

<p><a href="https://docs.docker.com/engine/install/ubuntu/">Install Docker for Linux</a>. Some package managers (e.g. Ubuntu/Debian) additionally require the <code>docker-compose</code> package to be installed separately.</p>

=== "Windows"

<p><a href="https://docs.docker.com/desktop/windows/install/">Install Docker for Windows</a>. Use the Linux instructions if you're installing in Windows Subsystem for Linux.</p>

===



1. Clone the repo:
    ```bash
    git clone https://github.com/postgresml/postgresml
    ```

2. Start Dockerized services. PostgresML will run on port 5433, just in case you already have Postgres running:
    ```bash
    cd postgresml && docker-compose up
    ```

3. Connect to Postgres in the container with PostgresML installed:
    ```bash
    psql postgres://postgres@localhost:5433/pgml_development
    ```

4. Validate your installation:


    === "SQL"

    ```postgresql
    SELECT pgml.version();
    ```

    === "Output"

    ```
     version
    ---------
     2.0.0
    (1 row)
    ```

    ===

5. Browse the dashboard on <a href="http://localhost:8000/" target="_blank">localhost:8000</a>


!!! note

If you'd like to preserve your database over multiple docker sessions, use `docker-compose stop` or `ctrl+c` when you shut down the containers. `docker-compose down` will remove the docker volumes, and completely reset the database.

!!!


## Basic Workflow

Here is a simple PostgresML example to get you started. We'll import a Scikit dataset, train a couple models on it and make real time predictions, all of it using only SQL.

#### Importing a Dataset

PostgresML comes with a few built-in datasets. You can also import your own CSV files. Let's import the `digits` dataset from Scikit:

=== "SQL"

```postgresql
SELECT * FROM pgml.load_dataset('digits');
```

=== "Output"

```
INFO:  num_features: 64, num_samples: 1797, feature_names: ["sepal length (cm)", "sepal width (cm)", "petal length (cm)", "petal width (cm)"
 table_name  | rows
-------------+------
 pgml.digits | 1797
(1 row)
```

===

#### Trainig a model

Now that we have a dataset, we can train a model on it. Let's train a simple XGBoost model:

=== "SQL"

```postgresql
SELECT * FROM pgml.train('My First PostgresML Project',
    task => 'classification',
    relation_name => 'pgml.digits',
    y_column_name => 'target',
    algorithm => 'xgboost',
    hyperparams => '{
        "n_estimators": 25
    }'
);
```

=== "Output"

```
INFO:  Snapshotting table "pgml.digits", this may take a little while...
INFO:  Snapshot of table "pgml.digits" created and saved in "pgml"."snapshot_1"
INFO:  Dataset { num_features: 64, num_labels: 1, num_rows: 1797, num_train_rows: 1348, num_test_rows: 449 }
INFO:  Training Model { id: 15, algorithm: xgboost, runtime: rust }
INFO:  Hyperparameter searches: 1, cross validation folds: 1
INFO:  Hyperparams: {
  "n_estimators": 25
}
INFO:  Metrics: {
  "f1": 0.88522536,
  "precision": 0.8835865,
  "recall": 0.88687027,
  "accuracy": 0.8841871,
  "mcc": 0.87189955,
  "fit_time": 0.44059604,
  "score_time": 0.005983766
}
           project           |      task      | algorithm | deployed 
-----------------------------+----------------+-----------+----------
 My first PostgresML project | classification | xgboost   | t
(1 row)
```

===

Training a LightGBM model is equally simple:

=== "SQL"

```postgresql
SELECT * FROM pgml.train('My First PostgresML Project',
    task => 'classification',
    relation_name => 'pgml.digits',
    y_column_name => 'target',
    algorithm => 'lightgbm'
);
```

=== "Output"

```
INFO:  Snapshotting table "pgml.digits", this may take a little while...
INFO:  Snapshot of table "pgml.digits" created and saved in "pgml"."snapshot_18"
INFO:  Dataset { num_features: 64, num_labels: 1, num_rows: 1797, num_train_rows: 1348, num_test_rows: 449 }
INFO:  Training Model { id: 16, algorithm: lightgbm, runtime: rust }
INFO:  Hyperparameter searches: 1, cross validation folds: 1
INFO:  Hyperparams: {}
INFO:  Metrics: {
  "f1": 0.91579026,
  "precision": 0.915012,
  "recall": 0.9165698,
  "accuracy": 0.9153675,
  "mcc": 0.9063865,
  "fit_time": 0.27111048,
  "score_time": 0.004169579
}
           project           |      task      | algorithm | deployed 
-----------------------------+----------------+-----------+----------
 My first PostgresML project | classification | lightgbm  | t
(1 row)
```

===


#### Making predictions

After training a model, you can use it to make predictions. PostgresML provides a `pgml.predict` function that takes a model name and a feature vector, and returns the predicted label:


=== "SQL"

```postgresql
SELECT 
    target,
    pgml.predict('My First PostgresML Project', image) AS prediction
FROM pgml.digits
LIMIT 5;
```

=== "Output"

```
 target | prediction 
--------+------------
      0 |          0
      1 |          1
      2 |          2
      3 |          3
      4 |          4
```

===

The following common machine learning tasks are performed automatically by PostgresML:

1. Snapshot the data so the experiment is reproducible
2. Split the dataset into train and test sets
3. Train and validate the model
4. Save it into the model store (a Postgres table)
5. Load it and cache it during inference

Check out our [Training](/docs/guides/training/overview/) and [Predictions](/docs/user_guides/predictions/overview/) documentation for more details. Some more advanced topics like [hyperparameter search](/docs/user_guides/training/hyperparameter_search/) and [GPU acceleration](/docs/user_guides/setup/gpu_support/) are available as well.

## Dashboard

The Dashboard app is available at <a href="http://localhost:8000/" target="_blank">localhost:8000</a>. You can use it to write experiments in Jupyter-style notebooks, manage projects, and visualize datasets used by PostgresML.

![Dashboard](/dashboard/static/images/dashboard/notebooks.png)
