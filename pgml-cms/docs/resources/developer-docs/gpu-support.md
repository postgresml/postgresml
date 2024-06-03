# GPU Support

PostgresML is capable of leveraging GPUs when the underlying libraries and hardware are properly configured on the database server. The CUDA runtime is statically linked during the build process, so it does not introduce additional dependencies on the runtime host.

!!! tip

Models trained on GPU may also require GPU support to make predictions. Consult the documentation for each library on configuring training vs inference.

!!!

## Tensorflow

GPU setup for Tensorflow is covered in the [documentation](https://www.tensorflow.org/install/pip). You may acquire pre-trained GPU enabled models for fine tuning from Hugging Face.

## Torch

GPU setup for Torch is covered in the [documentation](https://pytorch.org/get-started/locally/). You may acquire pre-trained GPU enabled models for fine tuning from Hugging Face.

## Flax

GPU setup for Flax is covered in the [documentation](https://github.com/google/jax#pip-installation-gpu-cuda). You may acquire pre-trained GPU enabled models for fine tuning from Hugging Face.

## XGBoost

GPU setup for XGBoost is covered in the [documentation](https://xgboost.readthedocs.io/en/stable/gpu/index.html).

!!! example

```postgresql
pgml.train(
    'GPU project', 
    algorithm => 'xgboost', 
    hyperparams => '{"tree_method" : "gpu_hist"}'
);
```

!!!

## LightGBM

GPU setup for LightGBM is covered in the [documentation](https://lightgbm.readthedocs.io/en/latest/GPU-Tutorial.html).

!!! example

```postgresql
pgml.train(
    'GPU project', 
    algorithm => 'lightgbm', 
    hyperparams => '{"device" : "cuda"}'
);
```

!!!

## Scikit-learn

None of the scikit-learn algorithms natively support GPU devices. There are a few projects to improve scikit performance with additional parallelism, although we currently have not integrated these with PostgresML:

* https://github.com/intel/scikit-learn-intelex
* https://github.com/rapidsai/cuml

If your project would benefit from GPU support, please consider opening an issue, so we can prioritize integrations.
