#!/bin/bash
#
# Install CUDA.
#

if [[ $(uname -a) == *"aarch64"* ]]; then
    ARCH="arm64"
else
    ARCH="amd64"
fi

# ARM
if [[ ${ARCH} == "arm64" ]]; then
    wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2204/sbsa/cuda-keyring_1.0-1_all.deb
fi

# Intel
if [[ ${ARCH} == "amd64" ]]; then
    wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2204/x86_64/cuda-keyring_1.0-1_all.deb
fi

dpkg -i cuda-keyring_1.0-1_all.deb
apt-get update
apt-get -y install cuda
