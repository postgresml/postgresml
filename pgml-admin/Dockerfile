FROM ubuntu:20.04
MAINTAINER docker@postgresml.com

RUN apt-get update
ARG DEBIAN_FRONTEND=noninteractive
ENV TZ=Etc/UTC
RUN apt-get install -y python3 python3-pip

COPY --chown=app:app . /app
WORKDIR /app/pgml

RUN pip3 install -r requirements.txt

WORKDIR /app
ENTRYPOINT ["/bin/bash", "/app/docker/entrypoint.sh"]
