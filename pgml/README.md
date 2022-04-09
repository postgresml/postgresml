# PgML - Postgres Machine Learning

This package implements data processing, training and scoring of ML models. It is built to be used
inside PL/Python functions, inside a Postgres connection process.

Most functions can work in any Python environment; some functions are tightly coupled to
PL/Python by requiring a `plpy` object passed in as an argument: it provides database access methods.

## Installation

For now, we require that this package is installed into the system Python packages space:

```bash
sudo pip3 setup.py install
```

## Usage

See `../sql/install.sql` for list of functions. This package is not meant to be used directly.
