#!/bin/bash
#
#
#
set -ex

docker run --privileged --rm tonistiigi/binfmt --install all
docker buildx create --use --name mybuilder || true
docker buildx build  \
	--platform linux/amd64,linux/arm64 \
	--tag ghcr.io/postgresml/pgml-rds-proxy:latest \
	--progress plain \
	--no-cache \
	--push \
	.
