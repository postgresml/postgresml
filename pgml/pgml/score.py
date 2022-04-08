"""Score"""

import os
import pickle

from pgml.exceptions import PgMLException


def load(name, source):
    """Load a model from file."""
    path = os.path.join(source, name)

    if not os.path.exists(path):
        raise PgMLException(f"Model source directory `{path}` does not exist.")

    with open(path, "rb") as f:
        return pickle.load(f)
