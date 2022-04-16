# PostgresML

PostgresML aims to be the easiest way to gain value from machine learning. Anyone with a basic understanding of SQL should be able to build and deploy models to production, while receiving the benefits of a high performance machine learning platform. PostgresML leverages state of the art algorithms with built in best practices, without having to setup additional infrastructure or learn additional programming languages.

## Installation

### Docker

The quickest way to try this out is with Docker. If you're on Mac, install [Docker for Mac](https://docs.docker.com/desktop/mac/install/); if you're on Linux (e.g. Ubuntu/Debian), you can follow [these instructions](https://docs.docker.com/engine/install/ubuntu/). For Ubuntu, also install `docker-compose`.

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

If you don't like Docker, a native installation is definitely possible. We recommend you use [Postgres.app](https://postgresapp.com/) because it comes with PL/Python, an extension we rely on, built into the installation. Once you have Postgres.app running, you'll need to install the Python framework. Mac OS has multiple distributions of Python, namely one from Brew and one from the Python community (python.org).
Postgres.app relies on the community one. The following are compatible versions of Python and PostgreSQL in Postgres.app:

| **PostgreSQL version** | **Python version** | **Download link**                                                                       |
|------------------------|--------------------|-----------------------------------------------------------------------------------------|
| 14                     | 3.9                | [Python 3.9 64-bit](https://www.python.org/ftp/python/3.9.12/python-3.9.12-macos11.pkg) |
| 13                     | 3.8                | [Python 3.8 64-bit](https://www.python.org/ftp/python/3.8.10/python-3.8.10-macos11.pkg) |

All Python.org installers for Mac OS are [available here](https://www.python.org/downloads/macos/).

#### Python package

To use our Python package inside Postgres, we need to install it into the global Python package space. Depending on which version of Python you installed in the previous step,
use its correspoding pip executable. Since Python was installed as a framework, sudo (root) is not required.

```bash
$ pip3.9 install pgml
```

#### PL/Python functions

Finally to interact with the package, install our functions and supporting tables into PostgreSQL:

```bash
psql -f sql/install.sql
```

If everything works, you should be able to run this successfully:

```bash
psql -c 'SELECT pgml.version()'
```

### Ubuntu/Debian

Each Ubuntu distribution comes with its own version of PostgreSQL, the simplest way is to install it from Aptitude:

```bash
$ sudo apt-get install -y postgresql-plpython3-12 python3 python3-pip postgresql-12
```

Restart PostgreSQL:

```bash
sudo service postgresql restart
```

Install our Python package and SQL functions:

```bash
$ sudo pip3 install pgml
$ psql -f sql/install.sql
```

If everything works, you should be able to run this successfully:

```bash
psql -c 'SELECT pgml.version()'
```

## Working with PostgresML


Getting started is as easy as creating a `table` or `view` that holds the training data, and then registering that with PostgresML. 

```sql
SELECT pgml.model_regression('Red Wine Quality', training_data_table_or_view_name, label_column_name);
```

And predict novel datapoints:

```sql
SELECT pgml.predict('Red Wine Quality', red_wines.*)
FROM pgml.red_wines
LIMIT 3;

 quality 
---------
 0.896432
 0.834822
 0.954502
(3 rows)
```

PostgresML similarly supports classification to predict discrete classes rather than numeric scores for novel data.

```sql
SELECT pgml.create_classification('Handwritten Digit Classifier', pgml.mnist_training_data, label_column_name);
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

Checkout the [documentation](https://TODO) to view the full capabilities, including:
- [Creating Training Sets](https://TODO)
    - [Classification](https://TODO)
    - [Regression](https://TODO)
- [Supported Algorithms](https://TODO)
    - [Scikit Learn](https://TODO)
    - [XGBoost](https://TODO)
    - [Tensorflow](https://TODO)
    - [PyTorch](https://TODO)

### Planned features
- Model management dashboard
- Data explorer
- More algorithms and libraries incluiding custom algorithm support


### FAQ

*How well does this scale?*

Petabyte sized Postgres deployements are [documented](https://www.computerworld.com/article/2535825/size-matters--yahoo-claims-2-petabyte-database-is-world-s-biggest--busiest.html) in production since at least 2008, and [recent patches](https://www.2ndquadrant.com/en/blog/postgresql-maximum-table-size/) have enabled working beyond exabyte up to the yotabyte scale. Machine learning models can be horizontally scaled using well tested Postgres replication techniques on top of a mature storage and compute platform.

*How reliable is this system?*

Postgres is widely considered mission critical, and some of the most [reliable](https://www.postgresql.org/docs/current/wal-reliability.html) technology in any modern stack. PostgresML allows an infrastructure organization to leverage pre-existing best practices to deploy machine learning into production with less risk and effort than other systems. For example, model backup and recovery happens automatically alongside normal data backup procedures.

*How good are the models?*

Model quality is often a tradeoff between compute resources and incremental quality improvements. PostgresML allows stakeholders to choose algorithms from several libraries that will provide the most bang for the buck. In addition, PostgresML automatically applies best practices for data cleaning like imputing missing values by default and normalizing data to prevent common problems in production. After quickly enabling 0 to 1 value creation, PostgresML enables further expert iteration with custom data preperation and algorithm implementations. Like most things in life, the ultimate in quality will be a concerted effort of experts working over time, but that shouldn't get in the way of a quick start.

*Is PostgresML fast?*

Colocating the compute with the data inside the database removes one of the most common latency bottlenecks in the ML stack, which is the (de)serialization of data between stores and services across the wire. Modern versions of Postgres also support automatic query parrellization across multiple workers to further minimize latency in large batch workloads. Finally, PostgresML will utilize GPU compute if both the algorithm and hardware support it, although it is currently rare in practice for production databases to have GPUs. Checkout our [benchmarks](https://todo).



### Installation in WSL or Ubuntu

Install Python3, pip, and Pl/Python3:

```bash
sudo apt update
sudo apt install -y postgresql-plpython3-12 python3 python3-pip
```

Restart the Postgres server:

```bash
sudo service postgresql restart
```

Create the extension:

```sql
CREATE EXTENSION plpython3u;
```

Install Scikit globally (I didn't bother setup Postgres with a virtualenv, but it's possible):

```
sudo pip3 install sklearn
```

### Run the example

```bash
psql -f scikit_train_and_predict.sql
```

Example output:

```
psql:scikit_train_and_predict.sql:4: NOTICE:  drop cascades to view scikit_train_view
DROP TABLE
CREATE TABLE
psql:scikit_train_and_predict.sql:14: NOTICE:  view "scikit_train_view" does not exist, skipping
DROP VIEW
CREATE VIEW
INSERT 0 500
CREATE FUNCTION
 scikit_learn_train_example
----------------------------
 OK
(1 row)

CREATE FUNCTION
 value | weight | prediction
-------+--------+------------
     1 |      5 |          5
     2 |      5 |          5
     3 |      5 |          5
     4 |      5 |          5
     5 |      5 |          5
(5 rows)
```

### Run the linear model

Install our PgML package globally:

```
cd pgml
sudo python3 setup.py install
cd ../
```

Run the test:

```
psql -f sql/test.sql
```

Make sure to run it exactly like this, from the root directory of the repo.
