# Contributing

## General

- [Use unix line endings](https://docs.github.com/en/get-started/getting-started-with-git/configuring-git-to-handle-line-endings)


## Setup your development environment

1) Install [pyenv](https://github.com/pyenv/pyenv) for your system to use the correct version of python specified in `.python-version`. Make sure your $PATH now includes `~/.pyenv/bin` && `~/.pyenv/shims`.

2) Install [pyenv-virtualenv](https://github.com/pyenv/pyenv-virtualenv) to isolate project dependencies which keeps `requirements.txt` clean and frozen. 

3) Install the version of python listed in `.python-version`:

   $ pyenv install 3.10.4

4) Create the virtual env:

   $ pyenv virtualenv 3.10.4 pgml-admin

5) Install the dependencies:

    $ pip install -r requirements.txt

6) If you ever add new dependencies, freeze them:

    $ pip freeze > requirements.txt

7) Make sure requirements.txt has no changes, which indicates your virtual environment is setup correctly.

    $ git status

8) setup your .env

    $ cp .env.TEMPLATE .env
    $ nano .env

9) Run the server

    $ ./manage.py runserver

## Maintain your development database 
How to reset your local database:

    $ psql -c "DROP DATABASE pgml_development" postgres://postgres@127.0.0.1:5433/; psql -c "CREATE DATABASE pgml_development" postgres://postgres@127.0.0.1:5433/; psql -c "create schema pgml" postgres://postgres@127.0.0.1:5433/pgml_development


Follow the installation instructions to create a local working Postgres environment, then install the pgml-extension from the git repository:

```
cd pgml-extension
sudo python3 setup.py install
```

Run the tests from the root of the repo:

```
cd pgml-extension
ON_ERROR_STOP=1 psql -f sql/test.sql postgres://postgres@127.0.0.1:5433/pgml_development
```

One liner:
```
cd pgml-extension; sudo /bin/pip3 install .; psql -c "DROP DATABASE pgml_development" postgres://postgres@127.0.0.1:5433/; psql -c "CREATE DATABASE pgml_development" postgres://postgres@127.0.0.1:5433/; psql -c "create schema pgml" postgres://postgres@127.0.0.1:5433/pgml_development; ON_ERROR_STOP=1 psql -f sql/test.sql -P pager postgres://postgres@127.0.0.1:5433/pgml_development; cd ..
```

Make sure to run it exactly like this, from the root directory of the repo.

## Update documentation

* `mkdocs serve` - Start the live-reloading docs server.
* `mkdocs build` - Build the documentation site.
* `mkdocs -h` - Print help message and exit.
