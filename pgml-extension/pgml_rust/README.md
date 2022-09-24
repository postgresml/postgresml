# PostgresML v2.0

PostgresML v2.0 rewritten in Rust for a 33x increase in performance.

## Requirements

We mostly develop this on Linux (Ubuntu), so it'll work best there, but it's very likely to work on other distros as well. Windows is not supported, except WSL2. Mac OS is supported, but some issues have been reported on M1s and M2s; that support is in progress.

For Scikit compatibility layer, Python 3.9 or Python 3.10 is required.

## Dependencies

If you haven't already, install:

- `cmake`
- `libclang-dev`
- `libopenblas-dev`
- `build-essential`
- Latest Rust compiler (https://rust-lang.org)

## Python

For using the Scikit-Learn backend, you need to install Python 3.9 or above. If your system comes with Python 3.8 or lower, you'll need to install `libpython3.9-dev` or `libpython3.10-dev`.

1. `sudo add-apt-repository ppa:deadsnakes/ppa`
2. `sudo apt update && sudo apt install libpython3.9-dev libpython3.10-dev`


## Local development

1. `cargo install cargo-pgx`
2. `cargo pgx init`
3. `cargo pgx run`
4. `DROP EXTENSION IF EXISTS pgml_rust;`
5. `CREATE EXTENSION pgml_rust;`
6. `SELECT * FROM pgml_rust.load_dataset('diabetes');`
7. `SELECT pgml_rust.train('Project name', 'regression', pgml_rust.diabetes', 'target', 'xgboost');`
8. `SELECT * FROM pgml_rust.predict('Project name', ARRAY[1, 5.0, 2.0]);`

## Packaging

This requires Docker. Once Docker is installed, you can run:

```bash
bash build_extension.sh
```

which will produce a `.deb` file in the current directory. The deb file can be installed with `apt-get`, for example:

```bash
apt-get install ./postgresql-pgml-12_0.0.4-ubuntu20.04-amd64.deb
```

which will take care of installing its dependencies as well. Make sure to run this as root and not with sudo.
