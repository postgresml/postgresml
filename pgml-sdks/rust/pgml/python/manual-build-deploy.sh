#!/bin/bash

echo "Make sure and set the environment variable MATURIN_PYPI_TOKEN to your PyPI token."


cd ..
rm -r ../../python/pgml/dist/
mkdir ../../python/pgml/dist/
maturin build --release --strip -i python3.8 -i python3.9 -i python3.10 -i python3.11 -o ../../python/pgml/dist
cd ../../python/pgml
maturin upload --skip-existing dist/*
