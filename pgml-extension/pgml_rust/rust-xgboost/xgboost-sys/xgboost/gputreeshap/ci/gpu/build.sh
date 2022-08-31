#!/bin/bash
# Copyright (c) 2020, NVIDIA CORPORATION.
#########################################
# GPUTreeShap GPU build and test script for CI #
#########################################

set -e
NUMARGS=$#
ARGS=$*

# Set path and build parallel level
export PATH=/usr/local/cuda/bin:$PATH
export PARALLEL_LEVEL=${PARALLEL_LEVEL:-4}
export CUDA_REL=${CUDA_VERSION%.*}

# Set home to the job's workspace
export HOME=$WORKSPACE

# Install gpuCI tools
curl -s https://raw.githubusercontent.com/rapidsai/gpuci-tools/main/install.sh | bash
source ~/.bashrc
cd ~

################################################################################
# SETUP - Check environment
################################################################################

gpuci_logger "Install cmake"
mkdir cmake
cd cmake
wget https://github.com/Kitware/CMake/releases/download/v3.18.2/cmake-3.18.2-Linux-x86_64.sh
sh cmake-3.18.2-Linux-x86_64.sh --skip-license
export PATH=$PATH:$PWD/bin
cd ..

gpuci_logger "Install gtest"
wget https://github.com/google/googletest/archive/release-1.10.0.zip
unzip release-1.10.0.zip
mv googletest-release-1.10.0 gtest && cd gtest
cmake . && make
cp -r googletest/include include
export GTEST_ROOT=$PWD
cd ..

gpuci_logger "Check environment"
env


gpuci_logger "Check GPU usage"
nvidia-smi

$CC --version
$CXX --version


################################################################################
# BUILD - Build tests
################################################################################

gpuci_logger "Build C++ targets"
mkdir $WORKSPACE/build
cd $WORKSPACE/build
cmake .. -DBUILD_GTEST=ON -DBUILD_EXAMPLES=ON -DBUILD_BENCHMARKS=ON
make -j

################################################################################
# TEST - Run GoogleTest
################################################################################

gpuci_logger "GoogleTest"
cd $WORKSPACE/build
./TestGPUTreeShap

################################################################################
# Run example
################################################################################
gpuci_logger "Example"
cd $WORKSPACE/build
./GPUTreeShapExample
