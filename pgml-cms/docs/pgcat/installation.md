# Installation

If you're using our Cloud, Dedicated databases come with the latest stable version of PgCat, managed deployments, and automatic configuration.

PgCat is free and open source, distributed under the MIT license. You can obtain its source code from our [repository](https://github.com/postgresml/pgcat) in GitHub. It can be installed by building it from source, by installing it from our APT repository, or by running it using our Docker image.

### Installing from source

To install PgCat from source, you'll need a recent version of the Rust compiler. Once setup, compiling PgCat is as simple as:

```
cargo build --release
```

which will produce the executable in `target/release/pgcat`. That executable can be placed into a system directory like `/usr/local/bin` and ran as a service or directly via a shell.

### Installing from APT

We are currently building and distributing a Debian package for Ubuntu 22.04 LTS as part of our release process. If you're using that version of Ubuntu, you can add our APT repository into your sources and install PgCat with `apt`:

```
sudo apt install pgcat
```

This will install the executable, a Systemd service called `pgcat`, and a configuration file template `/etc/pgcat.toml.example` which can be modified to your needs.

By default, the `pgcat` service will expect a `/etc/pgcat.toml` configuration file, which should be placed there by the user before the service can successfully start.

### Running with Docker

We automatically build and release a Docker image with each commit in the `main` branch of our GitHub repository. This image can be used as-is, but does require the user to provide a `pgcat.toml` configuration file.

Assuming you have a `pgcat.toml` file in your current working directory, you can run the latest version of PgCat with just one command:

```bash
docker run \
    -v $(pwd)/pgcat.toml:/etc/pgcat/pgcat.toml \
    ghcr.io/postgresml/pgcat:latest
```
