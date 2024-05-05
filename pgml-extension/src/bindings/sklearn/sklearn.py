#!/usr/bin/env python3
#
# Wrapper around Scikit-Learn loaded by PyO3
# in our Rust crate::engines::sklearn module.
#
import sklearn.cluster
import sklearn.linear_model
import sklearn.kernel_ridge
import sklearn.svm
import sklearn.ensemble
import sklearn.multioutput
import sklearn.gaussian_process
import sklearn.model_selection
import xgboost as xgb
import lightgbm
import catboost
import numpy as np
import pickle
import json

from sklearn.metrics import (
    r2_score,
    f1_score,
    precision_score,
    recall_score,
    accuracy_score,
    matthews_corrcoef,
    roc_auc_score,
    mean_squared_error,
    mean_absolute_error,
    confusion_matrix,
    silhouette_score,
    calinski_harabasz_score,
    fowlkes_mallows_score,
)

_ALGORITHM_MAP = {
    "linear_regression": sklearn.linear_model.LinearRegression,
    "linear_classification": sklearn.linear_model.LogisticRegression,
    "ridge_regression": sklearn.linear_model.Ridge,
    "ridge_classification": sklearn.linear_model.RidgeClassifier,
    "lasso_regression": sklearn.linear_model.Lasso,
    "elastic_net_regression": sklearn.linear_model.ElasticNet,
    "least_angle_regression": sklearn.linear_model.Lars,
    "lasso_least_angle_regression": sklearn.linear_model.LassoLars,
    "orthogonal_matching_pursuit_regression": sklearn.linear_model.OrthogonalMatchingPursuit,
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
    "catboost_regression": catboost.CatBoostRegressor,
    "catboost_classification": catboost.CatBoostClassifier,
    "affinity_propagation_clustering": sklearn.cluster.AffinityPropagation,
    "birch_clustering": sklearn.cluster.Birch,
    "dbscan_clustering": sklearn.cluster.DBSCAN,
    "feature_agglomeration_clustering": sklearn.cluster.FeatureAgglomeration,
    "kmeans_clustering": sklearn.cluster.KMeans,
    "mini_batch_kmeans_clustering": sklearn.cluster.MiniBatchKMeans,
    "mean_shift_clustering": sklearn.cluster.MeanShift,
    "optics_clustering": sklearn.cluster.OPTICS,
    "spectral_clustering": sklearn.cluster.SpectralClustering,
    "spectral_biclustering": sklearn.cluster.SpectralBiclustering,
    "spectral_coclustering": sklearn.cluster.SpectralCoclustering,
    "pca_decomposition": sklearn.decomposition.PCA,
}


def estimator(algorithm, num_features, num_targets, hyperparams):
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
        if num_targets > 1 and algorithm in [
            "bayesian_ridge_regression",
            "automatic_relevance_determination_regression",
            "stochastic_gradient_descent_regression",
            "passive_aggressive_regression",
            "theil_sen_regression",
            "huber_regression",
            "quantile_regression",
            "svm_regression",
            "nu_svm_regression",
            "linear_svm_regression",
            "ada_boost_regression",
            "gradient_boosting_trees_regression",
            "lightgbm_regression",
        ]:
            instance = sklearn.multioutput.MultiOutputRegressor(instance)

        X_train = np.asarray(X_train).reshape((-1, num_features))

        if num_targets > 0:
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


def predictor_proba(estimator):
    """Return the instantiated estimator
    given the number of features in X.

    Parameters:
        - estimator: Scikit-Learn estimator, instantiated.
        - num_targets: Used in joint models (more than 1 y target).
    """

    def predict_proba(X):
        X = np.asarray(X).reshape((-1, estimator.n_features_in_))
        y_hat = estimator.predict_proba(X)
        return list(np.asarray(y_hat).flatten())

    return predict_proba


def predictor_joint(estimator, num_targets):
    """Return the instantiated estimator
    given the number of features in X.

    Parameters:
        - estimator: Scikit-Learn estimator, instantiated.
        - num_targets: Used in joint models (more than 1 y target).
    """

    def predict(X):
        X = np.asarray(X).reshape((-1, estimator.n_features_in_))
        if hasattr(estimator.__class__, 'predict'):
            y_hat = estimator.predict(X)
        else:
            y_hat = estimator.transform(X)

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


def calculate_metric(metric_name):
    if metric_name == "r2":
        func = r2_score
    elif metric_name == "f1":
        func = f1_score
    elif metric_name == "precision":
        func = precision_score
    elif metric_name == "recall":
        func = recall_score
    elif metric_name == "roc_auc":
        func = roc_auc_score
    elif metric_name == "accuracy":
        func = accuracy_score
    elif metric_name == "mcc":
        func = matthews_corrcoef
    elif metric_name == "mse":
        func = mean_squared_error
    elif metric_name == "mae":
        func = mean_absolute_error
    elif metric_name == "confusion_matrix":
        func = confusion_matrix
    elif metric_name == "variance":
        func = variance
    else:
        raise Exception(f"Unknown metric requested: {metric_name}")

    def wrapper(y_true, y_hat):
        y_true = np.asarray(y_true).reshape((-1, 1))
        y_hat = np.asarray(y_hat).reshape((-1, 1))

        if metric_name == "confusion_matrix":
            return list(func(y_true, y_hat))
        else:
            return func(y_true, y_hat)

    return wrapper


def regression_metrics(y_true, y_hat):
    y_true = np.asarray(y_true).reshape((-1, 1))
    y_hat = np.asarray(y_hat).reshape((-1, 1))

    r2 = r2_score(y_true, y_hat)
    mse = mean_squared_error(y_true, y_hat)
    mae = mean_absolute_error(y_true, y_hat)

    return {
        "r2": r2,
        "mse": mse,
        "mae": mae,
    }


def classification_metrics(y_true, y_hat):
    y_true = np.asarray(y_true).reshape((-1, 1))
    y_hat = np.asarray(y_hat).reshape((-1, 1))

    unique_labels = set()

    for label in y_hat:
        unique_labels.add(label[0])
    for label in y_true:
        unique_labels.add(label[0])

    multiclass = len(unique_labels) > 2

    f1 = f1_score(y_true, y_hat, average=("macro" if multiclass else "binary"))
    f1_micro = f1_score(y_true, y_hat, average="micro")
    precision = precision_score(
        y_true, y_hat, average=("macro" if multiclass else "binary")
    )
    recall = recall_score(y_true, y_hat, average=("macro" if multiclass else "binary"))
    accuracy = accuracy_score(y_true, y_hat)
    mcc = matthews_corrcoef(y_true, y_hat)

    return {
        "f1": f1,
        "f1_micro": f1_micro,
        "precision": precision,
        "recall": recall,
        "accuracy": accuracy,
        "mcc": mcc,
    }


def clustering_metrics(num_features, inputs_labels):
    inputs = np.asarray(inputs_labels[0]).reshape((-1, num_features))
    labels = np.asarray(inputs_labels[1]).reshape((-1, 1))

    return {
        "silhouette": silhouette_score(inputs, labels),
    }

def decomposition_metrics(pca):
    return {
      "cumulative_explained_variance": sum(pca.explained_variance_ratio_)
    }
