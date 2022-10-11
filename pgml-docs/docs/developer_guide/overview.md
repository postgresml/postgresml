# Contributing

Thank you for your interest in contributing to PostgresML! We are an open source, MIT licensed project, and we welcome all contributions, including bug fixes, features, documentation, typo fixes, and Github stars.

Our project consists of three (3) applications:

1. Postgres extension (`pgml-extension`)
2. Dashboard web app (`pgml-dashboard`)
3. Documentation (`pgml-docs`)

The development environment for each differs slightly, but overall we use Python, Rust, and PostgreSQL, so as long as you have all of those installed, the setup should be straight forward.


## Postgres extension

PostgresML is a Rust extension written with `tcdi/pgx` crate. Local development therefore requires the [latest Rust compiler](https://www.rust-lang.org/learn/get-started) and PostgreSQL development headers and libraries.

The extension code is located in:

```bash
cd pgml-extension/
```

Once there, you can initialize `pgx` and get going:

```bash
cargo install cargo-pgx --version "0.4.5"
cargo pgx init # This will take a few minutes
```

`pgx` uses Postgres 13 by default. Since `pgml` is using shared memory, you need to add it to `shared_preload_libraries` in `postgresql.conf` which, for `pgx`, is located in `~/.pgx/data-13/postgresql.conf`.

```
shared_preload_libraries = 'pgml'
```

and you're ready to go:

```bash
cargo pgx run
```


If you ever want to reset the environment, simply spin up the database with `cargo pgx run` and drop the extension and metadata tables:

```postgresql
DROP EXTENSION pgml CASCADE;
DROP SCHEMA pgml CASCADE;
CREATE EXTENSION pgml;
```

## Dashboard app

The Dashboard is a Django application, and requires no special setup apart for what's required for a normal Django project.

```
cd pgml-dashboard/
```

Once there, you can setup a virtual environment and get going:

```bash
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
cp .env.TEMPLATE .env
python manage.py migrate
python manage.py runserver
```

The dashboard expects to have a PostgreSQL database with the `pgml` extension installed into the `pgml_development` database. You can install it by following our [Installation](/user_guides/setup/v2/installation/) instructions or by pointing the Django app to the database started by `cargo pgx run`.

## Documentation app

The documentation app (you're using it right now) is using MkDocs.

```
cd pgml-docs/
```

Once there, you can setup a virtual environment and get going:

```bash
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
python -m mkdocs serve
```

## General

We are a cross-platform team, some of us use WSL and some use Linux or Mac OS. Keeping that in mind, it's good to use common line endings for all files to avoid production errors, e.g. broken Bash scripts.

The project is presently using [Unix line endings](https://docs.github.com/en/get-started/getting-started-with-git/configuring-git-to-handle-line-endings).
