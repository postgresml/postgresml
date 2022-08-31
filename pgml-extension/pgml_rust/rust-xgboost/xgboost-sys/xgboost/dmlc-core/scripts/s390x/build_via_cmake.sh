#!/bin/bash

set -ex

rm -rf build
mkdir -p build
cd build
cmake .. -DGOOGLE_TEST=ON -DCMAKE_VERBOSE_MAKEFILE=ON
make -j$(nproc)
