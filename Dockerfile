FROM ubuntu:20.04
RUN apt-get update
ARG DEBIAN_FRONTEND=noninteractive
ENV TZ=Etc/UTC
RUN apt-get install -y tzdata
RUN apt-get install -y postgresql-plpython3-12 python3 python3-pip postgresql-12
RUN pip3 install sklearn
RUN apt-get install sudo -y
COPY . /app
WORKDIR /app/pgml
RUN python3 setup.py install
WORKDIR /app
ENTRYPOINT ["/bin/bash", "/app/entrypoint.sh"]
