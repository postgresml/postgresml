# PgML Extension - Postgres Machine Learning

This package implements data processing, training and scoring of ML models inside Postgres from PL/Python functions. It is dependent on the internal to Postgres `plpy` connection and data access methods, so can only be used via a Postgres connection through the APIs exposed as native Postgres functions.

## Installation

Postgres requires that this package is installed into the system Python packages space:

To install from pypi:

```bash
sudo pip3 install pgml-extension
```

To install from a clone of the repository:

```bash
sudo python3 setup.py install
```

## Usage

See `sql/install.sql` for list of functions exposed through SQL. This package is not meant to be used directly.
