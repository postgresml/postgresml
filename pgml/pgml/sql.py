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


def models_directory():
    """Get the directory where we store our models."""
    data_directory = plpy.execute(
        """
        SELECT setting FROM pg_settings WHERE name = 'data_directory'
    """,
        1,
    )[0]["setting"]

    models_dir = os.path.join(data_directory, "pgml_models")

    # TODO: Ideally this happens during extension installation.
    if not os.path.exists(models_dir):
        os.mkdir(models_dir, 0o770)

    return models_dir
