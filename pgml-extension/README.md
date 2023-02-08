# PostgresML Extension

PostgresML is a PostgreSQL extension providing end-to-end machine learning inside your database. The extension is primarily written in Rust using [pgx](https://github.com/tcdi/pgx) and provides a SQL interface to various machine learning algorithm implementations such as [XGBoost](https://github.com/dmlc/xgboost), [LightGBM](https://github.com/microsoft/LightGBM), and [other classical methods](https://github.com/rust-ml/linfa).

See [our blog](https://postgresml.org/blog/postgresml-is-moving-to-rust-for-our-2.0-release/) for a performance comparison and further motivations.

Please see the [quick start instructions](https://postgresml.org/user_guides/setup/quick_start_with_docker/) for general information on installing or deploying PostgresML. A [developer guide](https://postgresml.org/developer_guide/overview/) is also available for those who would like to contribute.
