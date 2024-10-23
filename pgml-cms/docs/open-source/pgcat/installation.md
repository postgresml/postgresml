---
description: PgCat installation instructions from source, Aptitude repository and using Docker.
---

# PgCat installation

If you're using our [cloud](https://postgresml.org/signup), you're already using PgCat. All databases are using the latest and greatest PgCat version, with automatic updates and monitoring. You can connect directly with your PostgreSQL client libraries and applications, and PgCat will take care of the rest.

## Open source

PgCat is free and open source, distributed under the MIT license. You can obtain its source code from our [repository in GitHub](https://github.com/postgresml/pgcat). PgCat can be installed by building it from source, by downloading it from our Aptitude repository, or by using our Docker image.

### Installing from source

To install PgCat from source, you'll need a recent version of the Rust compiler and the C/C++ build toolchain to compile dependencies, like `pg_query`. If you have those installed already, compiling PgCat is as simple as:

```
cargo build --release
```

This will produce the executable in `target/release/pgcat` directory which can be placed into a system directory like `/usr/local/bin` and ran as a Systemd service, or directly via a shell command.

### Installing from Aptitude

As part of our regular release process, we are building and distributing a Debian package for Ubuntu 22.04 LTS. If you're using that version of Ubuntu, you can add our Aptitude repository into your sources and install PgCat with `apt`:

```
echo "deb [trusted=yes] https://apt.postgresml.org $(lsb_release -cs) main" | \
sudo tee -a /etc/apt/sources.list && \
sudo apt-get update && \
sudo apt install pgcat
```

The Debian package will install the following items:

- The PgCat executable, placed into `/usr/bin/pgcat`
- A Systemd service definition, placed into `/usr/systemd/system/pgcat.service`
- A configuration file template, placed into `/etc/pgcat.example.toml`

By default, the `pgcat` service will expect the configuration file to be located in `/etc/pgcat.toml`, so make sure to either write your own, or modify and rename the template before starting the service.

### Running with Docker

With each commit to the `main` branch of our [GitHub repository](https://github.com/postgresml/pgcat), we build and release a Docker image. This image can be used as-is, but does require the user to provide a `pgcat.toml` configuration file.

Assuming you have `pgcat.toml` in your current working directory, you can run the latest version of PgCat with just one command:

```bash
docker run \
  -v $(pwd)/pgcat.toml:/etc/pgcat/pgcat.toml \
ghcr.io/postgresml/pgcat:latest
```
