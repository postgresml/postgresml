FROM ubuntu:jammy
ARG POSTGRESML_VERSION=2.7.1-2
RUN apt update && \
	apt install -y \
		lsb-release \
		curl \
		ca-certificates \
		gnupg \
		coreutils \
		sudo
RUN echo "deb [trusted=yes] https://apt.postgresml.org $(lsb_release -cs) main" > /etc/apt/sources.list.d/postgresml.list
RUN echo "deb http://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list
RUN curl https://www.postgresql.org/media/keys/ACCC4CF8.asc | gpg --dearmor | tee /etc/apt/trusted.gpg.d/apt.postgresql.org.gpg >/dev/null

ENV TZ=UTC
ENV DEBIAN_FRONTEND=noninteractive
RUN apt update -y && apt install postgresml-15=${POSTGRESML_VERSION} -y

COPY entrypoint.sh /app/entrypoint.sh
COPY --chown=postgres:postgres local_dev.conf /etc/postgresql/15/main/conf.d/01-local_dev.conf
COPY --chown=postgres:postgres pg_hba.conf /etc/postgresql/15/main/pg_hba.conf

ENTRYPOINT ["bash", "/app/entrypoint.sh"]
