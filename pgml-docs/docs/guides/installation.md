---
title: "Postgres<span style='color: dodgerblue'>ML</span>"
---

# Installation

## with Docker <small>Recommended</small>
=== ":material-linux: Linux"

    [Install Docker for Linux](https://docs.docker.com/engine/install/ubuntu/). Some package managers (e.g. Ubuntu/Debian) additionally require the `docker-compose` package to be installed seperately.

=== ":material-apple: OS X"

    [Install Docker for OS X](https://docs.docker.com/desktop/mac/install/) 

=== ":material-microsoft-windows: Windows"

    [Install Docker for Windows](https://docs.docker.com/desktop/windows/install/). Use the Linux instructions if you're installing in Windows Subsystem for Linux.

1) Clone the repo:
```bash
$ git clone git@github.com:postgresml/postgresml.git
```

2) Start dockerized services. PostgresML will run on port 5433, just in case you already have Postgres running:
```bash
$ cd postgresml && docker-compose up
```

3) Connect to Postgres in the container with PostgresML installed:
```bash
$ psql postgres://postgres@localhost:5433/pgml_development
```

4) Validate your installation:
```sql
pgml_development=# SELECT pgml.version();
 version
---------
 0.8.1
(1 row)
```

Docker Compose will also start the admin app running locally [http://localhost:8000/](http://localhost:8000/)


## Native Installation & Production Deployments

A PostgresML deployment consists of two different runtimes. The foundational runtime is a Python extension for Postgres ([pgml-extension](./pgml-extension/)) that facilitates the machine learning lifecycle inside the database. Additionally, we provide a dashboard ([pgml-admin](./pgml-admin/)) that can connect to your Postgres server and provide additional management functionality. It will also provide visibility into the models you build and data they use. 

### Mac OS

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