"""
Train the model.
"""

# TODO: import more models here
from sklearn.linear_model import LinearRegression
from sklearn.model_selection import train_test_split
from sklearn.metrics import mean_squared_error, r2_score

import pickle
import os

from pgml.sql import all_rows
from pgml.exceptions import PgMLException
from pgml.validate import check_type


def train(cursor, y_column, name, save=True, destination="/tmp/pgml_models"):
    """Train the model on data on some rows.

    Arguments:
            - cursor: iterable with rows,
            - y_column: the name of the column containing the y predicate (a.k.a solution),
            - name: the name of the model, e.g 'test_model',
            - save: to save the model to disk or not.

    Return:
            Path on disk where the model was saved or could be saved if saved=True.
    """
    X = []
    y = []
    columns = []

    for row in all_rows(cursor):
        row = row.copy()

        check_type(row)

        if y_column not in row:
            PgMLException(
                f"Column `{y}` not found. Did you name your `y_column` correctly?"
            )

        y_ = row.pop(y_column)
        x_ = []

        # Always pull the columns in the same order from the row.
        # Python dict iteration is not always in the same order (hash table).
        if not columns:
            for col in row:
                columns.append(col)

        for column in columns:
            x_.append(row[column])
        X.append(x_)
        y.append(y_)

    X_train, X_test, y_train, y_test = train_test_split(X, y)

    # Just linear regression for now, but can add many more later.
    lr = LinearRegression()
    lr.fit(X_train, y_train)

    # Test
    y_pred = lr.predict(X_test)
    msq = mean_squared_error(y_test, y_pred)
    r2 = r2_score(y_test, y_pred)

    path = os.path.join(destination, name)

    if save:
        with open(path, "wb") as f:
            pickle.dump(lr, f)

    return path, msq, r2
