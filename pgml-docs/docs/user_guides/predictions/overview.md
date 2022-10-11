# Making Predictions

The `pgml.predict()` function is the key value proposition of PostgresML. It provides online predictions using the best, automatically deployed model for a project.

## API

The API for predictions is very simple and only requires two arguments: the project name and the features used for prediction.

```postgresql title="pgml.predict()"
pgml.predict (
	project_name TEXT,
	features REAL[]
)
```

### Parameters

| Parameter | Description | Example |
|-----------|-------------|---------|
| `project_name`| The project name used to train models in `pgml.train()`. | `My First PostgresML Project` |
| `features` | The feature vector used to predict a novel data point. | `ARRAY[0.1, 0.45, 1.0]` |

!!! example
    
    ```postgresql
    SELECT pgml.predict(
        'My Classification Project', 
        ARRAY[0.1, 2.0, 5.0]
    ) AS prediction;
    ```

where `ARRAY[0.1, 2.0, 5.0]` is the same type of features used in training, in the same order as in the training data table or view. This score can be used in other regular queries.

!!! example
    ```postgresql
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


### Example

If you've already been through the [Training Overview](/user_guides/training/overview/), you can see the results of those efforts:

=== "SQL"

    ```postgresql
    SELECT
        target,
        pgml.predict('Handwritten Digit Image Classifier', image) AS prediction
    FROM pgml.digits 
    LIMIT 10;
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
          5 |          5
          6 |          6
          7 |          7
          8 |          8
          9 |          9
    (10 rows)
    ```

## Active Model

Since it's so easy to train multiple algorithms with different hyperparameters, sometimes it's a good idea to know which deployed model is used to make predictions. You can find that out by querying the `pgml.deployed_models` view:

=== "SQL"

    ```postgresql
    SELECT * FROM pgml.deployed_models;
    ```

=== "Output"

    ```
     id |                name                |      task      | algorithm | runtime |        deployed_at
    ----+------------------------------------+----------------+-----------+---------+----------------------------
      4 | Handwritten Digit Image Classifier | classification | xgboost   | rust    | 2022-10-11 13:06:26.473489
    (1 row)
    ```

PostgresML will automatically deploy a model only if it has better metrics than existing ones, so it's safe to experiment with different algorithms and hyperparameters.

Take a look at [Deploying Models](/user_guides/predictions/deployments/) documentation for more details.
