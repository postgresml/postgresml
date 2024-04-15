#!/bin/bash

echo "Make sure and set the environment variable MATURIN_PYPI_TOKEN to your PyPI token."

cd ..
PYTHON_STUB_FILE="python/pgml/pgml.pyi" maturin publish -r $1 -i python3.8 -i python3.9 -i python3.10 -i python3.11 -i python3.12 --skip-existing -F python
