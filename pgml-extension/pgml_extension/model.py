from re import M
import os
from typing import OrderedDict
import plpy
import sklearn.linear_model
import sklearn.kernel_ridge
import sklearn.svm
import sklearn.ensemble
import sklearn.multioutput
import sklearn.gaussian_process
import sklearn.model_selection
import numpy
import pandas
from datasets import Dataset, DatasetDict, DatasetInfo
import datasets
import xgboost as xgb
import diptest
import lightgbm
from sklearn.model_selection import train_test_split
from sklearn.metrics import (
    mean_squared_error,
    r2_score,
    f1_score,
    precision_score,
    recall_score,
    roc_auc_score,
    accuracy_score,
    log_loss,
)

import pickle
import json

from pgml_extension.exceptions import PgMLException
from pgml_extension.sql import q, c


_POSTGRES_TO_PANDAS_TYPE_MAP = {
    "boolean": "bool",
    "character": "int8",
    "smallint": "int16",
    "smallserial": "int16",
    "integer": "int32",
    "serial": "int32",
    "bigint": "int64",
    "bigserial": "int64",
    "real": "float64",
    "double precision": "float64",
    "numeric": "float64",
    "decimal": "float64",
    "text": "string",
    "character varying": "string",
    "bytea": "binary",
    "timestamp without time zone": "timestamp",
    "timestamp with time zone": "timestamp",
    "date": "date32",
    "time without time zone": "time64",
    "time with time zone": "time64",
    "jsonb": "string",
}
_PYTHON_TO_PANDAS_TYPE_MAP = {
    str: "string",
    int: "int32",
    float: "float32",
    bool: "boolean",
}
_DISCRETE_NUMERIC_TYPES = {
    "boolean",
    "character",
    "smallint",
    "small_serial",
    "integer",
    "serial",
    "bigint",
    "bigserial",
}
_TEXT_TYPES = {
    "character varying",
    "text",
}
_DEFAULT_KEY_METRIC_MAP = {
    "regression": "r2",
    "classification": "f1",
    "text-classification": "f1",
    "question-answering": "f1",
    "translation": "blue",
    "summarization": "rouge_ngram_f1",
}
_DEFAULT_HYPERPARAM_SCORE_MAP = {
    "regression": "r2",
    "classification": "f1_micro",
    "text-classification": "f1_micro",
    "question-answering": "f1_micro",
    "translation": "blue",
    "summarization": "rouge_ngram_f1",
}
_ALGORITHM_MAP = {
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
    "xgboost_random_forest_regression": xgb.XGBRFRegressor,
    "xgboost_random_forest_classification": xgb.XGBRFClassifier,
    "lightgbm_regression": lightgbm.LGBMRegressor,
    "lightgbm_classification": lightgbm.LGBMClassifier,
}

_project_cache = {}
_last_deploy_id = None


