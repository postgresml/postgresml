FROM ubuntu:20.04
MAINTAINER docker@postgresml.com

RUN apt-get update
ARG DEBIAN_FRONTEND=noninteractive
ENV TZ=Etc/UTC
RUN apt-get install -y python3 python3-pip libpq-dev curl postgresql-client-12 tzdata

COPY requirements.txt /app/requirements.txt
WORKDIR /app

RUN pip3 install -r requirements.txt

COPY . /app
WORKDIR /app

ENTRYPOINT ["/bin/bash", "/app/docker/entrypoint.sh"]
