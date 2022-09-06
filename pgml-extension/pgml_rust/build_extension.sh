#!/bin/bash
#
# Build the extension.
#

echo "Building base image, this will take a little while"
docker build . --build-arg VERSION=${1:-"22.04"} --build-arg PGVERSION=${2:-"14"}

IMAGE_ID=$(docker images | awk '{print $3}' | awk 'NR==2')

docker run -v $(pwd):/output ${IMAGE_ID}