class Project(object):
    """
    Use projects to refine multiple models of a particular dataset on a specific task.

    Attributes:
        id (int): a unique identifier
        name (str): a human friendly unique identifier
        task (str): the purpose of this project
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

        project = cls()
        project.__dict__ = dict(result[0])
        project.__init__()
        return project

    @classmethod
    def find_by_name(cls, name: str, last_deploy_id: int = None):
        """
        Get a Project from the database by name.

        This is the prefered API to retrieve projects, and they are cached by
        name to avoid needing to go to he database on every usage.

        Args:
            name (str): the project name
        Returns:
            Project or None: instantiated from the database if found
        """
        global _project_cache, _last_deploy_id

        if last_deploy_id is not None and _last_deploy_id != last_deploy_id:
            for project in _project_cache.values():
                project.expire_cached_deployed_model()
            _last_deploy_id = last_deploy_id

        project = _project_cache.get(name)
        if project is None:
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

            project = cls()
            project.__dict__ = dict(result[0])
            project.__init__()
            _project_cache[name] = project

        return project

    @classmethod
    def create(cls, name: str, task: str):
        """
        Create a Project and save it to the database.

        Args:
            name (str): a human friendly identifier
            task (str): valid values are ["regression", "classification"].
        Returns:
            Project: instantiated from the database
        """
        if task is None:
            raise PgMLException(f"You must specify and task when creating a new Project.")
        project = cls()
        project.__dict__ = dict(
            plpy.execute(
                f"""
                    INSERT INTO pgml.projects (name, task, created_at, updated_at) 
                    VALUES ({q(name)}, {q(task)}, clock_timestamp(), clock_timestamp()) 
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
            self._deployed_model = Model.find_deployed(self)
        return self._deployed_model

    def expire_cached_deployed_model(self):
        result = plpy.execute(
            f"""
            SELECT deployments.model_id 
            FROM pgml.deployments 
            WHERE deployments.project_id = {q(self.id)}
            ORDER by deployments.created_at DESC
            LIMIT 1
        """
        )
        if len(result) == 0:
            return

        if self._deployed_model and self._deployed_model.id != result[0]["model_id"]:
            self._deployed_model = None

    def deploy(self, qualifier="best_score", algorithm_name=None):
        model = Model.find_by_project_and_qualifier_algorithm_name(self, qualifier, algorithm_name)
        if model and model.id != self.deployed_model.id:
            model.deploy(qualifier)
        return model

    @property
    def key_metric_name(self):
        return _DEFAULT_KEY_METRIC_MAP[self.task_type]

    @property
    def hyperparam_score_name(self):
        return _DEFAULT_HYPERPARAM_SCORE_MAP[self.task_type]

    @property
    def last_snapshot(self):
        return Snapshot.last_for_project_id(self.id)

    @property
    def task_type(self):
        if self.task.startswith("translation"):
            return "translation"
        return self.task


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
            test_size (float or int, optional): If float, should be between 0.0 and 1.0 and represent the proportion of the dataset to include in the test split. If int, represents the absolute number of test samples. If None, the value is set to 0.25.
            test_sampling: (str, optional): How to sample to create the test data. Defaults to "random". Valid values are ["first", "last", "random"].
        Returns:
            Snapshot: metadata instantiated from the database
        """

        snapshot = cls()
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
        snapshot.__init__()
        sql = f"""
            CREATE TABLE pgml."snapshot_{snapshot.id}" AS 
            SELECT * FROM {snapshot.relation_name}
        """
        if snapshot.test_sampling == "random":
            sql += "ORDER BY random()"
        plpy.execute(sql)

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
        snapshot.__init__()
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

        snapshot = cls()
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

        snapshot = cls()
        snapshot.__dict__ = dict(result[0])
        snapshot.__init__()
        return snapshot

    def __init__(self):
        self._features = None

    def analyze(self):
        sample = plpy.execute(
            f"""
            SELECT * 
            FROM "pgml"."snapshot_{self.id}"
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
                if isinstance(value, bool):
                    quoted_column = c(column) + "::INT"
                else:
                    quoted_column = c(column)
                values.append(
                    f"""\n  min({quoted_column})::FLOAT4 AS "{column}_min", max({quoted_column})::FLOAT4 AS "{column}_max", avg({quoted_column})::FLOAT4 AS "{column}_mean", stddev({quoted_column})::FLOAT4 AS "{column}_stddev", percentile_disc(0.25) within group (order by {quoted_column}) AS "{column}_p25", percentile_disc(0.5) within group (order by {quoted_column}) AS "{column}_p50", percentile_disc(0.75) within group (order by {quoted_column}) AS "{column}_p75", count({quoted_column})::INT AS "{column}_count", count(distinct {quoted_column})::INT AS "{column}_distinct", sum(({quoted_column} IS NULL)::int)::INT AS "{column}_nulls" """
                )
        self.analysis = dict(plpy.execute(f"SELECT {','.join(values)} FROM {self.relation_name}")[0])

        for (column, value) in dict(sample[0]).items():
            if isinstance(value, float) or isinstance(value, int):
                data = [row[column] for row in sample]
                self.analysis[f"{column}_dip"] = diptest.dipstat(data)

        self.columns = {}
        result = plpy.execute(
            f"""
                SELECT column_name, data_type 
                FROM information_schema.columns 
                WHERE table_name = 'snapshot_{self.id}' and table_schema = {q("pgml")}
            """
        )
        for row in result:
            self.columns[row["column_name"]] = row["data_type"]

        self.__dict__ = dict(
            plpy.execute(
                f"""
                    UPDATE pgml.snapshots 
                    SET columns = {q(self.columns)}, analysis = {q(self.analysis)}, updated_at = clock_timestamp()
                    WHERE id = {q(self.id)} 
                    RETURNING *
                """,
                1,
            )[0]
        )

    @property
    def snapshot_name(self):
        return f'"pgml"."snapshot_{self.id}"'

    def data(self):
        """
        Returns:
            list, list, list, list: All rows from the snapshot split into X_train, X_test, y_train, y_test sets.
        """
        data = plpy.execute(f"SELECT * FROM {self.snapshot_name}")

        # Always pull the columns in the same order from the row.
        # Python dict iteration is not always in the same order (hash table).
        features = list(data[0].keys())
        for column in self.y_column_name:
            features.remove(column)

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

            x_ = numpy.array(x_).flatten()  # TODO be smart about flattening X depending on algorithm
            X.append(x_)
            y.append(y_)

        # Split into training and test sets
        if self.test_sampling == "random":
            return train_test_split(X, y, test_size=self.test_size, shuffle=False)
        else:
            split = self.test_size
            if self.test_sampling == "first":
                X.reverse()
                y.reverse()
                if isinstance(split, float):
                    split = 1.0 - split
            if isinstance(split, float):
                split = int(self.test_size * len(X))
            return X[:split], X[split:], y[:split], y[split:]

        # TODO normalize and clean data

    @property
    def dataset(self):
        data = plpy.execute(f"SELECT * FROM {self.snapshot_name}")

        # parse jsonb columns
        for i, pg_type in enumerate(data.coltypes()):
            col = data.colnames()[i]
            if pg_type == 3802:  # jsonb
                for row in data:
                    row[col] = json.loads(row[col])

        # Datasets are constructed from DataFrames
        dataframe = pandas.DataFrame.from_records(data)
        if self.test_sampling == "first":
            dataframe = dataframe.loc[::-1].reset_index(drop=True)

        # construct the features for the Dataset
        features = OrderedDict()
        for name, pg_type in self.features.items():
            if pg_type == "jsonb":
                value = data[0][name]
                if type(value) == dict:
                    if all(isinstance(v, list) for v in value.values()):
                        features[name] = datasets.Sequence(
                            {k: datasets.Value(_PYTHON_TO_PANDAS_TYPE_MAP[type(v[0])]) for k, v in value.items()}
                        )
                    elif name == "translation":
                        features[name] = datasets.Translation(languages=list(value.keys()))
                    else:
                        raise PgMLException(f"Unhandled json dict: {value}")
                else:
                    raise PgMLException(f"Unhandled json value: {value}")
            elif name in self.y_column_name:
                if pg_type == "boolean":
                    features[name] = datasets.ClassLabel(num_classes=2)
                elif pg_type in _DISCRETE_NUMERIC_TYPES:
                    features[name] = datasets.ClassLabel(num_classes=int(dataframe[name].max() + 1))
                elif pg_type in _TEXT_TYPES:
                    # TODO need to differentiate between Seq2Seq vs Classification labels
                    features[name] = datasets.Value(_POSTGRES_TO_PANDAS_TYPE_MAP[pg_type])
                else:
                    raise PgMLException(f"Unhandled label type: {pg_type}")
            else:
                features[name] = datasets.Value(_POSTGRES_TO_PANDAS_TYPE_MAP[pg_type])
        features = datasets.Features(features)

        # Split the data. It was shuffled during snapshot creation if appropriate.
        test_size = self.test_size
        if test_size > 1:
            test_size = int(test_size)
        train, test = train_test_split(dataframe, test_size=test_size, shuffle=False)

        return DatasetDict(
            {
                "train": Dataset.from_pandas(train, features=features, preserve_index=False),
                "test": Dataset.from_pandas(test, features=features, preserve_index=False),
            }
        )

    @property
    def features(self):
        if self._features:
            return self._features

        result = plpy.execute(
            f"""
            SELECT column_name, data_type
            FROM information_schema.columns
            WHERE table_schema = 'pgml'
                AND table_name = 'snapshot_{self.id}'
            ORDER BY ordinal_position ASC
        """
        )

        features = OrderedDict()
        for row in result:
            features[row["column_name"]] = row["data_type"]

        self._features = features
        return self._features

    @property
    def feature_names(self):
        return list(filter(lambda x: x not in self.y_column_name, self.features))


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
        algorithm: the in memory version of the model parameters that can make predictions
    """

    @classmethod
    def algorithm_from_name_and_task(cls, name: str, task: str):
        return _ALGORITHM_MAP[name + "_" + task]

    @classmethod
    def create(
        cls,
        project: Project,
        snapshot: Snapshot,
        algorithm_name: str,
        hyperparams: dict,
        search: str,
        search_params: dict,
        search_args: dict,
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
            INSERT INTO pgml.models (project_id, snapshot_id, algorithm_name, hyperparams, status, search, search_params, search_args, created_at, updated_at) 
            VALUES ({q(project.id)}, {q(snapshot.id)}, {q(algorithm_name)}, {q(hyperparams)}, 'new', {q(search)}, {q(search_params)}, {q(search_args)}, clock_timestamp(), clock_timestamp()) 
            RETURNING *
        """
        )
        model = cls()
        model.__dict__ = dict(result[0])
        model.__init__()
        model._snapshot = snapshot
        model._project = project
        return model

    @classmethod
    def find_deployed(cls, project):
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
              AND deployments.project_id = {q(project.id)}
            ORDER by deployments.created_at DESC
            LIMIT 1
        """
        )
        if len(result) == 0:
            return None

        if project.task in ["regression", "classification"]:
            model = cls()
        else:
            # avoid circular dependency by importing after this module is completely initialized
            from . import transformers

            model = transformers.Model()

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
            sql += f"{where}\nORDER BY models.metrics->>{q(project.key_metric_name)} DESC NULLS LAST"
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

        model = cls()
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
        result = plpy.execute(
            f"""
            SELECT models.* 
            FROM pgml.models 
            WHERE project_id = {q(project.id)}
            ORDER by models.metrics->>{q(project.key_metric_name)} DESC
            LIMIT 1
        """
        )
        if len(result) == 0:
            return None

        model = cls()
        model.__dict__ = dict(result[0])
        model.__init__()
        return model

    @classmethod
    def find_by_id(cls, pk: int):
        """Find the model by primary key in pgml.models"""
        result = plpy.execute(
            f"""
                SELECT models.*
                FROM pgml.models
                WHERE id = {q(pk)}
            """
        )
        if len(result) == 0:
            return None

        model = cls()
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
        if "search_params" in self.__dict__ and type(self.search_params) is str:
            self.search_params = json.loads(self.search_params)
        if "search_args" in self.__dict__ and type(self.search_args) is str:
            self.search_args = json.loads(self.search_args)

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
    def path(self):
        return os.path.join("/tmp", "postgresml", "models", str(self.id))

    @property
    def pickle_path(self):
        return os.path.join(self.path, "algorithm.pickle")

    @property
    def algorithm(self):
        if self._algorithm is None:
            files = plpy.execute(
                f"SELECT * FROM pgml.files WHERE model_id = {self.id} AND path = '{self.pickle_path}' LIMIT 1"
            )
            if len(files) > 0:
                self._algorithm = pickle.loads(files[0]["data"])
            else:
                algorithm = Model.algorithm_from_name_and_task(self.algorithm_name, self.project.task)
                self._algorithm = algorithm(**self.hyperparams)
                if len(self.snapshot.y_column_name) > 1:
                    if self.project.task == "regression" and self.algorithm_name in [
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
                        "lightgbm",
                    ]:
                        self._algorithm = sklearn.multioutput.MultiOutputRegressor(self._algorithm)

        return self._algorithm

    def train(self):
        X_train, X_test, y_train, y_test = self.snapshot.data()
        result = self.algorithm.fit(X_train, y_train)
        metrics = {}
        if self.search:
            self._algorithm = result.best_estimator_
            self.hyperparams = result.best_params_
            for key, value in result.cv_results_.items():
                if isinstance(value, numpy.ndarray):
                    result.cv_results_[key] = value.tolist()
            metrics["search_results"] = {
                "best_index": int(result.best_index_),
                "n_splits": int(result.n_splits_),
                **result.cv_results_,
            }

        # Test
        y_pred = self.algorithm.predict(X_test)
        if hasattr(self.algorithm, "predict_proba"):
            y_prob = self.algorithm.predict_proba(X_test)
            y_test = numpy.array(y_test).flatten()
        else:
            y_prob = None
        if self.project.task == "regression":
            metrics["mean_squared_error"] = mean_squared_error(y_test, y_pred)
            metrics["r2"] = r2_score(y_test, y_pred)
        elif self.project.task == "classification":
            metrics["f1"] = f1_score(y_test, y_pred, average="weighted")
            metrics["precision"] = precision_score(y_test, y_pred, average="weighted")
            metrics["recall"] = recall_score(y_test, y_pred, average="weighted")
            metrics["accuracy"] = accuracy_score(y_test, y_pred)
            if y_prob is not None:
                metrics["log_loss"] = log_loss(y_test, y_prob)
                roc_auc_y_prob = y_prob
                if (
                    y_prob.shape[1] == 2
                ):  # binary classification requires only the greater label by passed to roc_auc_score
                    roc_auc_y_prob = y_prob[:, 1]
                metrics["roc_auc"] = roc_auc_score(y_test, roc_auc_y_prob, average="weighted", multi_class="ovo")

        self.metrics = metrics

        # Save the results
        plpy.execute(
            f"""
            INSERT into pgml.files (model_id, path, part, data) 
            VALUES ({q(self.id)}, {q(self.pickle_path)}, 0, '\\x{pickle.dumps(self.algorithm).hex()}')
            """
        )

    def fit(self, snapshot: Snapshot):
        """
        Learns the parameters of this model and records them in the database.

        Args:
            snapshot (Snapshot): dataset used to train this model
        """

        search_args = {"scoring": self.project.hyperparam_score_name, "error_score": "raise", **self.search_args}
        if self.search == "grid":
            self._algorithm = sklearn.model_selection.GridSearchCV(self.algorithm, self.search_params, **search_args)
        elif self.search == "random":
            self._algorithm = sklearn.model_selection.RandomizedSearchCV(
                self.algorithm, self.search_params, **search_args
            )
        elif self.search is not None:
            raise PgMLException(
                f"Unknown hyperparam search `{self.search}`, available options are: ['grid', 'random']."
            )

        self.train()

        self.__dict__ = dict(
            plpy.execute(
                f"""
            UPDATE pgml.models
            SET status = 'successful',
                hyperparams = {q(self.hyperparams)},
                metrics = {q(self.metrics)},
                updated_at = clock_timestamp()
            WHERE id = {q(self.id)}
            RETURNING *
        """
            )[0]
        )
        self.__init__()

    def deploy(self, strategy):
        """Promote this model to the active version for the project that will be used for predictions"""

        deployment_id = dict(
            plpy.execute(
                f"""
            INSERT INTO pgml.deployments (project_id, model_id, strategy, created_at) 
            VALUES ({q(self.project_id)}, {q(self.id)}, {q(strategy)}, clock_timestamp())
            RETURNING id
        """
            )[0]
        )["id"]

        plpy.execute(
            f"""
        CREATE OR REPLACE FUNCTION pgml.predict(
            project_name TEXT,          -- Human-friendly project name
            features DOUBLE PRECISION[] -- Must match the training data column order
        )
        RETURNS DOUBLE PRECISION
        AS $$
            from pgml_extension.model import Project

            return float(Project.find_by_name(project_name, {q(deployment_id)}).deployed_model.predict(features))
        $$ LANGUAGE plpython3u;
        """
        )

        plpy.execute(
            f"""
        CREATE OR REPLACE FUNCTION pgml.predict_joint(
            project_name TEXT,          -- Human-friendly project name
            features DOUBLE PRECISION[] -- Must match the training data column order
        )
        RETURNS DOUBLE PRECISION[]
        AS $$
            from pgml_extension.model import Project

            return Project.find_by_name(project_name, {q(deployment_id)}).deployed_model.predict(features)
        $$ LANGUAGE plpython3u;
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
        y = self.algorithm.predict([numpy.array(data).flatten()])
        if isinstance(y[0], numpy.ndarray) and len(self.snapshot.y_column_name) == 1:
            # HACK: it's unfortunate that some sklearn models always return a 2D array, and some only return a 2D array for multiple outputs.
            return y[0][0]
        return y[0]


