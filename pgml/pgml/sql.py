"""Tools to run SQL.
"""
import os
import plpy


def all_rows(cursor):
    """Fetch all rows from a plpy-like cursor."""
    while True:
        rows = cursor.fetch(5)
        if not rows:
            return

        for row in rows:
            yield row

