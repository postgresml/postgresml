# Quick Start w/ Docker

We've prepared a Docker image that will allow you to quickly spin up a new PostgreSQL database with PostgreML already installed. It also includes some Scikit toy datasets so you can easily experiment with PostgresML workflows without having to import your own data.

You can skip to [Installation](/user_guides/setup/v2/installation/) for production installation instructions.

=== ":material-apple: OS X"

    [Install Docker for OS X](https://docs.docker.com/desktop/mac/install/).

=== ":material-linux: Linux"

    [Install Docker for Linux](https://docs.docker.com/engine/install/ubuntu/). Some package managers (e.g. Ubuntu/Debian) additionally require the `docker-compose` package to be installed separately.

=== ":material-microsoft-windows: Windows"

    [Install Docker for Windows](https://docs.docker.com/desktop/windows/install/). Use the Linux instructions if you're installing in Windows Subsystem for Linux.

1. Clone the repo:
```bash
git clone git@github.com:postgresml/postgresml.git
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
        pgml_development=# SELECT pgml.version();
         version
        ---------
         2.0.0
        (1 row)
        ```

## Quick Start

Here is a simple PostgresML workflow to get you started. We'll import a Scikit dataset, train a couple models on it and make real time predictions, all of it using only SQL.

1. Import the `digits` dataset:

    === "SQL"

        ```postgresql
        SELECT * FROM pgml.load_dataset('digits');
        ```
    === "Output"
        ```
        pgml=# SELECT * FROM pgml.load_dataset('digits');
        INFO:  num_features: 64, num_samples: 1797, feature_names: ["sepal length (cm)", "sepal width (cm)", "petal length (cm)", "petal width (cm)"]
         table_name  | rows 
        -------------+------
         pgml.digits | 1797
        (1 row)
        ```

2. Train an XGBoost model:

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
        pgml=# SELECT * FROM pgml.train('My First PostgresML Project',
            task => 'classification',
            relation_name => 'pgml.digits',
            y_column_name => 'target',
            algorithm => 'xgboost',
            hyperparams => '{
                "n_estimators": 25
            }'
        );
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

3. Train a LightGBM model:

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
        pgml=# SELECT * FROM pgml.train('My First PostgresML Project',
            task => 'classification',
            relation_name => 'pgml.digits',
            y_column_name => 'target',
            algorithm => 'lightgbm'
        );
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

        Looks like LightGBM did better with default hyperparameters. It's automatically deployed and will be used for inference.

4. Infer a few data point in real time:

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
        pgml=# SELECT                                                 
            target,
            pgml.predict('My First PostgresML Project', image) AS prediction 
        FROM pgml.digits
        LIMIT 5;

         target | prediction 
        --------+------------
              0 |          0
              1 |          1
              2 |          2
              3 |          3
              4 |          4
        ```


The following common machine learning tasks are performed automatically by PostgresML:

1. Snapshot the data so the experiment is reproducible
2. Split the dataset into train and test sets
3. Train and validate the model
4. Save it into the model store (a Postgres table)
5. Load it and cache it during inference

Check out our [Training](/user_guides/training/overview/) and [Predictions](/user_guides/predictions/overview/) documentation for more details. Some more advanced topics like [hyperparameter search](/user_guides/training/hyperparameter_search/) and [GPU acceleration](/user_guides/setup/gpu_support/) are available as well.

## Dashboard

The Dashboard app is available at https://localhost:8000. You can use it to write experiments in Jupyer-style notebooks, manage projects, and visualize datasets used by PostgresML.

![Dashboard](/images/dashboard/notebooks.png)
