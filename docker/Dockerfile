FROM nvidia/cuda:12.6.3-devel-ubuntu24.04
ENV PATH="/usr/local/cuda/bin:${PATH}"
RUN apt update && \
	apt install -y \
		lsb-release \
		curl \
		ca-certificates \
		gnupg \
		coreutils \
		sudo \
		openssl \
		python3-pip \
		software-properties-common

# Add deadsnakes PPA for Python 3.11
RUN add-apt-repository -y ppa:deadsnakes/ppa && \
    apt update && \
    apt install -y python3.11 python3.11-dev python3.11-venv python3.11-distutils

RUN echo "deb [trusted=yes] https://apt.postgresml.org $(lsb_release -cs) main" > /etc/apt/sources.list.d/postgresml.list
RUN echo "deb http://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list
RUN curl https://www.postgresql.org/media/keys/ACCC4CF8.asc | gpg --dearmor | tee /etc/apt/trusted.gpg.d/apt.postgresql.org.gpg >/dev/null

ENV TZ=UTC
ENV DEBIAN_FRONTEND=noninteractive
RUN apt update -y && \
    apt install -y git postgresml-python && \
    apt install -y postgresml-17 postgresml-dashboard
RUN git clone --branch v0.8.0 https://github.com/pgvector/pgvector && \
cd pgvector && \
echo "trusted = true" >> vector.control && \
	make && \
	make install

COPY entrypoint.sh /app/entrypoint.sh
COPY dashboard.sh /app/dashboard.sh

COPY --chown=postgres:postgres local_dev.conf /etc/postgresql/17/main/conf.d/01-local_dev.conf
COPY --chown=postgres:postgres pg_hba.conf /etc/postgresql/17/main/pg_hba.conf

ENTRYPOINT ["bash", "/app/entrypoint.sh"]
