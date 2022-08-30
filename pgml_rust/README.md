# Rust meet PostgresML

Here we have some POC code to use Rust for PostgresML.

## Dependencies

1. `git submodule init`
2. `git submodule update`
3. `cd src/rust-xgboost/xgboost-sys`
4. `git submodule init`
5. `git submodule update`
6. `cd xgboost`
7. `git submodule init`
8. `git submodule update`

Phew, that was a lot of submodules. You just downloaded:

- `xgboost` Rust bindings
- `xgboost-sys` Bindgen 1:1 Rust:C++ XGBoost types,
- `xgboost` C++ source code
- `xgboost` dependencies

## Local development

1. `cargo install pgx`
2. `cargo pgx run`
3. `\i diabetes.sql`
4. `SELECT pgml_train('pgml.diabetes', ARRAY['age', 'sex', ...], 'target');`
5. `SELECT * FROM pgml_predict(ARRAY[1, 5.0, ...]);`

Lots of todos, but still a decent PoC.
