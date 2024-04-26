FROM ubuntu:22.04
ENV PGCAT_VERSION=2.0.0-alpha19
RUN apt update && \
	apt install -y curl postgresql-client-common postgresql-client-14 && \
	apt clean
WORKDIR /pgml-rds-proxy
COPY --chown=root:root download-pgcat.sh download-pgcat.sh
COPY --chown=root:root run.sh run.sh
RUN bash download-pgcat.sh
ENTRYPOINT ["bash", "run.sh"]