def snapshot(
    relation_name: str,
    y_column_name: str,
    test_size: float or int = 0.25,
    test_sampling: str = "random",
):
    """Create a snapshot of a relation.

    Same args as train() below."""
    return Snapshot.create(relation_name, y_column_name, test_size, test_sampling)


def train(
    project_name: str,
    task: str = None,
    relation_name: str = None,
    y_column_name: str = None,
    algorithm_name: str = "linear",
    hyperparams: dict = {},
    search: str = None,
    search_params: dict = {},
    search_args: dict = {},
    test_size: float or int = 0.25,
    test_sampling: str = "random",
):
    """Create a regression model from a table or view filled with training data.

    Args:
        project_name (str): a human friendly identifier
        task (str): Defaults to "regression". Valid values are ["regression", "classification"].
        relation_name (str): the table or view that stores the training data
        y_column_name (str): the column in the training data that acts as the label
        algorithm_name (str, optional): the algorithm used to implement the task. Defaults to "linear". Valid values are ["linear", "svm", "random_forest", "gradient_boosting"].
        test_size (float or int, optional): If float, should be between 0.0 and 1.0 and represent the proportion of the dataset to include in the test split. If int, represents the absolute number of test samples. If None, the value is set to 0.25.
        test_sampling: (str, optional): How to sample to create the test data. Defaults to "random". Valid values are ["first", "last", "random"].
    """
    # Project
    try:
        project = Project.find_by_name(project_name)
        if task is not None and task != project.task:
            raise PgMLException(
                f"Project `{project_name}` already exists with a different task: `{project.task}`. Create a new project instead."
            )
        task = project.task
    except PgMLException:
        project = Project.create(project_name, task)

    if task not in ["regression", "classification"]:
        raise PgMLException(f"Unknown task `{task}`, available options are: regression, classification.")

    # Create or use an existing snapshot.
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

    # Model
    if algorithm_name is None:
        algorithm_name = "linear"

    # Default repeatable random state when possible
    algorithm = Model.algorithm_from_name_and_task(algorithm_name, task)
    if "random_state" in algorithm().get_params() and "random_state" not in hyperparams:
        hyperparams["random_state"] = 0

    model = Model.create(project, snapshot, algorithm_name, hyperparams, search, search_params, search_args)
    model.fit(snapshot)

    # Deployment
    if (
        project.deployed_model is None
        or project.deployed_model.metrics[project.key_metric_name] < model.metrics[project.key_metric_name]
    ):
        model.deploy("new_score")
        return "deployed"
    else:
        return "not deployed"
