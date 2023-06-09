#! /usr/bin/bash

# Remove old dist directory and create new one
mkdir ../../python/pgml-sdk/dist/
rm -r ../../python/pgml-sdk/dist/*

# Build wheel
maturin build -r -o ../../python/pgml-sdk/dist/

# Copy pyproject.toml over
rm ../../python/pgml-sdk/pyproject.toml
cp pyproject.toml ../../python/pgml-sdk/

echo "Wheel built and files moved over"
echo "Uploading to pypi"

# Do the upload
maturin upload --repository testpypi ../../python/pgml-sdk/dist/*
