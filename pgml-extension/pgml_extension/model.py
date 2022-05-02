from re import M
import plpy
import sklearn.linear_model
import sklearn.kernel_ridge
import sklearn.svm
import sklearn.ensemble
import sklearn.multioutput
import sklearn.gaussian_process
import xgboost as xgb
import diptest
from sklearn.model_selection import train_test_split
from sklearn.metrics import (
    mean_squared_error,
    r2_score,
    f1_score,
    precision_score,
    recall_score,
)

import pickle
import json

from pgml_extension.exceptions import PgMLException
from pgml_extension.sql import q


def flatten(S):
    if S == []:
        return S
    if isinstance(S[0], list):
        return flatten(S[0]) + flatten(S[1:])
    return S[:1] + flatten(S[1:])


class Project(object):
    """
    Use projects to refine multiple models of a particular dataset on a specific objective.

    Attributes:
        id (int): a unique identifier
        name (str): a human friendly unique identifier
        objective (str): the purpose of this project
        created_at (Timestamp): when this project was created
        updated_at (Timestamp): when this project was last updated
    """

    def __init__(self):
        self._deployed_model = None

    @classmethod
    def find(cls, id: int):
        """
        Get a Project from the database.

        Args:
            id (int): the project id
        Returns:
            Project or None: instantiated from the database if found
        """
        result = plpy.execute(
            f"""
                SELECT * 
                FROM pgml.projects 
                WHERE id = {q(id)}
            """,
            1,
        )
        if len(result) == 0:
            return None

        project = Project()
        project.__dict__ = dict(result[0])
        project.__init__()
        return project

    @classmethod
    def find_by_name(cls, name: str):
        """
        Get a Project from the database by name.

        This is the prefered API to retrieve projects, and they are cached by
        name to avoid needing to go to he database on every usage.

        Args:
            name (str): the project name
        Returns:
            Project or None: instantiated from the database if found
        """

        # bust the cache after a deployment
        result = plpy.execute(
            f"""
                SELECT * 
                FROM pgml.projects 
                WHERE name = {q(name)}
            """,
            1,
        )
        if len(result) == 0:
            raise PgMLException(f"Project '{name}' does not exist.")

        project = Project()
        project.__dict__ = dict(result[0])
        project.__init__()
        return project

    @classmethod
    def create(cls, name: str, objective: str):
        """
        Create a Project and save it to the database.

        Args:
            name (str): a human friendly identifier
            objective (str): valid values are ["regression", "classification"].
        Returns:
            Project: instantiated from the database
        """
        if objective is None:
            raise PgMLException(f"You must specify and objective when creating a new Project.")
        project = Project()
        project.__dict__ = dict(
            plpy.execute(
                f"""
                    INSERT INTO pgml.projects (name, objective, created_at, updated_at) 
                    VALUES ({q(name)}, {q(objective)}, clock_timestamp(), clock_timestamp()) 
                    RETURNING *
                """,
                1,
            )[0]
        )
        project.__init__()
        return project

    @property
    def deployed_model(self):
        """
        Returns:
            Model: that should currently be used for predictions
        """
        if self._deployed_model is None:
            self._deployed_model = Model.find_deployed(self.id)
        return self._deployed_model

    def deploy(self, qualifier="best_score", algorithm_name=None):
        model = Model.find_by_project_and_qualifier_algorithm_name(self, qualifier, algorithm_name)
        if model and model.id != self.deployed_model.id:
            model.deploy(qualifier)
        return model

    @property
    def last_snapshot(self):
        return Snapshot.last_for_project_id(self.id)


