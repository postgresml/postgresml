"""
Run some basic sanity checks on the data.
"""

# import sklearn
from pgml.exceptions import PgMLException


def check_type(row):
    """We only accept certain column types for now."""
    for col in row:
        if type(row[col]) not in (int, float):
            raise PgMLException(f"Column '{col}' is not a integer or float.")
