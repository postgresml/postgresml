FROM levkk/pgml-extension:latest

COPY --chown=postgres:postgres . /app
WORKDIR /app

# Install pgml extension globally.
RUN pip3 install .

WORKDIR /app
ENTRYPOINT ["/bin/bash", "/app/docker/entrypoint.sh"]

