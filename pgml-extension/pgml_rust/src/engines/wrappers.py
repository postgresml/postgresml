#!/usr/bin/env python3
#
# Wrapper around Scikit-Learn loaded by PyO3
# in our Rust crate::engines::sklearn module.
#
import sklearn.linear_model
import sklearn.kernel_ridge
import sklearn.svm
import sklearn.ensemble
import sklearn.multioutput
import sklearn.gaussian_process
import sklearn.model_selection
import numpy as np
import pickle
import json

_ALGORITHM_MAP = {
    "linear_regression": sklearn.linear_model.LinearRegression,
    "linear_classification": sklearn.linear_model.LogisticRegression,
    "ridge_regression": sklearn.linear_model.Ridge,
    "ridge_classification": sklearn.linear_model.RidgeClassifier,
    "lasso_regression": sklearn.linear_model.Lasso,
    "elastic_net_regression": sklearn.linear_model.ElasticNet,
    "least_angle_regression": sklearn.linear_model.Lars,
    "lasso_least_angle_regression": sklearn.linear_model.LassoLars,
    "orthogonal_matching_persuit_regression": sklearn.linear_model.OrthogonalMatchingPursuit,
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
}


def estimator(algorithm_name, num_features, hyperparams):
    """Returns the correct estimator based on algorithm names
    we defined internally.

    Parameters:
        - algorithm_name: The human-readable name of the algorithm (see dict above).
        - num_features: The number of features in X.
        - hyperparams: JSON of hyperparameters.
    """
    return estimator_joint(algorithm_name, num_features, 1, hyperparams)


def estimator_joint(algorithm_name, num_features, num_targets, hyperparams):
    """Returns the correct estimator based on algorithm names we defined
    internally (see dict above).


    Parameters:
        - algorithm_name: The human-readable name of the algorithm (see dict above).
        - num_features: The number of features in X.
        - num_targets: Used for joint models (models that have more than one y target).
        - hyperparams: JSON of hyperparameters.
    """
    if hyperparams is None:
        hyperparams = {}
    else:
        hyperparams = json.loads(hyperparams)

    def train(X_train, y_train):
        instance = _ALGORITHM_MAP[algorithm_name](**hyperparams)

        X_train = np.asarray(X_train).reshape((-1, num_features))

        # Only support single value models for just now.
        y_train = np.asarray(y_train).reshape((-1, num_targets))

        instance.fit(X_train, y_train)
        return instance

    return train


def estimator_search_joint(algorithm_name, num_features, num_targets, hyperparams, search_params, search, search_args):
    """Hyperparameter search.

    Parameters:
        - algorithm_name: The human-readable name of the algorithm (see dict above).
        - num_features: The number of features in X.
        - num_targets: For joint training (more than one y target).
        - hyperparams: JSON of hyperparameters.
        - search_params: Hyperparameters to search (see Scikit docs for examples).
        - search: Type of search to do, grid or random.
        - search_args: See Scikit docs for examples.

    Return:
        A tuple of Estimator and chosen hyperparameters.
    """
    if search_args is None:
        search_args = {}
    else:
        search_args = json.loads(search_args)

    if search is None:
        search = "grid"

    search_params = json.loads(search_params)
    hyperparams = json.loads(hyperparams)

    if search == "random":
        algorithm = sklearn.model_selection.RandomizedSearchCV(
            _ALGORITHM_MAP[algorithm_name](**hyperparams),
            search_params,
        )
    elif search == "grid":
        algorithm = sklearn.model_selection.GridSearchCV(
            _ALGORITHM_MAP[algorithm_name](**hyperparams),
            search_params,
        )
    else:
        raise Exception(f"search can be 'grid' or 'random', got: '{search}'")

    def train(X_train, y_train):
        X_train = np.asarray(X_train).reshape((-1, num_features))
        y_train = np.asarray(y_train).reshape((-1, num_targets))

        algorithm.fit(X_train, y_train)

        return (algorithm.best_estimator_, json.dumps(algorithm.best_params_))

    return train


def estimator_search(algorithm_name, num_features, hyperparams, search_params, search, search_args):
    """Hyperparameter search.

    Parameters:
        - algorithm_name: The human-readable name of the algorithm (see dict above).
        - num_features: The number of features in X.
        - hyperparams: JSON of hyperparameters.
        - search_params: Hyperparameters to search (see Scikit docs for examples).
        - search: Type of search to do, grid or random.
        - search_args: See Scikit docs for examples.
    """
    return estimator_search_joint(algorithm_name, num_features, 1, hyperparams, search_params, search, search_args)


def test(estimator, X_test):
    """Validate the estimator using the test dataset.

    Parameters:
        - estimator: Scikit-Learn estimator, instantiated.
        - X_test: test dataset.
    """
    y_hat = estimator.predict(X_test)

    # Single value models only just for now.
    return list(np.asarray(y_hat).flatten())


def predictor(estimator, num_features):
    """Return the instantiated estimator
    given the number of features in X.

    Parameters:
        - estimator: Scikit-Learn estimator, instantiated.
        - num_features: The number of features in X.
    """
    return predictor_joint(estimator, num_features, 1)


def predictor_joint(estimator, num_features, num_targets):
    """Return the instantiated estimator
    given the number of features in X.

    Parameters:
        - estimator: Scikit-Learn estimator, instantiated.
        - num_features: The number of features in X.
        - num_targets: Used in joint models (more than 1 y target).
    """

    def predict(X):
        X = np.asarray(X).reshape((-1, num_features))
        y_hat = estimator.predict(X)

        # Only support single value models for just now.
        if num_targets == 1:
            return list(np.asarray(y_hat).flatten())
        else:
            return list(y_hat)

    return predict


def save(estimator):
    """Save the estimtator as bytes (pickle).

    Parameters:
        - estimator: Scikit-Learn estimator, instantiated.

    Return:
        bytes
    """
    return pickle.dumps(estimator)


def load(data):
    """Load the estimator from bytes (pickle).

    Parameters:
        - data: bytes

    Return:
        Scikit-Learn estimator
    """
    return pickle.loads(bytes(data))
