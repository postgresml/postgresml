#!/bin/bash
#
# Install CUDA.
#
set -e

UBUNTU_VERSION=$(lsb_release -a | grep Release | awk '{ print $2 }')

if [[ $(uname -a) == *"aarch64"* ]]; then
    ARCH="arm64"
else
    ARCH="amd64"
fi

# ARM
if [[ ${ARCH} == "arm64" ]]; then
    wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu${UBUNTU_VERSION//.}/sbsa/cuda-keyring_1.0-1_all.deb
fi

# Intel
if [[ ${ARCH} == "amd64" ]]; then
    wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu${UBUNTU_VERSION//.}/x86_64/cuda-keyring_1.0-1_all.deb
fi

dpkg -i cuda-keyring_1.0-1_all.deb
apt-get update
apt-get -y install cuda