class Snapshot(object):
    """
    Snapshots capture a set of training & test data for repeatability.

    Attributes:
        id (int): a unique identifier
        relation_name (str): the name of the table or view to snapshot
        y_column_name (str): the label for training data
        test_size (float or int, optional): If float, should be between 0.0 and 1.0 and represent the proportion of the dataset to include in the test split. If int, represents the absolute number of test samples. If None, the value is set to the complement of the train size. If train_size is also None, it will be set to 0.25.
        test_sampling (str, optional): How to sample to create the test data. Defaults to "random". Valid values are ["first", "last", "random"].
        status (str): The current status of the snapshot, e.g. 'new' or 'created'
        created_at (Timestamp): when this snapshot was created
        updated_at (Timestamp): when this snapshot was last updated
    """

    @classmethod
    def create(
        cls,
        relation_name: str,
        y_column_name: list,
        test_size: float or int,
        test_sampling: str,
    ):
        """
        Create a Snapshot and save it to the database.

        This creates both a metadata record in the snapshots table, as well as creating a new table
        that holds a snapshot of all the data currently present in the relation so that training
        runs may be repeated, or further analysis may be conducted against the input.

        Args:
            relation_name (str): the name of the table or view to snapshot
            y_column_name (str): the label for training data
            test_size (float or int, optional): If float, should be between 0.0 and 1.0 and represent the proportion of the dataset to include in the test split. If int, represents the absolute number of test samples. If None, the value is set to the complement of the train size. If train_size is also None, it will be set to 0.25.
            test_sampling: (str, optional): How to sample to create the test data. Defaults to "random". Valid values are ["first", "last", "random"].
        Returns:
            Snapshot: metadata instantiated from the database
        """

        snapshot = Snapshot()
        snapshot.__dict__ = dict(
            plpy.execute(
                f"""
            INSERT INTO pgml.snapshots (relation_name, y_column_name, test_size, test_sampling, status, created_at, updated_at)
            VALUES ({q(relation_name)}, ARRAY[{",".join([q(name) for name in y_column_name])}], {q(test_size)}, {q(test_sampling)}, 'new', clock_timestamp(), clock_timestamp())
            RETURNING *
        """,
                1,
            )[0]
        )
        plpy.execute(
            f"""
            CREATE TABLE pgml."snapshot_{snapshot.id}" AS 
            SELECT * FROM {snapshot.relation_name}
            ORDER BY random();
        """
        )
        snapshot.analyze()
        snapshot.__dict__ = dict(
            plpy.execute(
                f"""
            UPDATE pgml.snapshots 
            SET status = 'created', updated_at = clock_timestamp()
            WHERE id = {q(snapshot.id)} 
            RETURNING *
        """,
                1,
            )[0]
        )
        return snapshot

    @classmethod
    def find(cls, id):
        result = plpy.execute(
            f"""
            SELECT snapshots.* 
            FROM pgml.snapshots 
            WHERE snapshots.id = {q(id)}
            LIMIT 1
        """
        )
        if len(result) == 0:
            return None

        snapshot = Snapshot()
        snapshot.__dict__ = dict(result[0])
        snapshot.__init__()
        return snapshot

    @classmethod
    def last_for_project_id(cls, project_id):
        result = plpy.execute(
            f"""
            SELECT snapshots.* 
            FROM pgml.snapshots 
            JOIN pgml.models
              ON models.snapshot_id = snapshots.id
              AND models.project_id = {q(project_id)}
            ORDER by snapshots.created_at DESC
            LIMIT 1
        """
        )
        if len(result) == 0:
            return None

        snapshot = Snapshot()
        snapshot.__dict__ = dict(result[0])
        snapshot.__init__()
        return snapshot

    def analyze(self):
        sample = plpy.execute(
            f"""
            SELECT * 
            FROM pgml."snapshot_{self.id}"
            LIMIT 1000
        """
        )
        # Sanity check the data
        if len(sample) == 0:
            raise PgMLException(
                f"Relation `{self.relation_name}` contains no rows. Did you pass the correct `relation_name`?"
            )
        
        for column in self.y_column_name:
            if not column in sample[0]:
                raise PgMLException(f"Column `{column}` not found. Did you pass the correct `y_column_name`?")

        values = ["count(*) AS samples"]
        for (column, value) in dict(sample[0]).items():
            if isinstance(value, float) or isinstance(value, int):
                values.append(
                    f"\n  min({column})::FLOAT4 AS {column}_min, max({column})::FLOAT4 AS {column}_max, avg({column})::FLOAT4 AS {column}_mean, stddev({column})::FLOAT4 AS {column}_stddev, percentile_disc(0.25) within group (order by {column}) AS {column}_p25, percentile_disc(0.5) within group (order by {column}) as {column}_p50, percentile_disc(0.75) within group (order by {column}) as {column}_p75, count({column})::INT AS {column}_count, count(distinct {column})::INT AS {column}_distinct, sum(({column} IS NULL)::int)::INT AS {column}_nulls"
                )
        self.analysis = dict(plpy.execute(f"SELECT {','.join(values)} FROM {self.relation_name}")[0])

        for (column, value) in dict(sample[0]).items():
            if isinstance(value, float) or isinstance(value, int):
                data = [row[column] for row in sample]
                self.analysis[f"{column}_dip"] = diptest.dipstat(data)

        self.columns = {}
        parts = self.relation_name.split(".")
        sql = "SELECT column_name, data_type FROM information_schema.columns "
        if len(parts) > 1:
            sql += f"WHERE table_name = {q(parts[1])} and table_schema = {q(parts[0])}"
        else:
            sql += f"WHERE table_name = {q(parts[0])}"
        result = plpy.execute(sql)
        for row in result:
            self.columns[row["column_name"]] = row["data_type"]

        self.__dict__ = dict(
            plpy.execute(
                f"""
            UPDATE pgml.snapshots 
            SET columns = {q(json.dumps(self.columns))}::JSON, analysis = {q(json.dumps(self.analysis))}::JSON, updated_at = clock_timestamp()
            WHERE id = {q(self.id)} 
            RETURNING *
        """,
                1,
            )[0]
        )

    def data(self):
        """
        Returns:
            list, list, list, list: All rows from the snapshot split into X_train, X_test, y_train, y_test sets.
        """
        data = plpy.execute(
            f"""
            SELECT * 
            FROM pgml."snapshot_{self.id}"
        """
        )

        # Always pull the columns in the same order from the row.
        # Python dict iteration is not always in the same order (hash table).
        features = list(data[0].keys())
        for column in self.y_column_name:
            features.remove(column)
        features.sort()

        # Split the label from the features
        X = []
        y = []
        for row in data:
            y_ = []
            for column in self.y_column_name:
                y_.append(row.pop(column))

            x_ = []
            for feature in features:
                x_.append(row[feature])

            x_ = flatten(x_)  # TODO be smart about flattening X depending on algorithm
            X.append(x_)
            y.append(y_)

        # Split into training and test sets
        if self.test_sampling == "random":
            return train_test_split(X, y, test_size=self.test_size, random_state=0)
        else:
            if self.test_sampling == "first":
                X.reverse()
                y.reverse()
                if isinstance(split, float):
                    split = 1.0 - split
            split = self.test_size
            if isinstance(split, float):
                split = int(self.test_size * X.len())
            return X[:split], X[split:], y[:split], y[split:]

        # TODO normalize and clean data


