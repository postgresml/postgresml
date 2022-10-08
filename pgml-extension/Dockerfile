#
# Extension builder.
#
ARG UBUNTU_VERSION=22.04
FROM ubuntu:${UBUNTU_VERSION}

ARG PGVERSION=14
ARG DEBIAN_FRONTEND=noninteractive
ARG PACKAGE_VERSION=2.0.0
ARG PACKAGE_PYTHON=true
ENV TZ=Etc/UTC

# Apt-fast
RUN apt-get update && apt-get install software-properties-common -y

RUN add-apt-repository ppa:apt-fast/stable --yes
RUN apt-get update && apt-get -y install apt-fast

RUN  apt-fast install apt-fast curl unzip gpg -y

# PostgresSQL
RUN curl https://www.postgresql.org/media/keys/ACCC4CF8.asc | gpg --dearmor | tee /etc/apt/trusted.gpg.d/apt.postgresql.org.gpg > /dev/null
RUN echo "deb http://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list

# CMake
RUN curl -L https://apt.kitware.com/keys/kitware-archive-latest.asc | gpg --dearmor | tee /usr/share/keyrings/kitware-archive-keyring.gpg >/dev/null
RUN echo "deb [signed-by=/usr/share/keyrings/kitware-archive-keyring.gpg] https://apt.kitware.com/ubuntu/ $(lsb_release -cs) main" | tee /etc/apt/sources.list.d/kitware.list >/dev/null

RUN apt-get update && apt-fast install postgresql-${PGVERSION} libopenblas-dev cmake postgresql-server-dev-${PGVERSION} pkg-config libssl-dev build-essential libclang-dev clang libpython3-dev python3-dev -y

USER postgres

# Install Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/var/lib/postgresql/.cargo/bin:${PATH}"

# Install tcdi/pgx
RUN cargo install cargo-pgx --version 0.4.5
RUN cargo pgx init --pg${PGVERSION} /usr/bin/pg_config

COPY --chown=postgres:postgres . /app
WORKDIR /app

# cargo pgx package command ignores --no-default-features sadly, so we
# have to have a separate Cargo.toml without Python. No big deal.
# Make sure to diff it and update it if you change the main Cargo.toml.
ENV PACKAGE_PYTHON=${PACKAGE_PYTHON}
RUN bash -c 'if [[ "${PACKAGE_PYTHON}" == "false" ]]; then cp docker/Cargo.toml.no-python Cargo.toml; fi'

# Build and upload package to S3
RUN cargo pgx package

# Deb file goes here, mount it on your local system
VOLUME /output
USER root

ENV PACKAGE_VERSION=$PACKAGE_VERSION
ENV OUTPUT_DIR=/output

# Run
ENTRYPOINT ["bash", "docker/build_ubuntu_deb.sh", "/output"]

