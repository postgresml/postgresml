# PostgresML Python SDK
This Python SDK provides an easy interface to use PostgresML generative AI capabilities. 

## Table of Contents

- [Quickstart](#quickstart)

### Quickstart
1. Install Python 3.11. This package should work for Python >=3.8. However, at this time, only Python 3.11 has been tested.
2. Clone the repository and checkout the SDK branch (before PR)
```
git clone https://github.com/postgresml/postgresml
cd postgresml
git checkout santi-pgml-memory-sdk-python
cd pgml-sdks/python/pgml
```
3. Install poetry `pip install poetry`
4. Initialize Python environment

```
poetry env use python3.11
poetry shell
poetry install
poetry build
```
5. Run a **vector search** example
```
python examples/vector_search.py
```

