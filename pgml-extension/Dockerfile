FROM ubuntu:20.04
MAINTAINER docker@postgresml.com

RUN apt-get dist-upgrade
RUN apt-get update
ARG DEBIAN_FRONTEND=noninteractive
ENV TZ=Etc/UTC
RUN apt-get install -y postgresql-plpython3-12 python3 python3-pip postgresql-12 tzdata sudo

# Cache this, quicker
RUN pip3 install xgboost sklearn

COPY --chown=postgres:postgres . /app
WORKDIR /app

# Install pgml extension globally.
RUN pip3 install .

# Listen on 0.0.0.0 and allow 'root' to connect without a password.
# Please modify for production deployments accordingly.
RUN cp /app/docker/postgresql.conf /etc/postgresql/12/main/postgresql.conf
RUN cp /app/docker/pg_hba.conf /etc/postgresql/12/main/pg_hba.conf

WORKDIR /app
ENTRYPOINT ["/bin/bash", "/app/docker/entrypoint.sh"]

