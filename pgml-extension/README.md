# PostgresML 2.0

PostgresML is a PostgreSQL extension providing end-to-end machine learning inside your database. The extension is written in Rust using `tcdi/pgx` and provides LightGBM, XGBoost and [Linfa](https://github.com/rust-ml/linfa) algorithms.

A backwards compatibility layer to Scikit-learn is provided as well, so the entirety of Scikit, XGBoost and LightGBM are available via the standard Scikit interface using Python. The Python layer is written using `pyo3`.

See [our blog](https://postgresml.org/blog/postgresml-is-moving-to-rust-for-our-2.0-release/) for a performance comparison to Python.

## Requirements

PostgresML 2.0 requires Python 3.7 or above and the Rust compiler and toolchain. You can download the Rust compiler [here](https://rust-lang.org).

We develop this extension on Ubuntu, so it'll work best there, but it's very likely to work on other distros as well. Windows is only supported through WSL2. It's been tested and it works. Mac OS is supported, however due to a bug in LLVM's distribution of OpenMP, Apple M1s and M2s are not currently supported. The issue is being tracked [here](https://github.com/postgresml/postgresml/issues/364).

## Dependencies

If you haven't already, install:

- `cmake`
- `libclang-dev`
- `libopenblas-dev`
- `build-essential`
- `libpython3-dev` (Python 3.7 or higher)

## Python

If your system comes with Python 3.8 or lower, you'll need to install `libpython3.7-dev` or higher. You can get it from [`ppa:deadsnakes/ppa`](https://launchpad.net/~deadsnakes/+archive/ubuntu/ppa):

1. `sudo add-apt-repository ppa:deadsnakes/ppa`
2. `sudo apt update && sudo apt install libpython3.7-dev`


## Update postgresql.conf

PostgresML 2.0 requires to be loaded as a shared library. For local development, this is in `~/.pgx/data-13/postgresql.conf`:

```
shared_preload_libraries = 'pgml'     # (change requires restart)
```

## Local development

0. `git submodule update --init --recursive`
1. `cargo install cargo-pgx`
2. `cargo pgx init` (this will take a while, go get a coffee)
3. `cargo pgx run`
4. `DROP EXTENSION IF EXISTS pgml; DROP SCHEMA IF EXISTS pgml CASCADE;`
5. `CREATE EXTENSION pgml;`
6. `SELECT * FROM pgml.load_dataset('diabetes');`
7. `SELECT * FROM pgml.train('Project name', 'regression', 'pgml.diabetes', 'target', 'xgboost');`
8. `SELECT target, pgml.predict('Project name', ARRAY[age, sex, bmi, bp, s1, s2, s3, s4, s5, s6]) FROM pgml.diabetes LIMIT 10;`

## Packaging

This requires Docker. Once Docker is installed, you can run:

```bash
bash build_extension.sh
```

which will produce a `.deb` file in the current directory (this will take about 20 minutes). The deb file can be installed with `apt-get`, for example:

```bash
apt-get install ./postgresql-pgml-12_0.0.4-ubuntu20.04-amd64.deb
```

which will take care of installing its dependencies as well. Make sure to run this as root and not with sudo.
