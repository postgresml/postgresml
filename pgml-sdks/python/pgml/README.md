# PostgresML Python SDK
This Python SDK provides an easy interface to use PostgresML generative AI capabilities. 

## Table of Contents

- [Quickstart](#quickstart)

### Quickstart
1. Install Python 3.11. SDK should work for Python >=3.8. However, at this time, we have only tested Python 3.11.
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
5. SDK uses your local PostgresML database by default 
`postgres://postgres@127.0.0.1:5433/pgml_development`

If it is not up to date with `pgml.embed` please [signup for a free database](https://postgresml.org/signup) and set `PGML_CONNECTION` environment variable with serverless hosted database.

```
export PGML_CONNECTION="postgres://<username>:<password>@<hostname>:<port>/pgm<database>"
```
6. Run a **vector search** example
```
python examples/vector_search.py
```

