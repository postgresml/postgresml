# Rust meet PostgresML

Here we have some POC code to use Rust for PostgresML.

## Dependencies

All dependencies are vendored. I downloaded XGBoost 1.62 and all its submodules. We're also using the `master` branch of `xgboost` Rust crate and `openblas-src`.

If you haven't already, install:

- `cmake`
- `libclang-dev`
- `libopenblas-dev`

## Local development

1. `cargo install cargo-pgx`
2. `cargo pgx init`
3. `cargo pgx run`
4. `DROP EXTENSION IF EXISTS pgml_rust;`
5. `CREATE EXTENSION pgml_rust;`
6. `SELECT pgml_rust.train('Project name', 'regression', pgml_rust.diabetes', 'target', 'xgboost', '{}');`
7. `SELECT * FROM pgml_rust.predict('Project name', ARRAY[1, 5.0, 2.0]);`

## Packaging

We currently support Ubuntu 18.04 and newer. Mac OS (Apple Sillicon) support is in progress. Provided in the repo is the `.deb` builder, which requires Docker. Once Docker is installed, you can run:

```bash
bash build_extension.sh
```

which will produce a `.deb` file in the current directory. The deb file can be installed with `apt-get`, for example:

```bash
apt-get install ./postgresql-pgml-12_0.0.4-ubuntu20.04-amd64.deb
```

which will take care of installing its dependencies as well. Make sure to run this as root and not with sudo.
