# PostgresML installation

The simplest PostgresML deployment consists of two parts: the PostgreSQL extension, and the dashboard web app. The extension provides all the machine learning functionality, and can be used independently. The dashboard provides a system overview for easier management, and SQL notebooks for writing experiments.

## Extension

The extension can be installed by compiling it from source, or if you're using Ubuntu 22.04, from our package repository.

### macOS

!!! tip

If you're just looking to try PostgresML without installing it on your system, take a look at our [Quick Start with Docker](quick-start-with-docker) guide.

!!!

#### Get the source code

To get the source code for PostgresML, clone our GitHub repository:

```bash
git clone https://github.com/postgresml/postgresml
```

#### Install dependencies

We provide a `Brewfile` that will install all the necessary dependencies for compiling PostgresML from source:

```bash
cd pgml-extension && \
brew bundle
```

**Rust**

PostgresML is written in Rust, so you'll need to install the latest compiler from [rust-lang.org](https://rust-lang.org). Additionally, we use the Rust PostgreSQL extension framework `pgrx`, which requires some initialization steps:

```bash
cargo install cargo-pgrx --version 0.11.2 && \
cargo pgrx init
```

This step will take a few minutes. Perfect opportunity to get a coffee while you wait.

### Compile and install

With all the dependencies installed, you can compile and install the extension:

```bash
cargo pgrx install
```

This will compile all the necessary packages, including Rust bindings to XGBoost and LightGBM, together with Python support for Hugging Face transformers and Scikit-learn. The extension will be automatically installed into the PostgreSQL installation created by the `postgresql@15` Homebrew formula.

### Python dependencies

PostgresML uses Python packages to provide support for Hugging Face LLMs and Scikit-learn algorithms and models. To make this work on your system, you have two options: install those packages into a virtual environment (strongly recommended), or install them globally.

{% tabs %}
{% tab title="Virtual environment" %}
To install the necessary Python packages into a virtual environment, use the `virtualenv` tool installed previously by Homebrew:

```bash
virtualenv pgml-venv && \
source pgml-venv/bin/activate && \
pip install -r requirements.txt
```
{% endtab %}

{% tab title="Globally" %}
Installing Python packages globally can cause issues with your system. If you wish to proceed nonetheless, you can do so:

```bash
pip3 install -r requirements.txt
```
{% endtab %}
{% endtabs %}

### Configuration

We have one last step remaining to get PostgresML running on your system: configuration.

PostgresML needs to be loaded into shared memory by PostgreSQL. To do so, you need to add it to `preload_shared_libraries`.

Additionally, if you've chosen to use a virtual environment for the Python packages, we need to tell PostgresML where to find it.

Both steps can be done by editing the PostgreSQL configuration file `postgresql.conf` usinig your favorite editor:

```bash
vim /opt/homebrew/var/postgresql@15/postgresql.conf
```

Both settings can be added to the config, like so:

```
shared_preload_libraries = 'pgml,pg_stat_statements'
pgml.venv = '/absolute/path/to/your/pgml-venv'
```

Save the configuration file and restart PostgreSQL:

```bash
brew services restart postgresql@15
```

### Test your installation

You should be able to connect to PostgreSQL and use our extension now:

!!! generic

!!! code\_block time="953.681ms"

```postgresql
CREATE EXTENSION pgml;
SELECT pgml.version();
```

!!!

!!! results

```
psql (15.3 (Homebrew))
Type "help" for help.

pgml_test=# CREATE EXTENSION pgml;
INFO:  Python version: 3.11.4 (main, Jun 20 2023, 17:23:00) [Clang 14.0.3 (clang-1403.0.22.14.1)]
INFO:  Scikit-learn 1.2.2, XGBoost 1.7.5, LightGBM 3.3.5, NumPy 1.25.1
CREATE EXTENSION

pgml_test=# SELECT pgml.version();
 version 
---------
 2.9.1
(1 row)
```

!!!

!!!

### pgvector

We like and use pgvector a lot, as documented in our blog posts and examples, to store and search embeddings. You can install pgvector from source pretty easily:

```bash
git clone --branch v0.6.0 https://github.com/pgvector/pgvector && \
cd pgvector && \
echo "trusted = true" >> vector.control && \
make && \
make install
```

**Test pgvector installation**

You can create the `vector` extension in any database:

!!! generic

!!! code\_block time="21.075ms"

```postgresql
CREATE EXTENSION vector;
```

!!!

!!! results

```
psql (15.3 (Homebrew))
Type "help" for help.

pgml_test=# CREATE EXTENSION vector;
CREATE EXTENSION
```

!!!

!!!

### Ubuntu

!!! note

If you're looking to use PostgresML in production, [try our cloud](https://postgresml.org/signup). We support serverless deployments with modern GPUs and dedicated hardware if you would like to tweak PostgresML to your needs.

!!!

