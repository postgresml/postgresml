# Batch Predictions

## Batch Predictions

The `pgml.predict_batch()` function is a performance optimization which allows to return predictions for multiple rows in one function call. It works the same way as `pgml.predict()` in all other respects.

Many machine learning algorithms can benefit from calculating predictions in one operation instead of many, and batch predictions can be 3-6 times faster, for large datasets, than `pgml.predict()`.

### API

The API for batch predictions is very similar to individual predictions, and only requires two arguments: the project name and the _aggregated_ features used for predictions.

```postgresql
pgml.predict_batch(
    project_name TEXT,
    features REAL[]
)
```

### Parameters

| Parameter      | Example                       | Description                                                        |
| -------------- | ----------------------------- | ------------------------------------------------------------------ |
| `project_name` | `My first PostgresML project` | The project name used to train models in `pgml.train()`.           |
| `features`     | `array_agg(image)`            | An aggregate of feature vectors used to predict novel data points. |

!!! example

```postgresql
SELECT pgml.predict_batch(
    'My First PostgresML Project', 
    array_agg(ARRAY[0.1, 2.0, 5.0])
) AS prediction
FROM pgml.digits
```

!!!

Note that we are passing the result of `array_agg()` to our function because we want Postgres to accumulate all the features first, and only give it to PostgresML in one function call.

### Collecting Results

Batch predictions have to be fetched in a subquery or a CTE because they are using the `array_agg()` aggregate. To get the results back in an easily usable form, `pgml.predict_batch()` returns a `setof` result instead of a normal array, and that can be then built into a table:

\=== "SQL"

```postgresql
WITH predictions AS (
	SELECT pgml.predict_batch(
		'My Classification Project',
		array_agg(image)
	) AS prediction,
	unnest(
		array_agg(target)
	) AS target
	FROM pgml.digits
	WHERE target = 0
)
SELECT prediction, target FROM predictions
LIMIT 10;
```

\=== "Output"

```postgresql
 prediction | target 
------------+--------
          0 |      0
          0 |      0
          0 |      0
          0 |      0
          0 |      0
          0 |      0
          0 |      0
          0 |      0
          0 |      0
          0 |      0
(10 rows)
```

\===

Since we're using aggregates, one must take care to place limiting predicates into the `WHERE` clause of the CTE. For example, we used `WHERE target = 0` to batch predict images which are only classified into the `0` class.

#### Joins

To perform a join on batch predictions, it's necessary to have a uniquely identifiable join column for each row. As you saw in the example above, one can pass any column through the aggregation by using a combination of `unnest()` and `array_agg()`.

**Example**

```postgresql
WITH predictions AS (
	SELECT
		--
		-- Prediction
		--
		pgml.predict_batch(
			'My Bot Detector',
			array_agg(ARRAY[account_age, city, last_login])
		) AS prediction,

		--
		-- The pass-through unique identifier for each row
		--
		unnest(
			array_agg(user_id)
		) AS target
	FROM users

	--
	-- Filter which rows to pass to pgml.predict_batch()
	--
	WHERE last_login > NOW() - INTERVAL '1 minute'
)
SELECT prediction, email, ip_address
FROM users
INNER JOIN predictions
ON users.user_id = predictions.user_id
```
