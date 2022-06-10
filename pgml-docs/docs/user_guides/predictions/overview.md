# Predictions

The predict function is the key value proposition of PostgresML. It provides online predictions using the actively deployed model for a project.

## API

```sql linenums="1" title="pgml.predict"
pgml.predict (
	project_name TEXT,            -- Human-friendly project name
	features DOUBLE PRECISION[]   -- Must match the training data column order
)
```

!!! example
    Once a model has been trained for a project, making predictions is as simple as:
    
    ```sql linenums="1"
    SELECT pgml.predict(
        'Human-friendly project name', 
        ARRAY[...]
    ) AS prediction_score;
    ```

where `ARRAY[...]` is the same list of features for a sample used in training. This score can be used in normal queries, for example:

!!! example
    ```sql linenums="1"
    SELECT *,
        pgml.predict(
            'Probability of buying our products',
            ARRAY[user.location, NOW() - user.created_at, user.total_purchases_in_dollars]
        ) AS likely_to_buy_score
    FROM users
    WHERE comapany_id = 5
    ORDER BY likely_to_buy_score
    LIMIT 25;
    ```


## Making Predictions

If you've already been through the [training guide](/user_guides/training/overview/), you can see the results of those efforts:

=== "SQL"

    ```sql linenums="1"
    SELECT target, pgml.predict('Handwritten Digit Image Classifier', image) AS prediction
    FROM pgml.digits 
    LIMIT 10;
    ```

=== "Output"

    ```sql linenums="1"
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

## Checking the deployed algorithm
If you're ever curious about which deployed models will be used to make predictions, you can see them in the `pgml.deployed_models` VIEW.

=== "SQL"

    ```sql linenums="1"
    SELECT * FROM pgml.deployed_models;
    ```

=== "Output"

    ```sql linenums="1"
     id |                name                |      task      | algorithm_name |        deployed_at
    ----+------------------------------------+----------------+----------------+----------------------------
      1 | Handwritten Digit Image Classifier | classification | linear         | 2022-05-10 15:28:53.383893
    ```

