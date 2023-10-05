# Self-hosting

PostgresML is a Postgres extension, so running it is very similar to running a self-hosted PostgreSQL database server. A typical architecture consists of a primary database that will serve reads and writes, optional replicas to scale reads horizontally, and a pooler to load balance connections.

### Operating system

At PostgresML, we prefer running Postgres on Ubuntu, mainly because of its extensive network of supported hardware architectures, packages, and drivers. The rest of this guide will assume that we're using Ubuntu 22.04, the current long term support release of Ubuntu, but you can run PostgresML pretty easily on any other flavor of Linux.

### Installing PostgresML

PostgresML for Ubuntu 22.04 can be downloaded directly from our APT repository. There is no need to install any additional dependencies or compiling from source.

To add our APT repository to our sources, you can run:

```bash
echo "deb [trusted=yes] https://apt.postgresml.org jammy main" | \
sudo tee -a /etc/apt/sources.list
```

We don't sign our Debian packages since we can rely on HTTPS to guarantee the authenticity of our binaries.

Once you've added the repository, make sure to update APT:

```bash
sudo apt update
```

Finally, you can install PostgresML:

```bash
sudo apt install -y postgresml-14
```

Ubuntu 22.04 ships with PostgreSQL 14, but if you have a different version installed on your system, just change `14` in the package name to your Postgres version. We currently support all  versions supported by the community: Postgres 12 through 15.

### Validate your installation

You should be able to connect to Postgres and install the extension into the database of your choice:

```bash
sudo -u postgres psql
```

```
postgres=# CREATE EXTENSION pgml;
INFO:  Python version: 3.10.6 (main, Nov  2 2022, 18:53:38) [GCC 11.3.0]
INFO:  Scikit-learn 1.1.3, XGBoost 1.7.1, LightGBM 3.3.3, NumPy 1.23.5
CREATE EXTENSION
postgres=#
```

