FROM debian:bullseye-slim
MAINTAINER docker@postgresml.com

RUN apt-get update
ARG DEBIAN_FRONTEND=noninteractive
ENV TZ=Etc/UTC
RUN apt-get install -y postgresql-plpython3-13 python3 python3-pip postgresql-13 tzdata sudo cmake libpq-dev

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
