# PostgresML Dashboard

PostgresML provides a dashboard with analytical views of the training data and model performance, as well as integrated notebooks for rapid iteration. It is primarily written in Rust using [Rocket](https://rocket.rs/) as a lightweight web framework and [SQLx](https://github.com/launchbadge/sqlx) to interact with the database.

Please see the [online documentation](https://postgresml.org/user_guides/setup/quick_start_with_docker/) for general information on installing or deploying PostgresML. This document is intended to help developers set up a local copy of the dashboard. 

## Requirements

The dashboard requires a Postgres database with the [pgml-extension](https://github.com/postgresml/postgresml/tree/master/pgml-extension) to generate the core schema. See that subproject for developer setup.

We develop and test this web application on Linux, OS X, and Windows using WSL2.

## Build process

You'll need to specify a database url for the extension to interact with via an environment variable:

```commandline
export DATABASE_URL=postgres://user_name:password@localhost:5432/database_name
```

Build and run:

```commandline
cargo run
```

Incremental and automatic compilation for development cycles is supported with: 

```commandline
cargo watch --exec run
```

Run tests:
```commandline
cargo test
```