For Ubuntu, we compile and ship packages that include everything needed to install and run the extension. At the moment, only Ubuntu 22.04 (Jammy) is supported.

#### Add our repository

Add our repository to your system:

```bash
echo "deb [trusted=yes] https://apt.postgresml.org $(lsb_release -cs) main" | \
sudo tee -a /etc/apt/sources.list
```

#### Install PostgresML

Update your package lists and install PostgresML:

```bash
export POSTGRES_VERSION=14
sudo apt update && \
sudo apt install postgresml-${POSTGRES_VERSION}
```

The `postgresml-14` package includes all the necessary dependencies, including Python packages shipped inside a virtual environment. Your PostgreSQL server is configured automatically.

We support PostgreSQL versions 11 through 15, so you can install the one matching your currently installed PostgreSQL version.

#### Installing just the extension

If you prefer to manage your own Python environment and dependencies, you can install just the extension:

```bash
export POSTGRES_VERSION=14
sudo apt install postgresql-pgml-${POSTGRES_VERSION}
```

#### Optimized pgvector

pgvector, the extension we use for storing and searching embeddings, needs to be installed separately for optimal performance. Your hardware may support vectorized operation instructions (like AVX-512), which pgvector can take advantage of to run faster.

To install pgvector from source, you can simply:

```bash
git clone --branch v0.6.0 https://github.com/pgvector/pgvector && \
cd pgvector && \
echo "trusted = true" >> vector.control && \
make && \
make install
```

### Other Linuxes

PostgresML will compile and run on pretty much any modern Linux distribution. For a quick example, you can take a look at what we do to build the extension on [Ubuntu](https://github.com/postgresml/postgresml/blob/master/.github/workflows/ubuntu-packages-and-docker-image.yml), and modify those steps to work on your distribution.

#### Get the source code

To get the source code for PostgresML, clone our GitHub repository:

```bash
git clone https://github.com/postgresml/postgresml
```

#### Dependencies

You'll need the following packages installed first. The names are taken from Ubuntu (and other Debian based distros), so you'll need to change them to fit your distribution:

```
export POSTGRES_VERSION=14

build-essential
clang
libopenblas-dev
libssl-dev
bison
flex
pkg-config
cmake
libreadline-dev
libz-dev
tzdata
sudo
libpq-dev
libclang-dev
postgresql-{POSTGRES_VERSION}
postgresql-server-dev-${POSTGRES_VERSION}
python3
python3-pip
libpython3
lld
```

**Rust**

PostgresML is written in Rust, so you'll need to install the latest compiler version from [rust-lang.org](https://rust-lang.org).

#### `pgrx`

We use the `pgrx` Postgres Rust extension framework, which comes with its own installation and configuration steps:

```bash
cd pgml-extension && \
cargo install cargo-pgrx --version 0.11.2 && \
cargo pgrx init
```

This step will take a few minutes since it has to download and compile multiple PostgreSQL versions used by `pgrx` for development.

#### Compile and install

Finally, you can compile and install the extension:

```bash
cargo pgrx install
```

## Dashboard

The dashboard is a web app that can be run against any Postgres database which has the extension installed. There is a [Dockerfile](https://github.com/postgresml/postgresml/blob/master/pgml-dashboard/Dockerfile) included with the source code if you wish to run it as a container.

### Get the source code

To get our source code, you can clone our GitHub repository (if you haven't already):

```bash
git clone clone https://github.com/postgresml/postgresml && \
cd pgml-dashboard
```

### Configure your database

Use an existing database which has the `pgml` extension installed, or create a new one:

```bash
createdb pgml_dashboard && \
psql -d pgml_dashboard -c 'CREATE EXTENSION pgml;'
```

### Configure the environment

Create a `.env` file with the necessary `DATABASE_URL`, for example:

```bash
DATABASE_URL=postgres:///pgml_dashboard
```

### Get Rust

The dashboard is written in Rust and uses the SQLx crate to interact with Postgres. Make sure to install the latest Rust compiler from [rust-lang.org](https://rust-lang.org).

### Database setup

To setup the database, you'll need to install `sqlx-cli` and run the migrations:

```bash
cargo install sqlx-cli --version 0.6.3 && \
cargo sqlx database setup
```

### Frontend dependencies

The dashboard frontend is using Sass which requires Node & the Sass compiler. You can install Node from Brew, your package repository, or by using [Node Version Manager](https://github.com/nvm-sh/nvm).

If using nvm, you can install the latest stable Node version with:

```bash
nvm install stable
```

Once you have Node installed, you can install the Sass compiler globally:

```bash
npm install -g sass
```

### Compile and run

Finally, you can compile and run the dashboard:

```
cargo run
```

Once compiled, the dashboard will be available on [localhost:8000](http://localhost:8000).

The dashboard can also be packaged for distribution. You'll need to copy the static files along with the `target/release` directory to your server.
