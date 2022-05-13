FROM python:3.10
MAINTAINER docker@postgresml.com

RUN apt-get update
ARG DEBIAN_FRONTEND=noninteractive
ENV TZ=Etc/UTC
RUN apt-get install -y libpq-dev curl postgresql-client-13 tzdata

COPY requirements.txt /app/requirements.txt
WORKDIR /app

RUN pip install -U pip && \
	pip install -r requirements.txt

COPY . /app
WORKDIR /app

ENTRYPOINT ["/bin/bash", "/app/docker/entrypoint.sh"]
