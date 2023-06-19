#!/usr/bin/bash

cd ..
maturin build --release --strip -i python3.8 -i python3.9 -i python3.10 -i python3.11 -o ../../python/pgml/dist
cd ../../python/pgml
maturin upload --repository testpypi --skip-existing dist/*
