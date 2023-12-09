# Building from Source

PostgresML is a Postgres extension written in Rust, so it can be built and installed on any system supported by PostgreSQL and the Rust compiler. If you're planning on using GPU acceleration for Large Language Models or for XGBoost / LightGBM supervised learning, we would recommend you use an operating system well supported by Nvidia drivers and Cuda. Thankfully, that list is pretty large these days, including popular distributions like Ubuntu, Debian, RHEL, Centos, Fedora and OpenSuse.

### Dependencies

PostgresML depends on a few system packages and libraries that should be installed separately. The names of the packages vary based on the Linux distribution you're using, but in most cases you should be able to find all of them in your package manager repositories:

```
cmake
clang
pkg-config
build-essential
git
libclang-dev
libpython3-dev
libssl-dev
libopenblas-dev
postgresql-server-dev-14
lld
```

This guide assumes that you're using PostgreSQL 14, so if your Postgres version is different, replace `14` in `postgresql-server-dev-14` with the correct version. PostgresML supports all Postgres versions supported by `pgrx` and the PostgreSQL community (as of this writing, versions 12 through 16).

### Getting the source code

All of our source code is open source and hosted on GitHub. You can download it with git:

```bash
git clone https://github.com/postgresml/postgresml && \
cd postgresml && \
git submodule update --init --recursive
```

The repository contains the extension, the dashboard, SDKs, and all apps we've written that are powered by PostgresML.

### Installing PostgresML

For a typical deployment in production, you would need to compile and install the extension into your system PostgreSQL installation. PostgresML is using the `pgrx` Rust extension toolkit, so this is straight forward.

#### Install pgrx

`pgrx` is open source and available from crates.io. We are currently using the `0.10.0` version. It's important that your `pgrx` version matches what we're using, since there are some hard dependencies between our code and `pgrx`.

To install `pgrx`, simply run:

```
cargo install cargo-pgrx --version "0.10.0"
```

Before using `pgrx`, it needs to be initialized against the installed version of PostgreSQL. In this example, we'll be using the Ubuntu 22.04 default PostgreSQL 14 installation:

```
cargo pgrx init --pg14 /usr/bin/pg_config
```

#### Install the extension

Now that `pgrx` is initialized, you can compile and install the extension:

```
cd pgml-extension && \
cargo pgrx package
```

This will produce a number of artifacts in `target/release/pg14-pgml` which you can then copy to their respective folders in `/usr` using `sudo cp`. At the time writing, `pgrx` was working on a command that does this automatically, but it was not been released yet.

Once the files are copied into their respective folders in `/usr`, you need to make sure that the`pgml` extension is loaded in `shared_preload_libraries`. We use shared memory to control model versioning and other cool things that make PostgresML "just work". In `/etc/postgresql/14/main/postgresql.conf`, change or add the following line:

```
shared_preload_libraries = 'pgml'
```

Restart Postgres for this change to take effect:

```
sudo service postgresql restart
```

#### Validate the installation

To make sure PostgresML is installed correctly, you can create the extension in a database of your choice:

```
postgresml=# CREATE EXTENSION pgml;
INFO:  Python version: 3.10.6 (main, Nov  2 2022, 18:53:38) [GCC 11.3.0]
INFO:  Scikit-learn 1.1.3, XGBoost 1.7.1, LightGBM 3.3.3, NumPy 1.23.5
CREATE EXTENSION
```