class Model(object):
    """Models use an algorithm on a snapshot of data to record the parameters learned.

    Attributes:
        project (str): the project the model belongs to
        snapshot (str): the snapshot that provides the training and test data
        algorithm_name (str): the name of the algorithm used to train this model
        status (str): The current status of the model, e.g. 'new', 'training' or 'successful'
        created_at (Timestamp): when this model was created
        updated_at (Timestamp): when this model was last updated
        metrics (dict): key performance indicators for the model
        pickle (bytes): the serialized version of the model parameters
        algorithm: the in memory version of the model parameters that can make predictions
    """

    @classmethod
    def create(
        cls,
        project: Project,
        snapshot: Snapshot,
        algorithm_name: str,
        hyperparams: dict,
    ):
        """
        Create a Model and save it to the database.

        Args:
            project (str):
            snapshot (str):
            algorithm_name (str):
        Returns:
            Model: instantiated from the database
        """
        result = plpy.execute(
            f"""
            INSERT INTO pgml.models (project_id, snapshot_id, algorithm_name, hyperparams, status, created_at, updated_at) 
            VALUES ({q(project.id)}, {q(snapshot.id)}, {q(algorithm_name)}, {q(json.dumps(hyperparams))}, 'new', clock_timestamp(), clock_timestamp()) 
            RETURNING *
        """
        )
        model = Model()
        model.__dict__ = dict(result[0])
        model.__init__()
        model._snapshot = snapshot
        model._project = project
        return model

    @classmethod
    def find_deployed(cls, project_id: int):
        """
        Args:
            project_id (int): The project id
        Returns:
            Model: that should currently be used for predictions of the project
        """
        result = plpy.execute(
            f"""
            SELECT models.* 
            FROM pgml.models 
            JOIN pgml.deployments 
              ON deployments.model_id = models.id
              AND deployments.project_id = {q(project_id)}
            ORDER by deployments.created_at DESC
            LIMIT 1
        """
        )
        if len(result) == 0:
            return None

        model = Model()
        model.__dict__ = dict(result[0])
        model.__init__()
        return model

    @classmethod
    def find_by_project_and_qualifier_algorithm_name(cls, project: Project, strategy: str, algorithm_name: str):
        """
        Args:
            project_id (int): The project id
            strategy (str): The strategy
            algorithm_name (str): The algorithm
        Returns:
            Model: most recently created model that fits the criteria
        """

        sql = f"""
            SELECT models.* 
            FROM pgml.models 
        """
        where = f"\nWHERE models.project_id = {q(project.id)}"
        if algorithm_name is not None:
            where += f"\nAND algorithm_name = {q(algorithm_name)}"

        if strategy == "best_score":
            if project.objective == "regression":
                sql += f"{where}\nORDER BY models.metrics->>'mean_squared_error' DESC NULLS LAST"
            elif project.objective == "classification":
                sql += f"{where}\nORDER BY models.metrics->>'f1' ASC NULLS LAST"
        elif strategy == "most_recent":
            sql += f"{where}\nORDER by models.created_at DESC"
        elif strategy == "rollback":
            sql += f"""
                JOIN pgml.deployments ON deployments.project_id = {q(project.id)} 
                    AND deployments.model_id = models.id
                    AND models.id != {q(project.deployed_model.id)}
                {where}
                ORDER by deployments.created_at DESC
            """
        else:
            raise PgMLException(f"unknown strategy: {strategy}")
        sql += "\nLIMIT 1"

        result = plpy.execute(sql)
        if len(result) == 0:
            return None

        model = Model()
        model.__dict__ = dict(result[0])
        model.__init__()
        return model

    @classmethod
    def find_by_project_and_best_score(cls, project: Project):
        """
        Args:
            project (Project): The project
        Returns:
            Model: the model with the best metrics for the project
        """
        if project.objective == "regression":
            metric = "mean_squared_error"
        elif project.objective == "classification":
            metric = "f1"

        result = plpy.execute(
            f"""
            SELECT models.* 
            FROM pgml.models 
            WHERE project_id = {q(project.id)}
            ORDER by models.metrics->>{q(metric)} DESC
            LIMIT 1
        """
        )
        if len(result) == 0:
            return None

        model = Model()
        model.__dict__ = dict(result[0])
        model.__init__()
        return model

    def __init__(self):
        self._algorithm = None
        self._project = None
        self._snapshot = None
        if "hyperparams" in self.__dict__ and type(self.hyperparams) is str:
            self.hyperparams = json.loads(self.hyperparams)
        if "metrics" in self.__dict__ and type(self.metrics) is str:
            self.metrics = json.loads(self.metrics)

    @property
    def project(self):
        """
        Returns:
            Project: that this model belongs to
        """
        if self._project is None:
            self._project = Project.find(self.project_id)
        return self._project

    @property
    def snapshot(self):
        """
        Returns:
            Snapshot: that this model trains with
        """
        if self._snapshot is None:
            self._snapshot = Snapshot.find(self.project_id)
        return self._snapshot

    @property
    def algorithm(self):
        if self._algorithm is None:
            if self.pickle is not None:
                self._algorithm = pickle.loads(self.pickle)
            else:
                self._algorithm = {
                    "linear_regression": sklearn.linear_model.LinearRegression,
                    "linear_classification": sklearn.linear_model.LogisticRegression,
                    "ridge_regression": sklearn.linear_model.Ridge,
                    "ridge_classification": sklearn.linear_model.RidgeClassifier,
                    "lasso_regression": sklearn.linear_model.Lasso,
                    "elastic_net_regression": sklearn.linear_model.ElasticNet,
                    "least_angle_regression": sklearn.linear_model.Lars,
                    "lasso_least_angle_regression": sklearn.linear_model.LassoLars,
                    "orthoganl_matching_pursuit_regression": sklearn.linear_model.OrthogonalMatchingPursuit,
                    "bayesian_ridge_regression": sklearn.linear_model.BayesianRidge,
                    "automatic_relevance_determination_regression": sklearn.linear_model.ARDRegression,
                    "stochastic_gradient_descent_regression": sklearn.linear_model.SGDRegressor,
                    "stochastic_gradient_descent_classification": sklearn.linear_model.SGDClassifier,
                    "perceptron_classification": sklearn.linear_model.Perceptron,
                    "passive_aggressive_regression": sklearn.linear_model.PassiveAggressiveRegressor,
                    "passive_aggressive_classification": sklearn.linear_model.PassiveAggressiveClassifier,
                    "ransac_regression": sklearn.linear_model.RANSACRegressor,
                    "theil_sen_regression": sklearn.linear_model.TheilSenRegressor,
                    "huber_regression": sklearn.linear_model.HuberRegressor,
                    "quantile_regression": sklearn.linear_model.QuantileRegressor,
                    "kernel_ridge_regression": sklearn.kernel_ridge.KernelRidge,
                    "gaussian_process_regression": sklearn.gaussian_process.GaussianProcessRegressor,
                    "gaussian_process_classification": sklearn.gaussian_process.GaussianProcessClassifier,
                    "svm_regression": sklearn.svm.SVR,
                    "svm_classification": sklearn.svm.SVC,
                    "nu_svm_regression": sklearn.svm.NuSVR,
                    "nu_svm_classification": sklearn.svm.NuSVC,
                    "linear_svm_regression": sklearn.svm.LinearSVR,
                    "linear_svm_classification": sklearn.svm.LinearSVC,
                    "ada_boost_regression": sklearn.ensemble.AdaBoostRegressor,
                    "ada_boost_classification": sklearn.ensemble.AdaBoostClassifier,
                    "bagging_regression": sklearn.ensemble.BaggingRegressor,
                    "bagging_classification": sklearn.ensemble.BaggingClassifier,
                    "extra_trees_regression": sklearn.ensemble.ExtraTreesRegressor,
                    "extra_trees_classification": sklearn.ensemble.ExtraTreesClassifier,
                    "gradient_boosting_trees_regression": sklearn.ensemble.GradientBoostingRegressor,
                    "gradient_boosting_trees_classification": sklearn.ensemble.GradientBoostingClassifier,
                    "hist_gradient_boosting_regression": sklearn.ensemble.HistGradientBoostingRegressor,
                    "hist_gradient_boosting_classification": sklearn.ensemble.HistGradientBoostingClassifier,
                    "random_forest_regression": sklearn.ensemble.RandomForestRegressor,
                    "random_forest_classification": sklearn.ensemble.RandomForestClassifier,
                    "xgboost_regression": xgb.XGBRegressor,
                    "xgboost_classification": xgb.XGBClassifier,
                }[self.algorithm_name + "_" + self.project.objective](**self.hyperparams)
                if len(self.snapshot.y_column_name) > 0:
                    if self.project.objective == "regression" and self.algorithm_name in [
                        "bayesian_ridge",
                        "automatic_relevance_determination",
                        "stochastic_gradient_descent",
                        "passive_aggressive",
                        "theil_sen",
                        "huber",
                        "quantile",
                        "svm",
                        "nu_svm",
                        "linear_svm",
                        "ada_boost",
                        "gradient_boosting_trees",
                    ]:
                        self._algorithm = sklearn.multioutput.MultiOutputRegressor(self._algorithm)

        return self._algorithm

    def fit(self, snapshot: Snapshot):
        """
        Learns the parameters of this model and records them in the database.

        Args:
            snapshot (Snapshot): dataset used to train this model
        """
        X_train, X_test, y_train, y_test = snapshot.data()

        # Train the model
        self.algorithm.fit(X_train, y_train)

        # Test
        y_pred = self.algorithm.predict(X_test)
        metrics = {}
        if self.project.objective == "regression":
            metrics["mean_squared_error"] = mean_squared_error(y_test, y_pred)
            metrics["r2"] = r2_score(y_test, y_pred)
        elif self.project.objective == "classification":
            metrics["f1"] = f1_score(y_test, y_pred, average="weighted")
            metrics["precision"] = precision_score(y_test, y_pred, average="weighted")
            metrics["recall"] = recall_score(y_test, y_pred, average="weighted")

        # Save the model
        self.__dict__ = dict(
            plpy.execute(
                f"""
            UPDATE pgml.models
            SET pickle = '\\x{pickle.dumps(self.algorithm).hex()}',
                status = 'successful',
                metrics = {q(json.dumps(metrics))},
                updated_at = clock_timestamp()
            WHERE id = {q(self.id)}
            RETURNING *
        """
            )[0]
        )
        self.__init__()

    def deploy(self, strategy):
        """Promote this model to the active version for the project that will be used for predictions"""

        plpy.execute(
            f"""
            INSERT INTO pgml.deployments (project_id, model_id, strategy, created_at) 
            VALUES ({q(self.project_id)}, {q(self.id)}, {q(strategy)}, clock_timestamp())
        """
        )

    def predict(self, data: list):
        """Use the model for a set of features.

        Args:
            data (list): list of features to form a single prediction for

        Returns:
            float or int: scores for regressions or ints for classifications
        """
        # TODO: add metrics for tracking prediction volume/accuracy by model
        # TODO: smarter treatment for images rather than flattening
        y = self.algorithm.predict([flatten(data)])
        if self.project.objective == "regression" and len(self.snapshot.y_column_name) == 1:
            y = y[0]
        return y


