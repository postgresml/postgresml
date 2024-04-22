#!/bin/bash
#
#
#
set -ex

version="${1:-2.0.0-alpha18}"
image="ghcr.io/postgresml/pgml-rds-proxy"
image_with_version="$image:$version"

docker run --privileged --rm tonistiigi/binfmt --install all
docker buildx create --use --name mybuilder || true
docker buildx build  \
	--platform linux/amd64,linux/arm64 \
	--tag ${image_with_version} \
	--push \
	.
