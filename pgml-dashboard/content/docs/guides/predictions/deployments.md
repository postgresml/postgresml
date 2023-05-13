# Deployments

A model is automatically deployed and used for predictions if its key metric (<i>R<sup>2</sup></i> for regression, <i>F<sub>1</sub></i> for classification) is improved during training over the previous version. Alternatively, if you want to manage deploys manually, you can always change which model is currently responsible for making predictions.


## API

```postgresql title="pgml.deploy()"
pgml.deploy(
	project_name TEXT,
	strategy TEXT DEFAULT 'best_score',
	algorithm TEXT DEFAULT NULL
)
```

### Parameters

| Parameter | Description | Example |
|-----------|-------------|---------|
| `project_name` | The name of the project used in `pgml.train()` and `pgml.predict()`. | `My First PostgresML Project` |
| `strategy` | The deployment strategy to use for this deployment. | `rollback` |
| `algorithm`  | Restrict the deployment to a specific algorithm. Useful when training on multiple algorithms and hyperparameters at the same time. | `xgboost` |


#### Strategies

There are 3 different deployment strategies available:

| Strategy | Description |
|----------|-------------|
| `most_recent` | The most recently trained model for this project is immediately deployed, regardless of metrics. |
| `best_score` | The model that achieved the best key metric score is immediately deployed. |
| `rollback` | The model that was last deployed for this project is immediately redeployed, overriding the currently deployed model. |

The default deployment behavior allows any algorithm to qualify. It's automatically used during training, but can be manually executed as well:

=== "SQL"

```postgresql
SELECT * FROM pgml.deploy(
	'Handwritten Digit Image Classifier',
	strategy => 'best_score'
);
```

=== "Output"

```
              project               |  strategy  | algorithm
------------------------------------+------------+-----------
 Handwritten Digit Image Classifier | best_score | xgboost
(1 row)
```

===

#### Specific Algorithms

Deployment candidates can be restricted to a specific algorithm by including the `algorithm` parameter. This is useful when you're training multiple algorithms using different hyperparameters and want to restrict the deployment a single algorithm only:

=== "SQL"

```postgresql
SELECT * FROM pgml.deploy(
    project_name => 'Handwritten Digit Image Classifier', 
    strategy => 'best_score', 
    algorithm => 'svm'
);
```

=== "Output"

```
            project_name            |    strategy    | algorithm
------------------------------------+----------------+----------------
 Handwritten Digit Image Classifier | classification | svm
(1 row)
```

===

## Rolling Back

In case the new model isn't performing well in production, it's easy to rollback to the previous version. A rollback creates a new deployment for the old model. Multiple rollbacks in a row will oscillate between the two most recently deployed models, making rollbacks a safe and reversible operation.

=== "Rollback 1"

```sql linenums="1"
SELECT * FROM pgml.deploy(
	'Handwritten Digit Image Classifier',
	strategy => 'rollback'
);
```

=== "Output"

```
             project               | strategy | algorithm
------------------------------------+----------+-----------
 Handwritten Digit Image Classifier | rollback | linear
(1 row)
```

=== "Rollback 2"

```postgresql
SELECT * FROM pgml.deploy(
	'Handwritten Digit Image Classifier',
	strategy => 'rollback'
);
```

=== "Output"

```
              project               | strategy | algorithm
------------------------------------+----------+-----------
 Handwritten Digit Image Classifier | rollback | xgboost
(1 row)
```

===
