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
import xgboost as xgb
import lightgbm
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
    "xgboost_regression": xgb.XGBRegressor,
    "xgboost_classification": xgb.XGBClassifier,
    "xgboost_random_forest_regression": xgb.XGBRFRegressor,
    "xgboost_random_forest_classification": xgb.XGBRFClassifier,
    "lightgbm_regression": lightgbm.LGBMRegressor,
    "lightgbm_classification": lightgbm.LGBMClassifier,
}


def estimator(algorithm, num_features, hyperparams):
    """Returns the correct estimator based on algorithm names
    we defined internally.

    Parameters:
        - algorithm: The human-readable name of the algorithm (see dict above).
        - num_features: The number of features in X.
        - hyperparams: JSON of hyperparameters.
    """
    return estimator_joint(algorithm, num_features, 1, hyperparams)


def estimator_joint(algorithm, num_features, num_targets, hyperparams):
    """Returns the correct estimator based on algorithm names we defined
    internally (see dict above).


    Parameters:
        - algorithm: The human-readable name of the algorithm (see dict above).
        - num_features: The number of features in X.
        - num_targets: Used for joint models (models that have more than one y target).
        - hyperparams: JSON of hyperparameters.
    """
    if hyperparams is None:
        hyperparams = {}
    else:
        hyperparams = json.loads(hyperparams)

    def train(X_train, y_train):
        instance = _ALGORITHM_MAP[algorithm](**hyperparams)

        X_train = np.asarray(X_train).reshape((-1, num_features))

        # Only support single value models for just now.
        y_train = np.asarray(y_train).reshape((-1, num_targets))

        instance.fit(X_train, y_train)
        return instance

    return train


def predictor(estimator):
    """Return the instantiated estimator
    given the number of features in X.

    Parameters:
        - estimator: Scikit-Learn estimator, instantiated.
    """
    return predictor_joint(estimator, 1)


def predictor_joint(estimator, num_targets):
    """Return the instantiated estimator
    given the number of features in X.

    Parameters:
        - estimator: Scikit-Learn estimator, instantiated.
        - num_targets: Used in joint models (more than 1 y target).
    """
    def predict(X):
        X = np.asarray(X).reshape((-1, estimator.n_features_in_))
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
