---
description: A machine learning approach that uses unlabeled data
---

# Unsupervised Learning

PostgresML supports several clustering algorithms for unsupervised learning. Models can be trained using `pgml.train` on unlabeled data to identify groups within the data.

## Training

To build clusters on a given dataset, we can use the table or a view. Since clustering is an unsupervised algorithm, we don't need a column that represents a label as one of the inputs to `pgml.train`.

## API

In `pgml.train` you need to set `cluster` as task and pass a `project_name`. Most  parameters are optional.&#x20;

```sql
pgml.train(
    project_name TEXT,
    task TEXT DEFAULT NULL,
    relation_name TEXT DEFAULT NULL,
    algorithm TEXT DEFAULT 'linear',
    hyperparams JSONB DEFAULT '{}'::JSONB
)
```

## Algorithms

| Algorithm              | Reference                                                                                                         |
| ---------------------- | ----------------------------------------------------------------------------------------------------------------- |
| `affinity_propagation` | [AffinityPropagation](https://scikit-learn.org/stable/modules/generated/sklearn.cluster.AffinityPropagation.html) |
| `birch`                | [Birch](https://scikit-learn.org/stable/modules/generated/sklearn.cluster.Birch.html)                             |
| `kmeans`               | [K-Means](https://scikit-learn.org/stable/modules/generated/sklearn.cluster.KMeans.html)                          |
| `mini_batch_kmeans`    | [MiniBatchKMeans](https://scikit-learn.org/stable/modules/generated/sklearn.cluster.MiniBatchKMeans.html)         |

### Example

This example trains models on the sklean digits dataset -- which is a copy of the test set of the [UCI ML hand-written digits datasets](https://archive.ics.uci.edu/ml/datasets/Optical+Recognition+of+Handwritten+Digits). This demonstrates using a table with a single array feature column for clustering. You could do something similar with a vector column.

```sql
SELECT pgml.load_dataset('digits');

-- create an unlabeled table of the images for unsupervised learning
CREATE VIEW pgml.digit_vectors AS
SELECT image FROM pgml.digits;

-- view the dataset
SELECT left(image::text, 40) || ',...}' FROM pgml.digit_vectors LIMIT 10;

-- train a simple model to classify the data
SELECT * FROM pgml.train('Handwritten Digit Clusters', 'cluster', 'pgml.digit_vectors', hyperparams => '{"n_clusters": 10}');

-- check out the predictions
SELECT target, pgml.predict('Handwritten Digit Clusters', image) AS prediction
FROM pgml.digits
LIMIT 10;
```

### Other Algorithms

```sql
SELECT * FROM pgml.train('Handwritten Digit Clusters', algorithm => 'affinity_propagation');
SELECT * FROM pgml.train('Handwritten Digit Clusters', algorithm => 'birch', hyperparams => '{"n_clusters": 10}');
SELECT * FROM pgml.train('Handwritten Digit Clusters', algorithm => 'kmeans', hyperparams => '{"n_clusters": 10}');
SELECT * FROM pgml.train('Handwritten Digit Clusters', algorithm => 'mini_batch_kmeans', hyperparams => '{"n_clusters": 10}');
```
