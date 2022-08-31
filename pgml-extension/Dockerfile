FROM ubuntu:jammy
MAINTAINER team@postgresml.com

RUN apt-get update
ARG DEBIAN_FRONTEND=noninteractive
ENV TZ=Etc/UTC
RUN apt-get update && apt-get install -y curl lsb-release  python3 python3-pip  tzdata sudo cmake libpq-dev libclang-dev wget postgresql-plpython3-14 postgresql-14

RUN if uname -a | grep "x86"; then \
	curl https://static.postgresml.org/packages/postgresql-pgml-14_0.0.1-ubuntu1.0-amd64.deb -o pgml_rust.deb; \
	fi
RUN if uname -a | grep "aarch64"; then \
	curl https://static.postgresml.org/packages/postgresql-pgml-14_0.0.1-ubuntu1.0-arm64.deb -o pgml_rust.deb; \
	fi

RUN dpkg -i pgml_rust.deb && rm pgml_rust.deb

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
