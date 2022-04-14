FROM ubuntu:20.04
RUN apt-get update
ARG DEBIAN_FRONTEND=noninteractive
ENV TZ=Etc/UTC
RUN apt-get install -y postgresql-plpython3-12 python3 python3-pip postgresql-12 tzdata sudo
COPY --chown=postgres:postgres . /app
WORKDIR /app/pgml
RUN python3 setup.py install
RUN cp /app/docker/postgresql.conf /etc/postgresql/12/main/postgresql.conf
RUN cp /app/docker/pg_hba.conf /etc/postgresql/12/main/pg_hba.conf
WORKDIR /app
ENTRYPOINT ["/bin/bash", "/app/docker/entrypoint.sh"]
