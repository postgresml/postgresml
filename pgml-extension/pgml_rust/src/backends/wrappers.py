import sklearn.linear_model
import sklearn.kernel_ridge
import sklearn.svm
import sklearn.ensemble
import sklearn.multioutput
import sklearn.gaussian_process
import sklearn.model_selection
import numpy as np
import pickle

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
   }

def estimator(algorithm_name, num_features):
    def train(X_train, y_train):
        instance = _ALGORITHM_MAP[algorithm_name]()

        X_train = np.asarray(X_train).reshape((-1, num_features))

        # Only support single value models for just now.
        y_train = np.asarray(y_train).reshape((-1, 1))

        instance.fit(X_train, y_train)
        return instance
    return train

def test(estimator, X_test):
    y_hat = estimator.predict(X_test)

    # Single value models only just for now.
    return list(np.asarray(y_hat).flatten())


def predictor(estimator, num_features):
    def predict(X):
        X = np.asarray(X).reshape((-1, num_features))
        y_hat = estimator.predict(X)

        # Only support single value models for just now.
        return list(np.asarray(y_hat).flatten())
    return predict

def save(estimator):
    return pickle.dumps(estimator)

def load(data):
    return pickle.loads(bytes(data))
