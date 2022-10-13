
In this blog post, we'll compare the unified PostgresML v2.0 architecture with the decentralized architecture of Python machine learning microservices. Our goal is to demonstrate that doing inference at the data layer with Rust is much faster and simpler than with standard architectures used across the industry today.

## Setting the stage

### Data

We'll be using the [Flight Status Prediction](https://www.kaggle.com/datasets/robikscube/flight-delay-dataset-20182022) dataset from Kaggle which ends up being being about 1 GB of floating point features. We won't be using all columns because some of them are redundant, e.g. airport name and airport identifier.

### Algorithm

We'll be training an XGBoost model with default hyperparameters and 25 estimators (also known as boosting rounds).

### Architectures

#### PostgresML

PostgresML architecture will be a single PostgreSQL server with the PostgresML 2.0 extension installed, as documented in our [Installation](/user_guides/setup/v2/installation/) instructions. The client will be a Postgres client using [pgbench](https://www.postgresql.org/docs/current/pgbench.html).


#### Python

Python architecture will be composed of:

1. Flask & Gunicorn HTTP/1.1 server speaking JSON
2. CSV file containing the dataset for training
3. Redis feature store containing the dataset for inference
4. Python client using [Locust](https://locust.io/)

## Results

Python training RAM: 11.6GB

## Methodology

