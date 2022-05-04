# PostgresML

![PostgresML](./pgml-admin/app/static/images/logo-small.png)

[![CircleCI](https://circleci.com/gh/postgresml/postgresml/tree/master.svg?style=shield)](https://circleci.com/gh/postgresml/postgresml/tree/master)

PostgresML is an end-to-end machine learning system. It enables you to train models and make online predictions using only SQL, without your data ever leaving your favorite database.

## Motivation

Deploying machine learning models into existing applications is not straight forward. It involves operating new services, which need to be written in specialized languages with libraries outside of the experience of many software engineers. Those services tend to be architected around specialized datastores and hardware that requires additional management and know how. Data access needs to be secure across production and development environments without impeding productivity. This complexity pushes risks and costs beyond acceptable trade off limits for many otherwise valuable use cases.

PostgresML makes ML simple by moving the code to your data, rather than copying the data all over the place. You train models using simple SQL commands, and you get the predictions in your apps via a mechanism you're already using: a query over a standard Postgres connection.

Our goal is that anyone with a basic understanding of SQL should be able to build, deploy and maintain machine learning models in production, while receiving the benefits of a high performance machine learning platform. Ultimately, PostgresML aims to be the easiest, safest and fastest way to gain value from machine learning.

## Quick start

Using Docker, boot up PostresML locally:

```bash
$ docker-compose up
```

The system runs Postgres with the pgml-extension installed on port 5433 by default, just in case you happen to be running Postgres already:

```bash
$ psql -U postgres -h 127.0.0.1 -p 5433 -d pgml_development
```

We've included [examples](./pgml-extension/examples) to demonstrate the core functionality. You can run them directly like so: 

```bash
$ psql -U postgres -h 127.0.0.1 -p 5433 -d pgml_development -f pgml-extension/examples/classification.sql
```

The admin web UI is available on http://localhost:8000. After you run the classification example, you should see it in there:

![Admin UI](./pgml-admin/screenshots/snapshot.png)


See [installation instructions](#Installation) for installing PostgresML in different supported environments, and for more information.

## Features
### Vector Operations
PostgresML adds native vector operations that can be called from SQL:

```SQL
-- Elementwise arithmetic w/ constants
pgml.add(a REAL[], b REAL) -> REAL[]
pgml.subtract(minuend REAL[], subtrahend REAL) -> REAL[]
pgml.multiply(multiplicand REAL[], multiplier REAL) -> REAL[]
pgml.divide(dividend REAL[], divisor REAL) -> REAL[]

-- Pairwise arithmetic w/ vectors
pgml.add(a REAL[], b REAL[]) -> REAL[]
pgml.subtract(minuend REAL[], subtrahend REAL[]) -> REAL[]
pgml.multiply(multiplicand REAL[], multiplier REAL[]) -> REAL[]
pgml.divide(dividend REAL[], divisor REAL[]) -> REAL[]

-- Norms
pgml.norm_l0(vector REAL[]) -> REAL -- Dimensions not at the origin
pgml.norm_l1(vector REAL[]) -> REAL -- Manhattan distance from origin
pgml.norm_l2(vector REAL[]) -> REAL -- Euclidean distance from origin

-- Normalization
pgml.normalize_max(vector REAL[]) -> REAL[] -- -1:1 values
pgml.normalize_l1(vector REAL[]) -> REAL[] -- Unit Vector
pgml.normalize_l2(vector REAL[]) -> REAL[] -- Squared Unit Vector

-- Distances
pgml.distance_l1(a REAL[], b REAL[]) -> REAL -- Manhattan
pgml.distance_l2(a REAL[], b REAL[]) -> REAL -- Euclidean
pgml.dot_product(a REAL[], b REAL[]) -> REAL -- Projection
pgml.cosine_similarity(a REAL[], b REAL[]) -> REAL -- Direction
```

### Training models

Given a Postgres table or a view, PostgresML can train a model with many commonly used algorithms. We currently support the following regression and classification models from Scikit-Learn and XGBoost:


#### XGBoost
Algorithm | Regression | Classification
--- | --- | ---
`xgboost` | [XGBRegressor](https://xgboost.readthedocs.io/en/stable/python/python_api.html#xgboost.XGBRegressor) | [XGBClassifier](https://xgboost.readthedocs.io/en/stable/python/python_api.html#xgboost.XGBClassifier)
`xgboost_random_forest` | [XGBRFRegressor](https://xgboost.readthedocs.io/en/stable/python/python_api.html#xgboost.XGBRFRegressor) | [XGBRFClassifier](https://xgboost.readthedocs.io/en/stable/python/python_api.html#xgboost.XGBRFClassifier)

#### Scikit Ensembles
Algorithm | Regression | Classification
--- | --- | ---
`ada_boost` | [AdaBoostRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.AdaBoostRegressor.html) | [AdaBoostClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.AdaBoostClassifier.html)
`bagging` | [BaggingRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.BaggingRegressor.html) | [BaggingClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.BaggingClassifier.html)
`extra_trees` | [ExtraTreesRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.ExtraTreesRegressor.html) | [ExtraTreesClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.ExtraTreesClassifier.html)
`gradient_boosting_trees` | [GradientBoostingRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.GradientBoostingRegressor.html) | [GradientBoostingClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.GradientBoostingClassifier.html)
`random_forest` | [RandomForestRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.RandomForestRegressor.html) | [RandomForestClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.RandomForestClassifier.html)
`hist_gradient_boosting` | [HistGradientBoostingRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.HistGradientBoostingRegressor.html) | [HistGradientBoostingClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.ensemble.HistGradientBoostingClassifier.html)

#### Support Vector Machines
Algorithm | Regression | Classification
--- | --- | ---
`svm` | [SVR](https://scikit-learn.org/stable/modules/generated/sklearn.svm.SVR.html) | [SVC](https://scikit-learn.org/stable/modules/generated/sklearn.svm.SVC.html)
`nu_svm` | [NuSVR](https://scikit-learn.org/stable/modules/generated/sklearn.svm.NuSVR.html) | [NuSVC](https://scikit-learn.org/stable/modules/generated/sklearn.svm.NuSVC.html)
`linear_svm` | [LinearSVR](https://scikit-learn.org/stable/modules/generated/sklearn.svm.LinearSVR.html) | [LinearSVC](https://scikit-learn.org/stable/modules/generated/sklearn.svm.LinearSVC.html)

#### Linear Models
Algorithm | Regression | Classification
--- | --- | ---
`linear` | [LinearRegression](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.LinearRegression.html) | [LogisticRegression](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.LogisticRegression.html)
`ridge` |  [Ridge](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.Ridge.html) | [RidgeClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.RidgeClassifier.html)
`lasso` |  [Lasso](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.Lasso.html) | -
`elastic_net` | [ElasticNet](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.ElasticNet.html) | -
`least_angle` | [LARS](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.Lars.html) | -
`lasso_least_angle` | [LassoLars](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.LassoLars.html) | -
`orthoganl_matching_pursuit` | [OrthogonalMatchingPursuit](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.OrthogonalMatchingPursuit.html) | -
`bayesian_ridge` | [BayesianRidge](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.BayesianRidge.html) | -
`automatic_relevance_determination` | [ARDRegression](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.ARDRegression.html) | -
`stochastic_gradient_descent` | [SGDRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.SGDRegressor.html) | [SGDClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.SGDClassifier.html) 
`perceptron` | - | [Perceptron](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.Perceptron.html) 
`passive_aggressive` | [PassiveAggressiveRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.PassiveAggressiveRegressor.html) | [PassiveAggressiveClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.PassiveAggressiveClassifier.html) 
`ransac` | [RANSACRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.RANSACRegressor.html) | -
`theil_sen` | [TheilSenRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.TheilSenRegressor.html) | -
`huber` | [HuberRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.HuberRegressor.html) | -
`quantile` | [QuantileRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.QuantileRegressor.html) | -

#### Other
Algorithm | Regression | Classification
--- | --- | ---
`kernel_ridge` | [KernelRidge](https://scikit-learn.org/stable/modules/generated/sklearn.kernel_ridge.KernelRidge.html) | -
`gaussian_process` | [GaussianProcessRegressor](https://scikit-learn.org/stable/modules/generated/sklearn.gaussian_process.GaussianProcessRegressor.html) | [GaussianProcessClassifier](https://scikit-learn.org/stable/modules/generated/sklearn.gaussian_process.GaussianProcessClassifier.html)


Training a model is then as simple as:

```sql
SELECT * FROM pgml.train(
    'Human-friendly project name',
    'regression', 
    '<name of the table or view containing the data>',
    '<name of the column containing the y target values>',
    '<algorithm name>', -- optional 
    '<algorithm hyperparams>' -- optional
);
```

PostgresML will snapshot the data from the table, train the model with the algorithm, and automatically deploy model improvements as measured by key performance metrics to make predictions in production.

### Making predictions

Once the model is trained, making predictions is as simple as:

```sql
SELECT pgml.predict('Human-friendly project name', ARRAY[...]) AS prediction_score;
```

where `ARRAY[...]` is the same list of features for a sample used in training. This score then can be used in normal queries, for example:

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

As data in your database changes, it is possible to retrain the model again to get better predictions. With PostgresML, it's as simple as running the `pgml.train` command again. If the model scores better, it will be automatically used in predictions; if not, the existing model will be kept and continue to score in your queries. There is also a deployment API if you need to manually manage which model is active. We also snapshot the training data, so models can be retrained deterministically to validate and fix any issues.

#### Deployments

Models are automatically deployed if their key metric (`mean_squared_error` for regression, `f1` for classification) is improved over the currently deployed version during training. If you want to manage deploys manually, you can always change which model is currently responsible for making predictions with:

```sql
SELECT pgml.deploy(project_name TEXT, strategy TEXT DEFAULT 'best_score', algorithm_name TEXT DEFAULT NULL)
```

The default behavior allows any algorithm to qualify, but deployment candidates can be further restricted to a specific algorithm.

strategy | description
--- | ---
most_recent | The most recently trained model for this project
best_score | The model that achieved the best key metric score
rollback | The model that was previously deployed for this project


## Roadmap

This project is currently a proof of concept. Some important features, which we are currently thinking about or working on, are listed below.

### Production deployment

Most companies that use PostgreSQL in production do so using managed services like AWS RDS, Digital Ocean, Azure, etc. Those services do not allow running custom extensions, so we have to run PostgresML directly on VMs, e.g. EC2, droplets, etc. The idea here is to replicate production data directly from Postgres and make it available in real-time to PostgresML. We're considering solutions like logical replication for small to mid-size databases, and Debezium for multi-TB deployments.

### Model management dashboard

A good looking and useful UI goes a long way. A dashboard similar to existing solutions like MLFlow or AWS SageMaker will make the experience of working with PostgresML as pleasant as possible.


### Data explorer

A data explorer allows anyone to browse the dataset in production and to find useful tables and features to build effective machine learning models.


### More algorithms

Scikit-Learn is a good start, but we're also thinking about including Tensorflow, Pytorch, and many more useful models.


### Scheduled training

In applications where data changes often, it's useful to retrain the models automatically on a schedule, e.g. every day, every week, etc.


## FAQ

*How far can this scale?*

Petabyte-sized Postgres deployments are [documented](https://www.computerworld.com/article/2535825/size-matters--yahoo-claims-2-petabyte-database-is-world-s-biggest--busiest.html) in production since at least 2008, and [recent patches](https://www.2ndquadrant.com/en/blog/postgresql-maximum-table-size/) have enabled working beyond exabyte and up to the yotabyte scale. Machine learning models can be horizontally scaled using standard Postgres replicas.

*How reliable can this be?*

Postgres is widely considered mission critical, and some of the most [reliable](https://www.postgresql.org/docs/current/wal-reliability.html) technology in any modern stack. PostgresML allows an infrastructure organization to leverage pre-existing best practices to deploy machine learning into production with less risk and effort than other systems. For example, model backup and recovery happens automatically alongside normal Postgres data backup.

*How good are the models?*

Model quality is often a tradeoff between compute resources and incremental quality improvements. Sometimes a few thousands training examples and an off the shelf algorithm can deliver significant business value after a few seconds of training. PostgresML allows stakeholders to choose several different algorithms to get the most bang for the buck, or invest in more computationally intensive techniques as necessary. In addition, PostgresML automatically applies best practices for data cleaning like imputing missing values by default and normalizing data to prevent common problems in production. 

PostgresML doesn't help with reformulating a business problem into a machine learning problem. Like most things in life, the ultimate in quality will be a concerted effort of experts working over time. PostgresML is intended to establish successful patterns for those experts to collaborate around while leveraging the expertise of open source and research communities.

*Is PostgresML fast?*

Colocating the compute with the data inside the database removes one of the most common latency bottlenecks in the ML stack, which is the (de)serialization of data between stores and services across the wire. Modern versions of Postgres also support automatic query parrellization across multiple workers to further minimize latency in large batch workloads. Finally, PostgresML will utilize GPU compute if both the algorithm and hardware support it, although it is currently rare in practice for production databases to have GPUs. We're working on [benchmarks](./pgml-extension/sql/benchmarks.sql).


## Installation

### Running with Docker

The quickest way to try this out is with Docker, which is available for common operating systems: 
- [Docker Installation on OS X](https://docs.docker.com/desktop/mac/install/) or [Docker Installation on Linux](https://docs.docker.com/engine/install/ubuntu/). If you're on Linux (e.g. Ubuntu/Debian), you'll also need to install the `docker-compose` package. These Docker images also work on Windows/WSL2.

Starting up a local system is then as simple as:

```bash
$ docker-compose up -d
```

PostgresML will run on port 5433, just in case you already have Postgres running. Then to connect, run:

```bash
$ psql postgres://postgres@localhost:5433/pgml_development
```

To validate it works, you can execute this query and you should see this result:

```sql
SELECT pgml.version();

 version
---------
 0.5
(1 row)
```

Docker Compose will also start the admin app running locally on port 8000 [http://localhost:8000/](http://localhost:8000/).


### Native Installation & Production Deployments

A PostgresML deployment consists of two different runtimes. The foundational runtime is a Python extension for Postgres ([pgml-extension](./pgml-extension/)) that facilitates the machine learning lifecycle inside the database. Additionally, we provide an admin app ([pgml-admin](./pgml-admin/)) that can connect to your Postgres server and provide additional management functionality. It will also provide visibility into the models you build and data they use.

#### Mac OS (native)

We recommend you use [Postgres.app](https://postgresapp.com/) because it comes with [PL/Python](https://www.postgresql.org/docs/current/plpython.html), the extension we rely on, built into the installation. Otherwise, you'll need to install PL/Python. Once you have Postgres.app running, you'll need to install the Python framework. Mac OS has multiple distributions of Python, namely one from Brew and one from the Python community (Python.org); Postgres.app and PL/Python depend on the community one. The following versions of Python and Postgres.app are compatible:

| **PostgreSQL version** | **Python version** | **Download link**                                                                       |
|------------------------|--------------------|-----------------------------------------------------------------------------------------|
| 14                     | 3.9                | [Python 3.9 64-bit](https://www.python.org/ftp/python/3.9.12/python-3.9.12-macos11.pkg) |
| 13                     | 3.8                | [Python 3.8 64-bit](https://www.python.org/ftp/python/3.8.10/python-3.8.10-macos11.pkg) |

All Python.org installers for Mac OS are [available here](https://www.python.org/downloads/macos/). You can also get more details about this in the Postgres.app [documentation](https://postgresapp.com/documentation/plpython.html).

##### Python package

To use our Python package inside Postgres, we need to install it into the global Python package space. Depending on which version of Python you installed in the previous step, use the correspoding pip executable. Since Python was installed as a framework, sudo (root) is not required.

For PostgreSQL 14, use Python & Pip 3.9:

```bash
$ pip3.9 install pgml-extension
```

##### PL/Python functions

Finally to interact with the package, install our functions and supporting tables into the database:

```bash
$ psql -f sql/install.sql
```

If everything works, you should be able to run this successfully:

```bash
$ psql -c 'SELECT pgml.version()'
```

#### Ubuntu/Debian

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
$ sudo pip3 install pgml-extension
$ psql -f sql/install.sql
```

If everything works correctly, you should be able to run this successfully:

```bash
$ psql -c 'SELECT pgml.version()'
```

## Working with PostgresML

The two most important functions the framework provides are:

```sql
pgml.train(
    project_name TEXT, 
    objective TEXT, 
    relation_name TEXT, 
    y_column_name TEXT, 
    algorithm TEXT DEFAULT 'linear', 
    hyperparams JSONB DEFAULT '{}'::JSONB
)
```
and 
```sql
pgml.predict(project_name TEXT, features DOUBLE PRECISION[])
```

The first function trains a model, given a human-friendly project name, a `regression` or `classification` objective, a table or view name which contains the training and testing datasets, and the `y_column_name` containing the target values in that table. The second function predicts novel datapoints, given the project name for an exiting model trained with `pgml.train`, and a list of features used to train that model.

You can also browse complete [code examples in the repository](./pgml-extension/examples/).

### Regression Walkthrough

We'll walk through the [regression example](./pgml-extension/examples/regression.sql) first. You'll find that classification is extremely similar. You can test the entire script in PostgresML running in Docker with this:

```bash
$ psql -f examples/regression/run.sql -p 5433 -U root -h 127.0.0.1 -P pager
```

### Loading data

Generally, we'll assume that collecting data is outside the scope of PostgresML, firmly in the scope of Postgres and your business logic. For this example we load a toy dataset into the `pgml.diabetes` schema first:

```sql
SELECT pgml.load_dataset('diabetes');
 load_dataset
--------------
 OK
(1 row)
```

### Training a model

Training a model is as easy as creating a table or a view that holds the training data, and then registering that with PostgresML:

```sql
SELECT * from pgml.train('Diabetes Progression', 'regression', 'pgml.diabetes', 'target');
     project_name     | objective  | algorithm_name |  status
----------------------+------------+----------------+----------
 Diabetes Progression | regression | linear         | deployed
(1 row)
 ```

The function will snapshot the training data, train the model with a default linear regression algorithm, and make it available for predictions.

### Predictions

Predicting novel datapoints is as simple as:

```sql
SELECT pgml.predict('Diabetes Progression', ARRAY[0.038075905,0.05068012,0.061696205,0.021872355,-0.0442235,-0.03482076,-0.043400846,-0.002592262,0.01990842,-0.017646125]) AS progression;

    progression
-------------------
 162.1718930966903
(1 row)
```

You can also make predictions from data stored in a table or view:

```sql
SELECT pgml.predict('Diabetes Progression', ARRAY[age,sex,bmi,bp,s1,s2,s3,s4,s5,s6]) AS progression
FROM pgml.diabetes
LIMIT 10;

    progression
--------------------
 162.17189345230884
 122.84270489051747
 174.37641312463052
  181.1275149413752
   111.739254085156
  71.12693920265463
 134.69178395285311
  184.5315548739402
  208.7589398970435
   161.836547554568
(10 rows)
```

Take a look at the rest of the [regression example](./pgml-extension/examples/regression.sql) to see how to train different algorithms on this dataset. You may also be interested in the [classification example](./pgml-extension/examples/classification.sql) which happens to be extremely similar, although it optimizes for a different key metric.
