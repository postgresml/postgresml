# Contributing

## Setup your development environment

1) Install [pyenv](https://github.com/pyenv/pyenv) for your system to use the correct version of python specified in `.python-version`. Make sure your $PATH now includes `~/.pyenv/bin` && `~/.pyenv/shims`.

2) Install [pyenv-virtualenv](https://github.com/pyenv/pyenv-virtualenv) to isolate project dependencies which keeps `requirements.txt` clean and frozen. 

3) Install the version of python listed in `.python-version`:

   $ pyenv install <paste only the full version number, e.g. 3.10.4>

4) Create the virtual env:

   $ pyenv virtualenv <paste only the full version number, e.g. 3.10.4> postgresml

5) Install the dependencies:

    $ pip install -r requirements.txt

6) If you ever add new dependencies, freeze them:

    $ pip freeze > requirements.txt

7) Make sure requirements.txt has no changes, which indicates your virtual environment is setup correctly.

    $ git status

8) Create the development databases in the postgres cluster running inside docker:

    $ psql -c "CREATE DATABASE pgml_development" postgres://postgres:postgres@127.0.0.1:5433/
    $ psql -c "CREATE DATABASE pgml_test" postgres://postgres:postgres@127.0.0.1:5433/
    $ psql -c "CREATE SCHEMA pgml" postgres://postgres:postgres@127.0.0.1:5433/pgml_development
    $ psql -c "CREATE SCHEMA pgml" postgres://postgres:postgres@127.0.0.1:5433/pgml_test

9) setup your .env

    $ cp .env.TEMPLATE .env
    $ nano .env

10) Run db migrations

    $ ./scripts/manage.py migrate

11) Create the superuser

    $ ./scripts/manage.py createsuperuser

12) Run the server

    $ ./scripts/manage.py runserver


Reset your local database:

    $ psql -c "drop DATABASE pgml_development" postgres://postgres:postgres@127.0.0.1:5433/; psql -c "create database pgml_development" postgres://postgres:postgres@127.0.0.1:5433/; psql -c "create schema pgml" postgres://postgres:postgres@127.0.0.1:5433/pgml_development; 


Follow the installation instructions to create a local working Postgres environment, then install your PgML package from the git repository:

```
sudo python3 pgml/setup.py install
```

Run the tests from the root of the repo:

```
ON_ERROR_STOP=1 psql -f sql/test.sql
```

One liner:
```
cd pgml; sudo /usr/bin/python3.8 setup.py install; cd ..; ON_ERROR_STOP=1 psql -f sql/test.sql -p 5432 -h localhost -U postgres -P pager -d pgml_development
```

Make sure to run it exactly like this, from the root directory of the repo.
