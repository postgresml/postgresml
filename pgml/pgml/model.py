from cmath import e
import plpy

from sklearn.linear_model import LinearRegression
from sklearn.model_selection import train_test_split
from sklearn.metrics import mean_squared_error, r2_score

import pickle

from pgml.exceptions import PgMLException

class Regression:
    """Provides continuous real number predictions learned from the training data.
    """    
    def __init__(
        self,
        project_name: str, 
        relation_name: str, 
        y_column_name: str, 
        algorithm: str = "sklearn.linear_model",
        test_size: float or int = 0.1,
        test_sampling: str = "random"
    ) -> None:
        """Create a regression model from a table or view filled with training data.

        Args:
            project_name (str): a human friendly identifier
            relation_name (str): the table or view that stores the training data
            y_column_name (str): the column in the training data that acts as the label
            algorithm (str, optional): the algorithm used to implement the regression. Defaults to "sklearn.linear_model".
            test_size (float or int, optional): If float, should be between 0.0 and 1.0 and represent the proportion of the dataset to include in the test split. If int, represents the absolute number of test samples. If None, the value is set to the complement of the train size. If train_size is also None, it will be set to 0.25.
            test_sampling: (str, optional): How to sample to create the test data. Defaults to "random". Valid values are ["first", "last", "random"].
        """

        plpy.warning("snapshot")
        # Create a snapshot of the relation
        snapshot = plpy.execute(f"INSERT INTO pgml.snapshots (relation, y, test_size, test_sampling, status) VALUES ('{relation_name}', '{y_column_name}', {test_size}, '{test_sampling}', 'new') RETURNING *", 1)[0]
        plpy.execute(f"""CREATE TABLE pgml.snapshot_{snapshot['id']} AS SELECT * FROM "{relation_name}";""")
        plpy.execute(f"UPDATE pgml.snapshots SET status = 'created' WHERE id = {snapshot['id']}")

        plpy.warning("project")
        # Find or create the project
        project = plpy.execute(f"SELECT * FROM pgml.projects WHERE name = '{project_name}'", 1)
        plpy.warning(f"project {project}")
        if (project.nrows == 1):
            plpy.warning("project found")
            project = project[0]
        else:
            try: 
                project = plpy.execute(f"INSERT INTO pgml.projects (name) VALUES ('{project_name}') RETURNING *", 1)
                plpy.warning(f"project inserted {project}")
                if (project.nrows() == 1):
                    project = project[0]

            except Exception as e: # handle race condition to insert
                plpy.warning(f"project retry: #{e}")
                project = plpy.execute(f"SELECT * FROM pgml.projects WHERE name = '{project_name}'", 1)[0]

        plpy.warning("model")
        # Create the model
        model = plpy.execute(f"INSERT INTO pgml.models (project_id, snapshot_id, algorithm, status) VALUES ({project['id']}, {snapshot['id']}, '{algorithm}', 'training') RETURNING *")[0]

        plpy.warning("data")
        # Prepare the data
        data = plpy.execute(f"SELECT * FROM pgml.snapshot_{snapshot['id']}")

        # Sanity check the data
        if data.nrows == 0:
            PgMLException(
                f"Relation `{y_column_name}` contains no rows. Did you pass the correct `relation_name`?"
            )
        if y_column_name not in data[0]:
            PgMLException(
                f"Column `{y_column_name}` not found. Did you pass the correct `y_column_name`?"
            )

        # Always pull the columns in the same order from the row.
        # Python dict iteration is not always in the same order (hash table).
        columns = []
        for col in data[0]:
            if col != y_column_name:
                columns.append(col)

        # Split the label from the features
        X = []
        y = []
        for row in data:
            plpy.warning(f"row: {row}")
            y_ = row.pop(y_column_name)
            x_ = []

            for column in columns:
                x_.append(row[column])

            X.append(x_)
            y.append(y_)

        # Split into training and test sets
        plpy.warning("split")
        if (test_sampling == 'random'):
            X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=test_size, random_state=0)
        else:
            if (test_sampling == 'first'):
                X.reverse()
                y.reverse()
                if isinstance(split, float):
                    split = 1.0 - split
            split = test_size
            if isinstance(split, float):
                split = int(test_size * X.len())
            X_train, X_test, y_train, y_test = X[0:split], X[split:X.len()-1], y[0:split], y[split:y.len()-1]

        # TODO normalize and clean data

        plpy.warning("train")
        # Train the model
        algo = LinearRegression()
        algo.fit(X_train, y_train)

        plpy.warning("test")
        # Test
        y_pred = algo.predict(X_test)
        msq = mean_squared_error(y_test, y_pred)
        r2 = r2_score(y_test, y_pred)

        plpy.warning("save")
        # Save the model
        weights = pickle.dumps(algo)

        plpy.execute(f"""
            UPDATE pgml.models
            SET pickle = '\\x{weights.hex()}',
                status = 'successful',
                mean_squared_error = '{msq}',
                r2_score = '{r2}'
            WHERE id = {model['id']}
        """)

        # TODO: promote the model?
