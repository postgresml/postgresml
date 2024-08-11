# Decomposition

Models can be trained using `pgml.train` on unlabeled data to identify important features within the data. To decompose a dataset into it's principal components, we can use the table or a view. Since decomposition is an unsupervised algorithm, we don't need a column that represents a label as one of the inputs to `pgml.train`.

## Example

This example trains models on the sklearn digits dataset -- which is a copy of the test set of the [UCI ML hand-written digits datasets](https://archive.ics.uci.edu/ml/datasets/Optical+Recognition+of+Handwritten+Digits). This demonstrates using a table with a single array feature column for principal component analysis. You could do something similar with a vector column.

```postgresql
SELECT pgml.load_dataset('digits');

-- create an unlabeled table of the images for unsupervised learning
CREATE VIEW pgml.digit_vectors AS
SELECT image FROM pgml.digits;

-- view the dataset
SELECT left(image::text, 40) || ',...}' FROM pgml.digit_vectors LIMIT 10;

-- train a simple model to cluster the data
SELECT * FROM pgml.train('Handwritten Digit Components', 'decomposition', 'pgml.digit_vectors', hyperparams => '{"n_components": 3}');

-- check out the compenents
SELECT target, pgml.decompose('Handwritten Digit Components', image) AS pca
FROM pgml.digits
LIMIT 10;
```

Note that the input vectors have been reduced from 64 dimensions to 3, which explain nearly half of the variance across all samples.

## Algorithms

All decomposition algorithms implemented by PostgresML are online versions. You may use the [pgml.decompose](/docs/open-source/pgml/api/pgml.decompose "mention") function to decompose novel data points after the model has been trained.

| Algorithm                 | Reference                                                                                                           |
|---------------------------|---------------------------------------------------------------------------------------------------------------------|
| `pca` | [PCA](https://scikit-learn.org/stable/modules/generated/sklearn.decomposition.PCA.html) |

### Examples

```postgresql
SELECT * FROM pgml.train('Handwritten Digit Clusters', algorithm => 'pca', hyperparams => '{"n_components": 10}');
```
