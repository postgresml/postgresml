# PostgresML

![PostgresML](./logo-small.png)

[![CircleCI](https://circleci.com/gh/postgresml/postgresml/tree/master.svg?style=shield)](https://circleci.com/gh/postgresml/postgresml/tree/master)

PostgresML is an end-to-end machine learning system. Using only SQL, it allows to train models and run online predictions, alongside normal queries, directly using the data in your databases.

## Why

Deploying machine learning models into existing applications is not straight forward. Unless you're already using Python in your day to day work, you need to learn a new language and toolchain, figure out how to EL(T) your data from your database(s) into a warehouse or object storage, learn how to train models (Scikit-Learn, Pytorch, Tensorflow, etc.), and finally serve preditions to your apps, forcing your organization into microservices and all the complexity that comes with it.

PostgresML makes ML simple: your data doesn't really go anywhere, you train using simple SQL commands, and you get the predictions to your apps using a mechanism you've been using already: a Postgres connection and a query.

Our goal is that anyone with a basic understanding of SQL should be able to build and deploy machine learning models to production, while receiving the benefits of a high performance machine learning platform. Ultimately, PostgresML aims to be the easiest, safest and fastest way to gain value from machine learning.

## Quick start

Using Docker, boot up PostresML locally:

```bash
$ docker-compose up
```

The system is available on port 5433 by default, just in case you happen to be running Postgres already:

```bash
$ psql -U root -h 127.0.0.1 -p 5433
```

We've included a couple examples in the `examples/` folder. You can run them directly with: 

```bash
$ psql -U root -h 127.0.0.1 -p 5433 -f <filename>
```

See [installation instructions](#Installation) for installing PostgresML in different supported environments, and for more information.

## Features

### Training models

Given a Postgres table or a view, PostgresML can train a model using some commonly used algorithms. We currently support the following Scikit-Learn regression and classification models:

- `LinearRegression`,
- `LogisticRegression`,
- `SVR`,
- `SVC`,
- `RandomForestRegressor`,
- `RandomForestClassifier`,
- `GradientBoostingRegressor`,
- `GradientBoostingClassifier`.

Training a model is then as simple as:

```sql
SELECT * FROM pgml.train(
     'Human-friendly project name',
     'regression', 
     '<name of the table or view containing the data>',
     '<name of the column containing the y target values>'
);
```

PostgresML will snapshot the data from the table, train multiple models from the above list given the objective (`regression` or `classification`), and automatically choose and deploy the model with the best predictions.

### Making predictions

Once the model is trained, making predictions is as simple as:

```sql
SELECT pgml.predict('Human-friendly project name', ARRAY[...]) AS prediction_score;
```

where `ARRAY[...]` is a list of features for which we want to run a prediction. This list has to be in the same order as the columns in the data table. This score then can be used in normal queries, for example:

```sql
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

Take a look [below](#Working-with-PostgresML) for an example with real data.

### Model and data versioning

As data in your database changes, it is possible to retrain the model again to get better predictions. With PostgresML, it's as simple as running the `pgml.train` command again. If the model scores better, it will be automatically used in predictions; if not, the existing model will be kept and continue to score in your queries. We also snapshot the training data, so models can be retrained deterministically to validate and fix any issues.

## Roadmap

This project is currently a proof of concept. Some important features, which we are currently thinking about or working on, are listed below.

### Production deployment

Most companies that use PostgreSQL in production do so using managed services like AWS RDS, Digital Ocean, Azure, etc. Those services do not allow running custom extensions, so we have to run PostgresML
directly on VMs, e.g. EC2, droplets, etc. The idea here is to replicate production data directly from Postgres and make it available in real-time to PostgresML. We're considering solutions like logical replication for small to mid-size databases, and Debezium for multi-TB deployments.

### Model management dashboard

A good looking and useful UI goes a long way. A dashboard similar to existing solutions like MLFlow or AWS SageMaker will make the experience of working with PostgresML as pleasant as possible.


### Data explorer

A data explorer allows anyone to browse the dataset in production and to find useful tables and features to build effective machine learning models.


### More algorithms

Scikit-Learn is a good start, but we're also thinking about including Tensorflow, Pytorch, and many more useful models.


### Scheduled training

In applications where data changes often, it's useful to retrain the models automatically on a schedule, e.g. every day, every week, etc.


### FAQ

*How far can this scale?*

Petabyte sized Postgres deployements are [documented](https://www.computerworld.com/article/2535825/size-matters--yahoo-claims-2-petabyte-database-is-world-s-biggest--busiest.html) in production since at least 2008, and [recent patches](https://www.2ndquadrant.com/en/blog/postgresql-maximum-table-size/) have enabled working beyond exabyte and up to the yotabyte scale. Machine learning models can be horizontally scaled using standard Postgres replicas.

*How reliable can this be?*

Postgres is widely considered mission critical, and some of the most [reliable](https://www.postgresql.org/docs/current/wal-reliability.html) technology in any modern stack. PostgresML allows an infrastructure organization to leverage pre-existing best practices to deploy machine learning into production with less risk and effort than other systems. For example, model backup and recovery happens automatically alongside normal Postgres data backup.

*How good are the models?*

Model quality is often a tradeoff between compute resources and incremental quality improvements. Sometimes a few thousands training examples and an off the shelf algorithm can deliver significant business value after a few seconds of training. PostgresML allows stakeholders to choose several different algorithms to get the most bang for the buck, or invest in more computationally intensive techniques as necessary. In addition, PostgresML automatically applies best practices for data cleaning like imputing missing values by default and normalizing data to prevent common problems in production. 

PostgresML doesn't help with reformulating a business problem into a machine learning problem. Like most things in life, the ultimate in quality will be a concerted effort of experts working over time. PostgresML is intended to establish successful patterns for those experts to collaborate around while leveraging the expertise of open source and research communities.

*Is PostgresML fast?*

Colocating the compute with the data inside the database removes one of the most common latency bottlenecks in the ML stack, which is the (de)serialization of data between stores and services across the wire. Modern versions of Postgres also support automatic query parrellization across multiple workers to further minimize latency in large batch workloads. Finally, PostgresML will utilize GPU compute if both the algorithm and hardware support it, although it is currently rare in practice for production databases to have GPUs. We're working on [benchmarks](sql/benchmarks.sql).


## Installation

### Docker

The quickest way to try this out is with Docker. If you're on Mac, install [Docker for Mac](https://docs.docker.com/desktop/mac/install/). If you're on Linux (e.g. Ubuntu/Debian), you can follow [these instructions](https://docs.docker.com/engine/install/ubuntu/). For Ubuntu, also install `docker-compose`. Docker and this image also works on Windows/WSL2.

Starting up a local system is then as simple as:

```bash
$ docker-compose up -d
```

PostgresML will run on port 5433, just in case you already have Postgres running. Then to connect, run:

```bash
$ psql -h 127.0.0.1 -p 5433 -U root
```

To validate it works, you can execute this query and you should see this result:

```sql
SELECT pgml.version();

 version
---------
 0.1
(1 row)
```

### Mac OS (native)

If you want want to use Docker, a native installation is available. We recommend you use [Postgres.app](https://postgresapp.com/) because it comes with PL/Python, the extension we rely on, built into the installation. Once you have Postgres.app running, you'll need to install the Python framework. Mac OS has multiple distributions of Python, namely one from Brew and one from the Python community (Python.org);
Postgres.app and PL/Python depend on the community one. The following versions of Python and Postgres.app are compatible:

| **PostgreSQL version** | **Python version** | **Download link**                                                                       |
|------------------------|--------------------|-----------------------------------------------------------------------------------------|
| 14                     | 3.9                | [Python 3.9 64-bit](https://www.python.org/ftp/python/3.9.12/python-3.9.12-macos11.pkg) |
| 13                     | 3.8                | [Python 3.8 64-bit](https://www.python.org/ftp/python/3.8.10/python-3.8.10-macos11.pkg) |

All Python.org installers for Mac OS are [available here](https://www.python.org/downloads/macos/). You can also get more details about this in the Postgres.app [documentation](https://postgresapp.com/documentation/plpython.html).

#### Python package

To use our Python package inside Postgres, we need to install it into the global Python package space. Depending on which version of Python you installed in the previous step,
use its correspoding pip executable. Since Python was installed as a framework, sudo (root) is not required.

For PostgreSQL 14, use Python & Pip 3.9:

```bash
$ pip3.9 install pgml
```

#### PL/Python functions

Finally to interact with the package, install our functions and supporting tables into the database:

```bash
$ psql -f sql/install.sql
```

If everything works, you should be able to run this successfully:

```bash
$ psql -c 'SELECT pgml.version()'
```

### Ubuntu/Debian

Each Ubuntu/Debian distribution comes with its own version of PostgreSQL, the simplest way is to install it from Aptitude:

```bash
$ sudo apt-get install -y postgresql-plpython3-12 python3 python3-pip postgresql-12
```

Restart PostgreSQL:

```bash
$ sudo service postgresql restart
```

Install our Python package and SQL functions:

```bash
$ sudo pip3 install pgml
$ psql -f sql/install.sql
```

If everything works correctly, you should be able to run this successfully:

```bash
$ psql -c 'SELECT pgml.version()'
```

## Working with PostgresML

The two most important functions the framework provides are:

1. `pgml.train(project_name TEXT, objective TEXT, relation_name TEXT, y_column_name TEXT, algorithm TEXT DEFAULT NULL)`,
2. `pgml.predict(project_name TEXT, VARIADIC features DOUBLE PRECISION[])`.

The first function trains a model, given a human-friendly project name, a `regression` or `classification` objective, a table or view name which contains the training and testing datasets, and the name of the `y` column containing the target values. The second function predicts novel datapoints, given the project name for an exiting model trained with `pgml.train`, and a list of features used to train that model.

You can also browse complete [code examples in the repository](examples/).

### Walkthrough

We'll be using the [Red Wine Quality](https://www.kaggle.com/datasets/uciml/red-wine-quality-cortez-et-al-2009) dataset from Kaggle for this example. You can find it in the `data` folder in this repository. You can import it into PostgresML running in Docker with this:

```bash
$ psql -f data/winequality-red.sql -p 5433 -U root -h 127.0.0.1
```

### Training a model

Training a model is as easy as creating a table or a view that holds the training data, and then registering that with PostgresML:

```sql
SELECT * FROM pgml.train('Red Wine Quality', 'regression', 'wine_quality_red', 'quality');

    project_name  | objective  |  status
---------------------+------------+--------
 Red Wine Quality | regression | deployed
```

The function will snapshot the training data, train the model using multiple algorithms, automatically pick the best one, and make it available for predictions.

### Predictions

Predicting novel datapoints is as simple as:

```sql
SELECT pgml.predict('Red Wine Quality', 7.4, 0.66, 1.0, 1.8, 0.075, 17.0, 40.0, 0.9978, 3.58, 0.56, 9.4) AS quality;

 quality 
---------
    4.19
(1 row)
```

PostgresML similarly supports classification to predict discrete classes rather than numeric scores for novel data.

```sql
SELECT pgml.train('Handwritten Digit Classifier', 'classification', pgml.mnist_training_data, label_column_name);
```

And predict novel datapoints:

```sql
SELECT pgml.predict('Handwritten Digit Classifier', pgml.mnist_test_data.*)
FROM pgml.mnist
LIMIT 1;

 digit | likelihood
-------+----
 5     | 0.956432
(1 row)
```

### Contributing

Follow the installation instructions to create a local working Postgres environment, then install your PgML package from the git repository:

```
cd pgml
sudo python3 setup.py install
cd ../
```

Run the tests from the root of the repo:

```
psql -f sql/test.sql
```

One liner:
```
cd pgml; sudo python3 setup.py install; cd ../; sudo -u postgres psql -f sql/test.sql
```

Make sure to run it exactly like this, from the root directory of the repo.
