# Product description

This document describes the value proposition of this product.

## The problem

Machine learning is a hard to take advantage of for most startups. They either don't have the time or the knowhow
to deploy ML models into production. This problem exists for multi-billion dollar enteprises, it's 10x true
for small startups.

Python ecosystem is also hard to manage. Common problems are dependency hell and Python version conflicts.
Most of the time, engineers just want to train and deploy an algorithm; everything else is distraction.

Data is kept in databases that are hard for ML algorithms to access: MySQL, Postgres, Dynamo, etc.
The typical ML workflow is:

1. export data to a warehouse (e.g. Snowflake) or S3 (CSVs),
2. run a Python script that will train the model (while fighting through dependency hell),
3. pickle the model and upload it to object storage,
4. download and unpickle the model in production, behind an HTTP API,
5. serve predictions in a microservice.

By the time this workflow completes, the data is obsolete, the algorithm is wrong and the ML engineer
is polishing their CV or considering farming as an alternative career path.

## The solution

Colocate data and machine learning together in one system, train the models online, and run predictions
from the same system with a simple command. That system in our case is Postgres, because that's where most
startups keep their data. Postgres happens to be highly extendable as well, which makes our job easier.

The new workflow is now:

1. define the data with a SQL query (i.e. a view),
2. train an algorithm with a single command,
3. serve predictions with a SQL query.

No Python, no code of any kind really, no dependencies, no exports, imports, transforms,
S3 permission issues, deploys or JSON/GraphQL; from prototype to production in about 5 minutes.

Here is an example:

#### Define the data with a SQL query

```sql
CREATE VIEW my_data AS
    SELECT NOW() - created_at AS user_tenure,
           age,
           location,
           total_purchases,
    FROM users
    CROSS JOIN LATERAL (
        SELECT SUM(purchase_price) AS total_purchases FROM orders
        WHERE user_id = users.id
    );
```

#### Train the model

The function `pgml.train` accepts three arguments:

- the model name
- the `y` column for the algorithm,
- the algorithm to use, defaults to Linear Regression.

```sql
SELECT pgml.train('my_data', 'total_purchases');
```

#### Serve the model

The model is ready for serving! Let's serve this via SQL again:

```sql
SELECT pgml.score('my_model_1', '2 years'::interval) AS likely_purchase_amount_based_on_tenure;
```

You can call this directly from your app, no special infrastructure required.
