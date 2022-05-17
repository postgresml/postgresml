# GPU Support

PostgresML is capable of leveraging GPUs when the underlying libraries and hardware are properly configured on the database. 

## XGBoost 
XGBoost is currently the only integrated library that provides GPU accellaration. GPU setup for this library is covered in the [xgboost documentation](https://xgboost.readthedocs.io/en/stable/gpu/index.html). Additionally, you'll need to pass `pgml.train('GPU project', hyperparams => '{tree_method: "gpu_hist"}')` to take advantage during training.

!!! warning
    XGBoost models trained on GPU will also require GPU support to make predictions.

## Scikit-learn
None of the scikit-learn algorithms natively support GPU devices. There are a few projects to improve scikit performance with additional parralellism, although we currently have not integrated these with PostgresML:

- https://github.com/intel/scikit-learn-intelex
- https://github.com/rapidsai/cuml

If your project would benefit from GPU support, please consider providing benchmarks and opening an issue so we can prioritize these integrations.
