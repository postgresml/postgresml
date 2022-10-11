# Training

The training function is at the heart of PostgresML. It's a powerful single mechanism that can handle many difference training tasks which are configurable with the function parameters.

## API

Most parameters are optional and have configured defaults. The `project_name` parameter is required and is an easily recognizable identifier to organize your work.

```postgresql title="pgml.train()"
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
| `algorithm` | The algorithm to train on the dataset, see [Algorithm Selection](/user_guides/training/algorithm_selection/) for details. | `xgboost` |
| `hyperparams ` | The hyperparameters to pass to the algorithm for training, JSON formatted. | `{ "n_estimators": 25 }` |
| `search` | If set, PostgresML will perform a hyperparameter search to find the best hyperparameters for the algorithm. See [Hyperparameter Search](/user_guides/training/hyperparameter_search/) for details. | `grid` |
| `search_params` | Search parameters used in the hyperparameter search, using the scikit-learn notation, JSON formatted. | ```{ "n_estimators": [5, 10, 25, 100] }``` |
| `search_args` | Configuration parameters for the search. Currently only `n_iter` is supported for `random` search. | `{ "n_iter": 10 }` |
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

    This will create a "My Classification Project", copy the `pgml.digits` table into the `pgml` schema, naming it `pgml.snapshot_{id}` where `id` is the primary key of the snapshot, and train a linear classification model on the snapshot using the `target` column as the label.

When used for the first time in a project, `pgml.train()` function requires the `task` parameter, which can be either `regression` or `classification`. The task determines the relevant metrics and analysis performed on the data. All models trained within the project will refer to those metrics and analysis for benchmarking and deployment.

The first time it's called, the function will also require a `relation_name` and `y_column_name`. The two arguments will be used to create the first snapshot of training and test data. By default, 25% of the data (specified by the `test_size` parameter) will be randomly sampled to measure the performance of the model after the `algorithm` has been trained on the 75% of the data. 

!!! tip
    Postgres supports named arguments in functions, so you can easily regonize them and pass them as needed:

    ```postgresql
    SELECT * FROM pgml.train(
        'My Classification Project',
        algorithm => 'xgboost'
    );
    ```

Future calls to `pgml.train()` may restate the same `task` for a project or omit it, but they can't change it. Projects manage their deployed model using the metrics relevant to a particular task (e.g. `r2` or `f1`), so changing it would mean some models in the project are no longer directly comparable. In that case, it's better to start a new project.

!!! note
    If you'd like to train multiple models on the same snapshot, follow up calls to `pgml.train()` may omit the `relation_name`, `y_column_name`, `test_size` and `test_sampling` arguments to reuse identical data with multiple algorithms or hyperparameters.

    The snapshot is always saved after training runs if any follow up analysis required.



## Getting training data
A large part of the machine learning workflow is acquiring, cleaning, and preparing data for training algorithms. Naturally, we think Postgres is a great place to store your data. For the purpose of this example, we'll load a toy dataset, the classic handwritten digits image collection, from scikit-learn.

=== "SQL"

    ```postgresql
    SELECT pgml.load_dataset('digits');
    ```

=== "Output"

    ```
    NOTICE:  table "pgml.digits" does not exist, skipping -- (1)
    load_dataset
    --------------
    OK
    (1 row)
    ```

    1. This NOTICE can safely be ignored. PostgresML attempts to do a clean reload by dropping the `pgml.digits` table if it exists. The first time this command is run, the table does not exist.


PostgresML loads this into the table `pgml.digits`. You can examine the 2D arrays of image data, as well as the label in the `target` column.

=== "SQL"

    ```postgresql
    SELECT target, image FROM pgml.digits LIMIT 5;
    ```

=== "Output"

    ```
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
Now that we've got data, we're ready to train a model using an algorithm. We'll start with the default `linear` algorithm to demonstrate the basics. See the [Algorithms](/user_guides/training/algorithm_selection/) reference for a complete list of available choices.

=== "SQL"

    ```sql linenums="1"
    SELECT * FROM pgml.train('Handwritten Digit Image Classifier', 'classification', 'pgml.digits', 'target');
    ```

=== "Output"

    ```sql linenums="1"
                project_name            |     task       | algorithm |  status
    ------------------------------------+----------------+-----------+----------
     Handwritten Digit Image Classifier | classification | linear    | deployed
    (1 row)
    ```

The output gives us information about the training run, including the `deployed` status. This is great news indicating training has successfully reached a new high score for the project's key metric and our new model was automatically deployed as the one that will be used to make new predictions for the project. See [Deployments](/user_guides/predictions/deployments/) for a guide to managing the active model.

## Inspecting the results
Now we can inspect some of the artifacts a training run creates. 

=== "SQL"

    ```sql linenums="1"
    SELECT * FROM pgml.overview;
    ```

=== "Output"

    ```sql linenums="1"
                    name                |        deployed_at         |     task       | algorithm | relation_name | y_column_name | test_sampling | test_size
    ------------------------------------+----------------------------+----------------+-----------+---------------+---------------+---------------+-----------
     Handwritten Digit Image Classifier | 2022-05-10 15:06:32.824305 | classification | linear    | pgml.digits   | {target}      | random        |      0.25
    (1 row)
    ```

- See the [Examples](https://github.com/postgresml/postgresml/tree/master/pgml-extension/examples) for more kinds of training with different types of features, algorithms and tasks.
- See the [Models](/user_guides/schema/models/) reference for a complete description of the artifacts.
