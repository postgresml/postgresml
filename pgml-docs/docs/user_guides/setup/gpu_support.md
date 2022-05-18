# GPU Support

PostgresML is capable of leveraging GPUs when the underlying libraries and hardware are properly configured on the database server. 

!!! tip
    Models trained on GPU will also require GPU support to make predictions.

## XGBoost 
GPU setup for XGBoost is covered in the [xgboost documentation](https://xgboost.readthedocs.io/en/stable/gpu/index.html).

!!! example 
    ```sql linenums="1"
        pgml.train(
            'GPU project', 
            algorithm => 'xgboost', 
            hyperparams => '{"tree_method" : "gpu_hist"}'
        );
    ```

## LightGBM
GPU setup for LightGBM is covered in the [lightgbm documentation](https://lightgbm.readthedocs.io/en/latest/GPU-Tutorial.html). 

!!! example 
    ```sql linenums="1"
        pgml.train(
            'GPU project', 
            algorithm => 'lightgbm', 
            hyperparams => '{"device" : "gpu"}'
        );
    ```

## Scikit-learn
None of the scikit-learn algorithms natively support GPU devices. There are a few projects to improve scikit performance with additional parralellism, although we currently have not integrated these with PostgresML:

- https://github.com/intel/scikit-learn-intelex
- https://github.com/rapidsai/cuml

If your project would benefit from GPU support, please consider providing benchmarks and opening an issue so we can prioritize these integrations.
