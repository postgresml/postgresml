import plpy
from sklearn.linear_model import LinearRegression
from sklearn.ensemble import RandomForestRegressor
from sklearn.model_selection import train_test_split
from sklearn.metrics import mean_squared_error, r2_score

import pickle

from pgml.exceptions import PgMLException

class Project:
    def __init__(self, name):
        # Find or create the project
        result = plpy.execute(f"SELECT * FROM pgml.projects WHERE name = '{name}'", 1)
        if (result.nrows == 1):
            self.__dict__ = dict(result[0])
        else:
            try: 
                self.__dict__ = dict(plpy.execute(f"INSERT INTO pgml.projects (name) VALUES ('{name}') RETURNING *", 1)[0])
            except Exception as e: # handle race condition to insert
                self.__dict__ = dict(plpy.execute(f"SELECT * FROM pgml.projects WHERE name = '{name}'", 1)[0])

class Snapshot:
    def __init__(self, relation_name, y_column_name, test_size, test_sampling):
        self.__dict__ = dict(plpy.execute(f"INSERT INTO pgml.snapshots (relation_name, y_column_name, test_size, test_sampling, status) VALUES ('{relation_name}', '{y_column_name}', {test_size}, '{test_sampling}', 'new') RETURNING *", 1)[0])
        plpy.execute(f"""CREATE TABLE pgml.snapshot_{self.id} AS SELECT * FROM "{relation_name}";""")
        self.__dict__ = dict(plpy.execute(f"UPDATE pgml.snapshots SET status = 'created' WHERE id = {self.id} RETURNING *")[0])

    def data(self):
        data = plpy.execute(f"SELECT * FROM pgml.snapshot_{self.id}")

        # Sanity check the data
        if data.nrows == 0:
            PgMLException(
                f"Relation `{self.y_column_name}` contains no rows. Did you pass the correct `relation_name`?"
            )
        if self.y_column_name not in data[0]:
            PgMLException(
                f"Column `{self.y_column_name}` not found. Did you pass the correct `y_column_name`?"
            )

        # Always pull the columns in the same order from the row.
        # Python dict iteration is not always in the same order (hash table).
        columns = list(data[0].keys())
        columns.remove(self.y_column_name)
        columns.sort()

        # Split the label from the features
        X = []
        y = []
        for row in data:
            y_ = row.pop(self.y_column_name)
            x_ = []

            for column in columns:
                x_.append(row[column])

            X.append(x_)
            y.append(y_)

        # Split into training and test sets
        if (self.test_sampling == 'random'):
            return train_test_split(X, y, test_size=self.test_size, random_state=0)
        else:
            if (self.test_sampling == 'first'):
                X.reverse()
                y.reverse()
                if isinstance(split, float):
                    split = 1.0 - split
            split = self.test_size
            if isinstance(split, float):
                split = int(self.test_size * X.len())
            return X[0:split], X[split:X.len()-1], y[0:split], y[split:y.len()-1]

        # TODO normalize and clean data


class Model:
    def __init__(self, project, snapshot, algorithm):
        self.__dict__ = dict(plpy.execute(f"INSERT INTO pgml.models (project_id, snapshot_id, algorithm, status) VALUES ({project.id}, {snapshot.id}, '{algorithm}', 'training') RETURNING *")[0])

    def fit(self, snapshot):
        X_train, X_test, y_train, y_test = snapshot.data()

        # Train the model
        algo = {
            'linear': LinearRegression,
            'random_forest': RandomForestRegressor
        }[self.algorithm]()
        algo.fit(X_train, y_train)

        # Test
        y_pred = algo.predict(X_test)
        msq = mean_squared_error(y_test, y_pred)
        r2 = r2_score(y_test, y_pred)

        # Save the model
        weights = pickle.dumps(algo)

        self.__dict__ = dict(plpy.execute(f"""
            UPDATE pgml.models
            SET pickle = '\\x{weights.hex()}',
                status = 'successful',
                mean_squared_error = '{msq}',
                r2_score = '{r2}'
            WHERE id = {self.id}
            RETURNING *
        """)[0])

class Regression:
    """Provides continuous real number predictions learned from the training data.
    """    
    def __init__(
        self,
        project_name: str, 
        relation_name: str, 
        y_column_name: str, 
        algorithms: str = ["linear", "random_forest"],
        test_size: float or int = 0.1,
        test_sampling: str = "random"
    ) -> None:
        """Create a regression model from a table or view filled with training data.

        Args:
            project_name (str): a human friendly identifier
            relation_name (str): the table or view that stores the training data
            y_column_name (str): the column in the training data that acts as the label
            algorithm (str, optional): the algorithm used to implement the regression. Defaults to "linear". Valid values are ["linear", "random_forest"].
            test_size (float or int, optional): If float, should be between 0.0 and 1.0 and represent the proportion of the dataset to include in the test split. If int, represents the absolute number of test samples. If None, the value is set to the complement of the train size. If train_size is also None, it will be set to 0.25.
            test_sampling: (str, optional): How to sample to create the test data. Defaults to "random". Valid values are ["first", "last", "random"].
        """
        project = Project(project_name)
        snapshot = Snapshot(relation_name, y_column_name, test_size, test_sampling)
        for algorithm in algorithms:
            model = Model(project, snapshot, algorithm)
            model.fit(snapshot)
        # TODO: promote the model?
