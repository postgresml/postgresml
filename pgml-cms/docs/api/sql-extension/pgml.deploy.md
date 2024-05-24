---
description: >-
  Release trained models when ML quality metrics computed during training
  improve. Track model deployments over time and rollback if needed.
---

# pgml.deploy()

## Deployments

A model is automatically deployed and used for predictions if its key metric (_R2_ for regression, _F1_ for classification) is improved during training over the previous version. Alternatively, if you want to manage deploys manually, you can always change which model is currently responsible for making predictions.

## API

```postgresql
pgml.deploy(
    project_name TEXT,
    strategy TEXT DEFAULT 'best_score',
    algorithm TEXT DEFAULT NULL
)
```

### Parameters

| Parameter      | Example                         | Description                                                                                                                        |
| -------------- | ------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| `project_name` | `'My First PostgresML Project'` | The name of the project used in `pgml.train()` and `pgml.predict()`.                                                               |
| `strategy`     | `'rollback'`                    | The deployment strategy to use for this deployment.                                                                                |
| `algorithm`    | `'xgboost'`                     | Restrict the deployment to a specific algorithm. Useful when training on multiple algorithms and hyperparameters at the same time. |

### **Strategies**

There are 3 different deployment strategies available:

| Strategy      | Description                                                                                      |
| ------------- | ------------------------------------------------------------------------------------------------ |
| `most_recent` | The most recently trained model for this project is immediately deployed, regardless of metrics. |
| `best_score`  | The model that achieved the best key metric score is immediately deployed.                       |
| `rollback`    | The model that was deployed before to the current one is deployed.                               |

The default deployment behavior allows any algorithm to qualify. It's automatically used during training, but can be manually executed as well:

## Examples

### Deploying the best score

#### SQL

```postgresql
SELECT * FROM pgml.deploy(
   'Handwritten Digit Image Classifier',
    strategy => 'best_score'
);
```

#### Output

```postgresql
              project               |  strategy  | algorithm
------------------------------------+------------+-----------
 Handwritten Digit Image Classifier | best_score | xgboost
(1 row)
```

### **Specific Algorithms**

Deployment candidates can be restricted to a specific algorithm by including the `algorithm` parameter. This is useful when you're training multiple algorithms using different hyperparameters and want to restrict the deployment a single algorithm only:

#### SQL

```postgresql
SELECT * FROM pgml.deploy(
    project_name => 'Handwritten Digit Image Classifier', 
    strategy => 'best_score', 
    algorithm => 'svm'
);
```

#### Output

```postgresql
            project_name            |    strategy    | algorithm
------------------------------------+----------------+----------------
 Handwritten Digit Image Classifier | classification | svm
(1 row)
```

### Rolling Back

In case the new model isn't performing well in production, it's easy to rollback to the previous version. A rollback creates a new deployment for the old model. Multiple rollbacks in a row will oscillate between the two most recently deployed models, making rollbacks a safe and reversible operation.

#### Rollback

```postgresql
SELECT * FROM pgml.deploy(
	'Handwritten Digit Image Classifier',
	strategy => 'rollback'
);
```

#### Output

```postgresql
             project               | strategy | algorithm
------------------------------------+----------+-----------
 Handwritten Digit Image Classifier | rollback | linear
(1 row)
```

#### Rollback again

Rollbacks are actually new deployments, so issuing two rollbacks in a row will leave you back with the original model, making rollback safely undoable.

```postgresql
SELECT * FROM pgml.deploy(
	'Handwritten Digit Image Classifier',
	strategy => 'rollback'
);
```

#### Output

```postgresql
              project               | strategy | algorithm
------------------------------------+----------+-----------
 Handwritten Digit Image Classifier | rollback | xgboost
(1 row)
```

### Specific Model IDs

In the case you need to deploy an exact model that is not the `most_recent` or `best_score`, you may deploy a model by id. Model id's can be found in the `pgml.models` table.

#### SQL

```postgresql
SELECT * FROM pgml.deploy(12);
```

#### Output

```postgresql
              project               | strategy | algorithm
------------------------------------+----------+-----------
 Handwritten Digit Image Classifier | specific | xgboost
(1 row)
```
