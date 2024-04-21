FROM ubuntu:22.04
RUN apt update && \
	apt install -y curl postgresql-client-common postgresql-client-14 && \
	apt clean
WORKDIR /pgml-rds-proxy
COPY --chown=root:root build.sh build.sh
COPY --chown=root:root run.sh run.sh
RUN bash build.sh
ENTRYPOINT ["bash", "run.sh"]
