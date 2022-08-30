# Rust meet PostgresML

Here we have some POC code to use Rust for PostgresML.

## Dependencies

We're pulling rust-xgboost directly from Git because it's using the latest version of clang,
and we can't link more than one version of clang into the same executable/library. PGX is using a newer one,
hence the conflict.

1. `git clone https://github.com/davechallis/rust-xgboost`
2. `cd rust-xgboost`
3. `git submodule init && git submodule update`
4. `cd xgboost-sys/xgboost && git submodule init && git submodule update`
5. Modify `rust-xgboost/Cargo.toml` and change `xgboost-sys = "..."` to use the local version of `xgboost-sys`:

```
xgboost-sys = { path = "./xgboost-sys" }
```

## Local development

1. `cargo install pgx`
2. `cargo pgx run`
3. `CREATE SCHEMA pgml`
4. `\i diabetes.sql`
5. `SELECT pgml_train('pgml.diabetes', ARRAY['age', 'sex', ...], 'target');`
6. `SELECT * FROM pgml_predict(ARRAY[1, 5.0, ...]);`

Lots of todos, but still a decent PoC.
