#! /usr/bin/bash

# Remove old dist directory and create new one
mkdir ../../python/pgml/dist/
rm -r ../../python/pgml/dist/*

# Build wheel
maturin build -r -o ../../python/pgml/dist/

# Copy pyproject.toml over
rm ../../python/pgml/pyproject.toml
cp pyproject.toml ../../python/pgml/

echo "Wheel built and files moved over"
echo "Uploading to pypi"

# Do the upload
maturin upload --repository testpypi ../../python/pgml/dist/*
