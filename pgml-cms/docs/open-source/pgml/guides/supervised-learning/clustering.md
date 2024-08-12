# Clustering

Models can be trained using `pgml.train` on unlabeled data to identify groups within the data. To build clusters on a given dataset, we can use the table or a view. Since clustering is an unsupervised algorithm, we don't need a column that represents a label as one of the inputs to `pgml.train`.

## Example

This example trains models on the sklearn digits dataset -- which is a copy of the test set of the [UCI ML hand-written digits datasets](https://archive.ics.uci.edu/ml/datasets/Optical+Recognition+of+Handwritten+Digits). This demonstrates using a table with a single array feature column for clustering. You could do something similar with a vector column.

```postgresql
SELECT pgml.load_dataset('digits');

-- create an unlabeled table of the images for unsupervised learning
CREATE VIEW pgml.digit_vectors AS
SELECT image FROM pgml.digits;

-- view the dataset
SELECT left(image::text, 40) || ',...}' FROM pgml.digit_vectors LIMIT 10;

-- train a simple model to cluster the data
SELECT * FROM pgml.train('Handwritten Digit Clusters', 'clustering', 'pgml.digit_vectors', hyperparams => '{"n_clusters": 10}');

-- check out the predictions
SELECT target, pgml.predict('Handwritten Digit Clusters', image) AS prediction
FROM pgml.digits
LIMIT 10;
```

## Algorithms

All clustering algorithms implemented by PostgresML are online versions. You may use the [pgml.predict](/docs/open-source/pgml/api/pgml.predict/ "mention")function to cluster novel data points after the clustering model has been trained.

| Algorithm              | Reference                                                                                                         |
| ---------------------- | ----------------------------------------------------------------------------------------------------------------- |
| `affinity_propagation` | [AffinityPropagation](https://scikit-learn.org/stable/modules/generated/sklearn.cluster.AffinityPropagation.html) |
| `birch`                | [Birch](https://scikit-learn.org/stable/modules/generated/sklearn.cluster.Birch.html)                             |
| `kmeans`               | [K-Means](https://scikit-learn.org/stable/modules/generated/sklearn.cluster.KMeans.html)                          |
| `mini_batch_kmeans`    | [MiniBatchKMeans](https://scikit-learn.org/stable/modules/generated/sklearn.cluster.MiniBatchKMeans.html)         |

### Examples

```postgresql
SELECT * FROM pgml.train('Handwritten Digit Clusters', algorithm => 'affinity_propagation');
SELECT * FROM pgml.train('Handwritten Digit Clusters', algorithm => 'birch', hyperparams => '{"n_clusters": 10}');
SELECT * FROM pgml.train('Handwritten Digit Clusters', algorithm => 'kmeans', hyperparams => '{"n_clusters": 10}');
SELECT * FROM pgml.train('Handwritten Digit Clusters', algorithm => 'mini_batch_kmeans', hyperparams => '{
```