def train(
    project_name: str,
    objective: str = None,
    relation_name: str = None,
    y_column_name: str = None,
    algorithm_name: str = "linear",
    hyperparams: dict = {},
    test_size: float or int = 0.1,
    test_sampling: str = "random",
):
    """Create a regression model from a table or view filled with training data.

    Args:
        project_name (str): a human friendly identifier
        objective (str): Defaults to "regression". Valid values are ["regression", "classification"].
        relation_name (str): the table or view that stores the training data
        y_column_name (str): the column in the training data that acts as the label
        algorithm_name (str, optional): the algorithm used to implement the objective. Defaults to "linear". Valid values are ["linear", "svm", "random_forest", "gradient_boosting"].
        test_size (float or int, optional): If float, should be between 0.0 and 1.0 and represent the proportion of the dataset to include in the test split. If int, represents the absolute number of test samples. If None, the value is set to the complement of the train size. If train_size is also None, it will be set to 0.25.
        test_sampling: (str, optional): How to sample to create the test data. Defaults to "random". Valid values are ["first", "last", "random"].
    """

    try:
        project = Project.find_by_name(project_name)
        if objective is not None and objective != project.objective:
            raise PgMLException(
                f"Project `{project_name}` already exists with a different objective: `{project.objective}`. Create a new project instead."
            )
        objective = project.objective
    except PgMLException:
        project = Project.create(project_name, objective)

    if algorithm_name is None:
        algorithm_name = "linear"

    if objective not in ["regression", "classification"]:
        raise PgMLException(f"Unknown objective `{objective}`, available options are: regression, classification.")

    if relation_name is None:
        snapshot = project.last_snapshot
        if snapshot is None:
            raise PgMLException(
                f"You must pass a `relation_name` and `y_column_name` to snapshot the first time you train a model."
            )
        if y_column_name is not None and y_column_name != [None] and y_column_name != snapshot.y_column_name:
            raise PgMLException(
                f"You must pass a `relation_name` to use a different `y_column_name` than previous runs. {y_column_name} vs {snapshot.y_column_name}"
            )

    else:
        snapshot = Snapshot.create(relation_name, y_column_name, test_size, test_sampling)

    model = Model.create(project, snapshot, algorithm_name, hyperparams)
    model.fit(snapshot)

    if (
        project.deployed_model is None
        or (
            project.objective == "regression"
            and project.deployed_model.metrics["mean_squared_error"] > model.metrics["mean_squared_error"]
        )
        or (project.objective == "classification" and project.deployed_model.metrics["f1"] < model.metrics["f1"])
    ):
        model.deploy("new_score")
        return "deployed"
    else:
        return "not deployed"
