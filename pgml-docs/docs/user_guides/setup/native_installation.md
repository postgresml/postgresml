# Native Installation

A PostgresML deployment consists of two different runtimes. The foundational runtime is a Python extension for Postgres ([pgml-extension](https://github.com/postgresml/postgresml/tree/master/pgml-extension/)) that facilitates the machine learning lifecycle inside the database. Additionally, we provide a dashboard ([pgml-dashboard](https://github.com/postgresml/postgresml/tree/master/pgml-dashboard/)) that can connect to your Postgres server and provide additional management functionality. It will also provide visibility into the models you build and data they use. 

## Install PostgreSQL with PL/Python

PostgresML leverages Python libraries for their machine learning capabilities. You'll need to make sure the PostgreSQL installation has PL/Python built in.

=== ":material-apple: OS X"

    We recommend you use [Postgres.app](https://postgresapp.com/) because it comes with [PL/Python](https://www.postgresql.org/docs/current/plpython.html). Otherwise, you'll need to install PL/Python manually. Once you have Postgres.app running, you'll need to install the Python framework. Mac OS has multiple distributions of Python, namely one from Brew and one from the Python community (Python.org); Postgres.app and PL/Python depend on the community one. The following versions of Python and Postgres.app are compatible:

    | **PostgreSQL version** | **Python version** | **Download link**                                                                       |
    |------------------------|--------------------|-----------------------------------------------------------------------------------------|
    | 14                     | 3.9                | [Python 3.9 64-bit](https://www.python.org/ftp/python/3.9.12/python-3.9.12-macos11.pkg) |
    | 13                     | 3.8                | [Python 3.8 64-bit](https://www.python.org/ftp/python/3.8.10/python-3.8.10-macos11.pkg) |

    All Python.org installers for Mac OS are [available here](https://www.python.org/downloads/macos/). You can also get more details about this in the Postgres.app [documentation](https://postgresapp.com/documentation/plpython.html).

=== ":material-linux: Linux"

    Each Ubuntu/Debian distribution comes with its own version of PostgreSQL, the simplest way is to install it from Aptitude:

    ```bash
    $ sudo apt-get install -y postgresql-plpython3-12 python3 python3-pip postgresql-12
    ```

=== ":material-microsoft-windows: Windows"

    Enterprise db provides Windows builds of PostgreSQL [available for download](https://www.enterprisedb.com/downloads/postgres-postgresql-downloads).
    

## Install the extension

To use our Python package inside PostgreSQL, we need to install it into the global Python package space. Depending on which version of Python you installed in the previous step, use the corresponding pip executable. 

Change the `--database-url` option to point to your PostgreSQL server.

```bash
$ sudo pip3 install --install-option="--database-url=postgres://user_name:password@localhost:5432/database_name" pgml-extension
```

If everything works, you should be able to run this successfully:

```bash
$ psql -c 'SELECT pgml.version()' postgres://user_name:password@localhost:5432/database_name
```

## Run the dashboard

The PostgresML dashboard is a Django app, that can be run against any PostgreSQL installation. There is an included Dockerfile if you wish to run it as a container, or you may want to setup a Python venv to isolate the dependencies. Basic install can be achieved with:

1. Clone the repo:
```bash
$ git clone git@github.com:postgresml/postgresml.git && cd postgresml/pgml-dashboard
```

2. Set your `PGML_DATABASE_URL` environment variable:
```bash
$ echo PGML_DATABASE_URL=postgres://user_name:password@localhost:5432/database_name > .env
```

3. Install dependencies:
```bash
$ pip install -r requirements.txt
```

4. Run the server:
```bash
$ python manage.py runserver
```
