FROM ubuntu:jammy
MAINTAINER team@postgresml.com

RUN apt-get update
ARG DEBIAN_FRONTEND=noninteractive
ENV TZ=Etc/UTC
RUN apt-get update && apt-get install -y curl lsb-release  python3 python3-pip  tzdata sudo cmake libpq-dev libclang-dev wget postgresql-plpython3-14 postgresql-14

RUN echo "deb [trusted=yes] https://apt.postgresml.org $(lsb_release -cs) main" >> /etc/apt/sources.list
RUN cat /etc/apt/sources.list
RUN apt-get update && apt-get install -y postgresql-pgml-14

# Cache this, quicker
RUN pip3 install xgboost sklearn diptest torch lightgbm transformers datasets sentencepiece sacremoses sacrebleu rouge

COPY --chown=postgres:postgres . /app
WORKDIR /app

# Install pgml extension globally.
RUN pip3 install .

# Listen on 0.0.0.0 and allow 'root' to connect without a password.
# Please modify for production deployments accordingly.
RUN cp /app/docker/postgresql.conf /etc/postgresql/14/main/postgresql.conf
RUN cp /app/docker/pg_hba.conf /etc/postgresql/14/main/pg_hba.conf

WORKDIR /app
ENTRYPOINT ["/bin/bash", "/app/docker/entrypoint.sh"]
