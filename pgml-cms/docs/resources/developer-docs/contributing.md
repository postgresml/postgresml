# Contributing

Thank you for your interest in contributing to PostgresML! We are an open source, MIT licensed project, and we welcome all contributions, including bug fixes, features, documentation, typo fixes, and Github stars.

Our project consists of three (3) applications:

1. Postgres extension (`pgml-extension`)
2. Dashboard web app (`pgml-dashboard`)
3. Documentation (`pgml-cms`)

The development environment for each differs slightly, but overall we use Python, Rust, and PostgreSQL, so as long as you have all of those installed, the setup should be straight forward.

## Build Dependencies

1. Install the latest Rust compiler from [rust-lang.org](https://www.rust-lang.org/learn/get-started).
2. Install a [modern version](https://apt.kitware.com/) of CMake.
3.  Install PostgreSQL development headers and other dependencies:

    ```commandline
    export POSTGRES_VERSION=15
    sudo apt-get update && \
    sudo apt-get install -y \
        postgresql-server-dev-${POSTGRES_VERSION} \
        bison \
        build-essential \
        clang \
        cmake \
        flex \
        libclang-dev \
        libopenblas-dev \
        libpython3-dev \
        libreadline-dev \
        libssl-dev \
        pkg-config \
        python3-dev
    ```
4.  Install the Python dependencies

    If your system comes with Python 3.6 or lower, you'll need to install `libpython3.7-dev` or higher. You can get it from [`ppa:deadsnakes/ppa`](https://launchpad.net/\~deadsnakes/+archive/ubuntu/ppa):

    ```commandline
    sudo add-apt-repository ppa:deadsnakes/ppa && \
    sudo apt update && sudo apt install -y libpython3.7-dev
    ```
5.  Clone our git repository:

    ```commandline
    git clone https://github.com/postgresml/postgresml && \
    cd postgresml && \
    git submodule update --init --recursive && \
    ```

## Postgres extension

PostgresML is a Rust extension written with `tcdi/pgrx` crate. Local development therefore requires the [latest Rust compiler](https://www.rust-lang.org/learn/get-started) and PostgreSQL development headers and libraries.

The extension code is located in:

```commandline
cd pgml-extension/
```

You'll need to install basic dependencies

Once there, you can initialize `pgrx` and get going:

#### Pgrx command line and environments

```commandline
cargo install cargo-pgrx --version "0.11.2" --locked && \
cargo pgrx init # This will take a few minutes
```

#### Huggingface transformers

If you'd like to use huggingface transformers with PostgresML, you'll need to install the Python dependencies:

```commandline
sudo pip3 install -r requirements.txt
```

#### Update postgresql.conf

`pgrx` uses Postgres 15 by default. Since `pgml` is using shared memory, you need to add it to `shared_preload_libraries` in `postgresql.conf` which, for `pgrx`, is located in `~/.pgrx/data-15/postgresql.conf`.

```
shared_preload_libraries = 'pgml'     # (change requires restart)
```

Run the unit tests

```commandline
cargo pgrx test
```

Run the integration tests:

```commandline
cargo pgrx run --release
psql -h localhost -p 28813 -d pgml -f tests/test.sql -P pager
```

Run an interactive psql session

```commandline
cargo pgrx run
```

Create the extension in your database:

```commandline
CREATE EXTENSION pgml;
```

That's it, PostgresML is ready. You can validate the installation by running:


{% tabs %}
{% tab title="SQL" %}
```postgresql
SELECT pgml.version();
```
{% endtab %}

{% tab title="Output" %}
```postgresql
postgres=# select pgml.version();
      version
-------------------
 2.9.1
(1 row)
```
{% endtab %}
{% endtabs %}

Basic extension usage:

```postgresql
SELECT * FROM pgml.load_dataset('diabetes');
SELECT * FROM pgml.train('Project name', 'regression', 'pgml.diabetes', 'target', 'xgboost');
SELECT target, pgml.predict('Project name', ARRAY[age, sex, bmi, bp, s1, s2, s3, s4, s5, s6]) FROM pgml.diabetes LIMIT 10;
```

By default, the extension is built without CUDA support for XGBoost and LightGBM. You'll need to install CUDA locally to build and enable the `cuda` feature for cargo. CUDA can be downloaded [here](https://developer.nvidia.com/cuda-downloads?target\_os=Linux).

```commandline
CUDACXX=/usr/local/cuda/bin/nvcc cargo pgrx run --release --features pg15,python,cuda
```

If you ever want to reset the environment, simply spin up the database with `cargo pgrx run` and drop the extension and metadata tables:

```postgresql
DROP EXTENSION IF EXISTS pgml CASCADE;
DROP SCHEMA IF EXISTS pgml CASCADE;
CREATE EXTENSION pgml;
```

#### Packaging

This requires Docker. Once Docker is installed, you can run:

```bash
bash build_extension.sh
```

which will produce a `.deb` file in the current directory (this will take about 20 minutes). The deb file can be installed with `apt-get`, for example:

```bash
apt-get install ./postgresql-pgml-12_0.0.4-ubuntu20.04-amd64.deb
```

which will take care of installing its dependencies as well. Make sure to run this as root and not with sudo.

## Run the dashboard

The dashboard is a web app that can be run against any Postgres database with the extension installed. There is a Dockerfile included with the source code if you wish to run it as a container.

The dashboard requires a Postgres database with the [pgml-extension](https://github.com/postgresml/postgresml/tree/master/pgml-extension) to generate the core schema. See that subproject for developer setup.

We develop and test this web application on Linux, OS X, and Windows using WSL2.

Basic installation can be achieved with:

1. Clone the repo (if you haven't already for the extension):

```commandline
  cd postgresml/pgml-dashboard
```

2. Set the `DATABASE_URL` environment variable, for example to a running interactive `cargo pgrx run` session started previously:

```commandline
export DATABASE_URL=postgres://localhost:28815/pgml
```

3. Run migrations

```commandline
sqlx migrate run
```

4. Run tests:

```commandline
cargo test
```

5. Incremental and automatic compilation for development cycles is supported with:

```commandline
cargo watch --exec run
```

The website can be packaged for distribution. You'll need to copy the static files along with the `target/release` directory to your server.

## General

We are a cross-platform team, some of us use WSL and some use Linux or Mac OS. Keeping that in mind, it's good to use common line endings for all files to avoid production errors, e.g. broken Bash scripts.

The project is presently using [Unix line endings](https://docs.github.com/en/get-started/getting-started-with-git/configuring-git-to-handle-line-endings).
