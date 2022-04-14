import plpy
from sklearn.linear_model import LinearRegression
from sklearn.ensemble import RandomForestRegressor, RandomForestClassifier
from sklearn.model_selection import train_test_split
from sklearn.metrics import mean_squared_error, r2_score

import pickle

from pgml.exceptions import PgMLException
from pgml.sql import q

class Project(object):
    _cache = {}

    @classmethod
    def find(cls, id):
        result = plpy.execute(f"""
            SELECT * 
            FROM pgml.projects 
            WHERE id = {q(id)}
        """, 1)
        if (result.nrows == 0):
            return None

        project = Project()
        project.__dict__ = dict(result[0])
        project.__init__()
        cls._cache[project.name] = project
        return project

    @classmethod
    def find_by_name(cls, name):
        if name in cls._cache:
            return cls._cache[name]
    
        result = plpy.execute(f"""
            SELECT * 
            FROM pgml.projects 
            WHERE name = {q(name)}
        """, 1)
        if (result.nrows == 0):
            return None

        project = Project()
        project.__dict__ = dict(result[0])
        project.__init__()
        cls._cache[name] = project
        return project

    @classmethod
    def create(cls, name, objective):
        project = Project()
        project.__dict__ = dict(plpy.execute(f"""
            INSERT INTO pgml.projects (name, objective) 
            VALUES ({q(name)}, {q(objective)}) 
            RETURNING *
        """, 1)[0])
        project.__init__()
        cls._cache[name] = project
        return project

    def __init__(self):
        self._deployed_model = None

    @property
    def deployed_model(self):
        if self._deployed_model is None:
            self._deployed_model = Model.find_deployed(self.id)
        return self._deployed_model

class Snapshot(object):
    @classmethod
    def create(cls, relation_name, y_column_name, test_size, test_sampling):
        snapshot = Snapshot()
        snapshot.__dict__ = dict(plpy.execute(f"""
            INSERT INTO pgml.snapshots (relation_name, y_column_name, test_size, test_sampling, status)
            VALUES ({q(relation_name)}, {q(y_column_name)}, {q(test_size)}, {q(test_sampling)}, 'new')
            RETURNING *
        """, 1)[0])
        plpy.execute(f"""
            CREATE TABLE pgml."snapshot_{snapshot.id}" AS 
            SELECT * FROM "{snapshot.relation_name}";
        """)
        snapshot.__dict__ = dict(plpy.execute(f"""
            UPDATE pgml.snapshots 
            SET status = 'created' 
            WHERE id = {q(snapshot.id)} 
            RETURNING *
        """)[0])
        return snapshot

    def data(self):
        data = plpy.execute(f"""
            SELECT * 
            FROM pgml."snapshot_{self.id}"
        """)

        # Sanity check the data
        if data.nrows == 0:
            PgMLException(
                f"Relation `{self.relation_name}` contains no rows. Did you pass the correct `relation_name`?"
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

class Model(object):
    @classmethod
    def create(cls, project, snapshot, algorithm_name):
        result = plpy.execute(f"""
            INSERT INTO pgml.models (project_id, snapshot_id, algorithm_name, status) 
            VALUES ({q(project.id)}, {q(snapshot.id)}, {q(algorithm_name)}, 'training') 
            RETURNING *
        """)
        model = Model()
        model.__dict__ = dict(result[0])
        model.__init__()
        model._project = project
        return model

    @classmethod
    def find_deployed(cls, project_id):
        result = plpy.execute(f"""
            SELECT models.* 
            FROM pgml.models 
            JOIN pgml.deployments 
              ON deployments.model_id = models.id
              AND deployments.project_id = {q(project_id)}
            ORDER by deployments.created_at DESC
            LIMIT 1
        """)
        if (result.nrows == 0):
            return None

        model = Model()
        model.__dict__ = dict(result[0])
        model.__init__()
        return model

    def __init__(self):
        self._algorithm = None
        self._project = None

    @property
    def project(self):
        if self._project is None:
            self._project = Project.find(self.project_id)
        return self._project

    @property
    def algorithm(self):
        if self._algorithm is None:
            if self.pickle is not None:
                self._algorithm = pickle.loads(self.pickle)
            else:
                self._algorithm = {
                    'linear_regression': LinearRegression,
                    'random_forest_regression': RandomForestRegressor,
                    'random_forest_classification': RandomForestClassifier
                }[self.algorithm_name + '_' + self.project.objective]()
    
        return self._algorithm

    def fit(self, snapshot):
        X_train, X_test, y_train, y_test = snapshot.data()

        # Train the model
        self.algorithm.fit(X_train, y_train)

        # Test
        y_pred = self.algorithm.predict(X_test)
        msq = mean_squared_error(y_test, y_pred)
        r2 = r2_score(y_test, y_pred)

        # Save the model
        self.__dict__ = dict(plpy.execute(f"""
            UPDATE pgml.models
            SET pickle = '\\x{pickle.dumps(self.algorithm).hex()}',
                status = 'successful',
                mean_squared_error = {q(msq)},
                r2_score = {q(r2)}
            WHERE id = {q(self.id)}
            RETURNING *
        """)[0])

    def deploy(self):
        plpy.execute(f"""
            INSERT INTO pgml.deployments (project_id, model_id) 
            VALUES ({q(self.project_id)}, {q(self.id)})
        """)

    def predict(self, data):
        return self.algorithm.predict(data)


def train(
    project_name: str, 
    objective: str,
    relation_name: str, 
    y_column_name: str, 
    test_size: float or int = 0.1,
    test_sampling: str = "random"
) -> None:
    """Create a regression model from a table or view filled with training data.

    Args:
        project_name (str): a human friendly identifier
        objective (str): Defaults to "regression". Valid values are ["regression", "classification"].
        relation_name (str): the table or view that stores the training data
        y_column_name (str): the column in the training data that acts as the label
        algorithm (str, optional): the algorithm used to implement the objective. Defaults to "linear". Valid values are ["linear", "random_forest"].
        test_size (float or int, optional): If float, should be between 0.0 and 1.0 and represent the proportion of the dataset to include in the test split. If int, represents the absolute number of test samples. If None, the value is set to the complement of the train size. If train_size is also None, it will be set to 0.25.
        test_sampling: (str, optional): How to sample to create the test data. Defaults to "random". Valid values are ["first", "last", "random"].
    """
    project = Project.create(project_name, objective)
    snapshot = Snapshot.create(relation_name, y_column_name, test_size, test_sampling)
    best_model = None
    best_error = None
    if objective == "regression":
        algorithms = ["linear", "random_forest"]
    elif objective == "classification":
        algorithms = ["random_forest"]

    for algorithm_name in algorithms:
        model = Model.create(project, snapshot, algorithm_name)
        model.fit(snapshot)
        if best_error is None or model.mean_squared_error < best_error:
            best_error = model.mean_squared_error
            best_model = model        
    best_model.deploy()
