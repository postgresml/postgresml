# Dimensionality Reduction

In the case of embedding models trained on large bodies of text, most of the concepts they learn will be unused when dealing with any single piece of text. For collections of documents that deal with specific topics, only a fraction of the language models learned associations will be relevant. Dimensionality reduction is an important technique to improve performance _on your documents_, both in terms of quality and latency for embedding recall using nearest neighbor search.

## Why Dimensionality Reduction?

- **Improved Performance**: Reducing the number of dimensions can significantly improve the computational efficiency of machine learning algorithms.
- **Reduced Storage**: Lower-dimensional data requires less storage space.
- **Enhanced Visualization**: It is easier to visualize data in two or three dimensions.

## What is Matrix Decomposition?
Dimensionality reduction is a key technique in machine learning and data analysis, particularly when dealing with high-dimensional data such as embeddings. A table full of embeddings can be considered a matrix, aka a 2-dimensional array with rows and columns, where the embedding dimensions are the columns. We can use matrix decomposition methods, such as Principal Component Analysis (PCA) and Singular Value Decomposition (SVD), to reduce the dimensionality of embeddings. 
 
Matrix decomposition involves breaking down a matrix into simpler, constituent matrices. The most common decomposition techniques for this purpose are:

- **Principal Component Analysis (PCA)**: Reduces dimensionality by projecting data onto a lower-dimensional subspace that captures the most variance.
- **Singular Value Decomposition (SVD)**: Factorizes a matrix into three matrices, capturing the essential features in a reduced form.

## Dimensionality Reduction with PostgresML
PostgresML allows in-database execution of matrix decomposition techniques, enabling efficient dimensionality reduction directly within the database environment.

## Step-by-Step Guide to Using Matrix Decomposition

1. **Preparing the data**  
   We'll create a set of embeddings using modern embedding model with 384 dimensions.

   ```postgresql
   CREATE TABLE documents_with_embeddings (
   id SERIAL PRIMARY KEY,
   body TEXT,
   embedding FLOAT[] GENERATED ALWAYS AS (pgml.normalize_l2(pgml.embed('intfloat/e5-small-v2', body))) STORED
   );
   ```
    
   !!! generic
    
   !!! code_block time="46.823"
    
   ```postgresql
   INSERT INTO documents_with_embeddings (body)
   VALUES -- embedding vectors are automatically generated
       ('Example text data'),
       ('Another example document'),
       ('Some other thing');
   ```
    
   !!!
    
   !!! results
    
   ```postgresql
   INSERT 0 3
   ```
    
   !!!
    
   !!!

2. 

Ensure that your data is loaded into the Postgres database and is in a suitable format for decomposition. For example, we'll treat the if you have embeddings stored in a table:

```postgresql
SELECT pgml.load_dataset('digits');
```

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















```sql
SELECT * FROM embeddings_table;
## Introduction

## Principal Component Analysis


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

All decomposition algorithms implemented by PostgresML are online versions. You may use the [pgml.decompose](../../../api/sql-extension/pgml.decompose "mention") function to decompose novel data points after the model has been trained.

| Algorithm                 | Reference                                                                                                           |
|---------------------------|---------------------------------------------------------------------------------------------------------------------|
| `pca` | [PCA](https://scikit-learn.org/stable/modules/generated/sklearn.decomposition.PCA.html) |

### Examples

```postgresql
SELECT * FROM pgml.train('Handwritten Digit Clusters', algorithm => 'pca', hyperparams => '{"n_components": 10}');
```
