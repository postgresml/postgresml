FROM nvidia/cuda:12.1.1-devel-ubuntu22.04
LABEL maintainer="team@postgresml.com"

ARG DEBIAN_FRONTEND=noninteractive
ARG PGML_VERSION=2.6.0
ENV TZ=Etc/UTC
ENV PATH="/usr/local/cuda/bin:${PATH}"

# Dependencies
RUN apt-get update && \
    apt-get install -y \
    curl \
    lsb-release \
    python3 \
    python3-pip \
    tzdata \
    sudo \
    cmake \
    libpq-dev \
    libclang-dev \
    wget \
    git \
    gnupg \
    lsb-release

# PostgreSQL
RUN wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | apt-key add - && \
    echo "deb http://apt.postgresql.org/pub/repos/apt/ $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list && \
    apt-get update && \
    apt-get install -y postgresql-14 && \
    apt-get install -y postgresql-plpython3-14

# PostgresML
RUN echo "deb [trusted=yes] https://apt.postgresml.org $(lsb_release -cs) main" >> /etc/apt/sources.list
RUN cat /etc/apt/sources.list
RUN apt-get update && apt-get install -y postgresql-pgml-14=${PGML_VERSION}

COPY --chown=postgres:postgres requirements.txt /app/requirements.txt
WORKDIR /app
RUN pip3 install -r requirements.txt

COPY --chown=postgres:postgres requirements-xformers.txt /app/requirements-xformers.txt
RUN pip3 install -r requirements-xformers.txt --no-dependencies

COPY --chown=postgres:postgres . /app
# Listen on 0.0.0.0 and allow 'root' to connect without a password.
# Please modify for production deployments accordingly.
RUN cp /app/docker/postgresql.conf /etc/postgresql/14/main/postgresql.conf
RUN cp /app/docker/pg_hba.conf /etc/postgresql/14/main/pg_hba.conf
RUN cd /tmp && \
    git clone --branch v0.4.1 https://github.com/pgvector/pgvector.git && \
    cd pgvector && \
    make && \
    make install

ENTRYPOINT ["/bin/bash", "/app/docker/entrypoint.sh"]
