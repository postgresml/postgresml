FROM rust:1-bullseye AS rust_builder
RUN apt-get update && apt-get install -y postgresql-13 libpq-dev cmake libclang-dev postgresql-server-dev-13
COPY --chown=postgres:postgres pgml_rust /pgml_rust
WORKDIR /pgml_rust
USER postgres
RUN cargo install cargo-pgx
RUN cargo pgx init --pg13 /usr/bin/pg_config
RUN cargo pgx package

FROM debian:bullseye-slim
MAINTAINER team@postgresml.com

RUN apt-get update
ARG DEBIAN_FRONTEND=noninteractive
ENV TZ=Etc/UTC
RUN apt-get install -y postgresql-plpython3-13 python3 python3-pip postgresql-13 tzdata sudo cmake libpq-dev libclang-dev

# Install pgml_rust extension
COPY --from=rust_builder /pgml_rust/target/release/pgml_rust-pg13/usr/share/postgresql/13/extension/pgml_rust.control /usr/share/postgresql/13/extension/pgml_rust.control
COPY --from=rust_builder /pgml_rust/target/release/pgml_rust-pg13/usr/share/postgresql/13/extension/pgml_rust--0.0.0.sql /usr/share/postgresql/13/extension/pgml_rust--0.0.0.sql
COPY --from=rust_builder /pgml_rust/target/release/pgml_rust-pg13/usr/lib/postgresql/13/lib/pgml_rust.so /usr/lib/postgresql/13/lib/pgml_rust.so

# Cache this, quicker
RUN pip3 install xgboost sklearn diptest torch lightgbm transformers datasets sentencepiece sacremoses sacrebleu rouge

COPY --chown=postgres:postgres . /app
WORKDIR /app

# Install pgml extension globally.
RUN pip3 install .

# Listen on 0.0.0.0 and allow 'root' to connect without a password.
# Please modify for production deployments accordingly.
RUN cp /app/docker/postgresql.conf /etc/postgresql/13/main/postgresql.conf
RUN cp /app/docker/pg_hba.conf /etc/postgresql/13/main/pg_hba.conf

WORKDIR /app
ENTRYPOINT ["/bin/bash", "/app/docker/entrypoint.sh"]
