# Rust meet PostgresML

Here we have some POC code to use Rust for PostgresML.

## Dependencies

All dependencies are vendored. I downloaded XGBoost 1.62 and all its submodules. We're also using the `master` branch of `xgboost` Rust crate.

If you haven't already, install:

- `cmake`
- `libclang-dev`

## Local development

1. `cargo install pgx`
2. `cargo pgx run`
3. `CREATE SCHEMA pgml`
4. `\i diabetes.sql`
5. `SELECT pgml_train('pgml.diabetes', ARRAY['age', 'sex', ...], 'target');`
6. `SELECT * FROM pgml_predict(ARRAY[1, 5.0, ...]);`

Lots of todos, but still a decent PoC.
