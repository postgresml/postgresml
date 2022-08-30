# GPUTreeShap

GPUTreeShap is a cuda implementation of the TreeShap algorithm by Lundberg et al. [1] for Nvidia GPUs. It is a header only module designed to be included in decision tree libraries as a fast backend for model interpretability using SHAP values. GPUTreeShap also implements variants of TreeShap based on Taylor-Shapley interaction indices [2], and interventional probability instead of conditional probability [3].

See the associated publication [here](https://arxiv.org/abs/2010.13972)
```
@misc{mitchell2020gputreeshap,
      title={GPUTreeShap: Fast Parallel Tree Interpretability}, 
      author={Rory Mitchell and Eibe Frank and Geoffrey Holmes},
      year={2020},
      eprint={2010.13972},
      archivePrefix={arXiv},
      primaryClass={cs.LG}
}
```

## Using GPUTreeShap
GPUTreeShap is integrated with XGBoost 1.3 onwards, [see here for details](https://xgboost.readthedocs.io/en/latest/gpu/index.html#gpu-accelerated-shap-values) and [here for a demo notebook](https://github.com/dmlc/xgboost/blob/master/demo/gpu_acceleration/shap.ipynb).

Integration with the python shap package is a work in progress, and is expected to support a wider range of models such as LightGBM, Catboost, and sklearn random forests.

For usage in C++, see the example directory.

## Performance
Using the benchmark script `benchmark/benchmark.py` we run GPUTreeShap as a backend for xgboost and compare its performance against multithreaded CPU based implementation. Test models are generated on four different datasets at different sizes. The below comparison is run on an Nvidia DGX-1 system, comparing a single V100 to 2X 20-Core Intel Xeon
E5-2698 CPUs (40 physical cores total).

|       model       |trees|leaves |max_depth|average_depth|
|-------------------|----:|------:|--------:|------------:|
|covtype-small      |   80|    560|        3|        2.929|
|covtype-med        |  800| 113533|        8|        7.696|
|covtype-large      | 8000|6702132|       16|       13.654|
|cal_housing-small  |   10|     80|        3|        3.000|
|cal_housing-med    |  100|  21641|        8|        7.861|
|cal_housing-large  | 1000|3370373|       16|       14.024|
|fashion_mnist-small|  100|    800|        3|        3.000|
|fashion_mnist-med  | 1000| 144211|        8|        7.525|
|fashion_mnist-large|10000|2929303|       16|       11.437|
|adult-small        |   10|     80|        3|        3.000|
|adult-med          |  100|  13067|        8|        7.637|
|adult-large        | 1000| 642883|       16|       13.202|

|       model       |test_rows|cpu_time(s)|cpu_std |gpu_time(s)|gpu_std |speedup|
|-------------------|--------:|----------:|-------:|----------:|-------:|------:|
|covtype-small      |    10000|    0.03719|0.016989|    0.01637|0.006701| 2.2713|
|covtype-med        |    10000|    8.24571|0.065573|    0.45239|0.026825|18.2271|
|covtype-large      |    10000|  930.22357|0.555459|   50.88014|0.205488|18.2826|
|cal_housing-small  |    10000|    0.00708|0.005291|    0.00737|0.005849| 0.9597|
|cal_housing-med    |    10000|    1.27267|0.021711|    0.08722|0.019198|14.5912|
|cal_housing-large  |    10000|  315.20877|0.298429|   16.91054|0.343210|18.6398|
|fashion_mnist-small|    10000|    0.35401|0.142973|    0.16965|0.039150| 2.0866|
|fashion_mnist-med  |    10000|   15.10363|0.073838|    1.13051|0.084911|13.3600|
|fashion_mnist-large|    10000|  621.13735|0.144418|   47.53092|0.174141|13.0681|
|adult-small        |    10000|    0.00667|0.003201|    0.00620|0.005009| 1.0765|
|adult-med          |    10000|    1.13609|0.004031|    0.07788|0.010203|14.5882|
|adult-large        |    10000|   88.12258|0.198140|    4.66934|0.004628|18.8726|

## Memory usage
GPUTreeShap uses very little working GPU memory, only allocating space proportional to the model size. An application is far more likely to be constrained by the size of the dataset.

## Usage
See examples for sample integration into a C++ decision tree project. GPUTreeShap accepts a decision tree ensemble in the form of a list of unique paths through all branches of the tree, as well as an interface to a dataset allocated on the GPU, and returns feature contributions for each row in the dataset.

## References
[1] Lundberg, Scott M., Gabriel G. Erion, and Su-In Lee. "Consistent individualized feature attribution for tree ensembles." arXiv preprint arXiv:1802.03888 (2018).

[2] Sundararajan, Mukund, Kedar Dhamdhere, and Ashish Agarwal. "The Shapley Taylor Interaction Index." International Conference on Machine Learning. PMLR, 2020.

[3] https://drafts.distill.pub/HughChen/its_blog/
