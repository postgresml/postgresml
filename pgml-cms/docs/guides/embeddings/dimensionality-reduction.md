# Dimensionality Reduction

In the case of embedding models trained on large bodies of text, most of the concepts they learn will be unused when
dealing with any single piece of text. For collections of documents that deal with specific topics, only a fraction of
the language models learned associations will be relevant. Dimensionality reduction is an important technique to improve
performance _on your documents_, both in terms of quality and latency for embedding recall using nearest neighbor
search.

## Why Dimensionality Reduction?

- **Improved Performance**: Reducing the number of dimensions can significantly improve the computational efficiency of
  machine learning algorithms.
- **Reduced Storage**: Lower-dimensional data requires less storage space.
- **Enhanced Visualization**: It is easier to visualize data in two or three dimensions.

## What is Matrix Decomposition?

Dimensionality reduction is a key technique in machine learning and data analysis, particularly when dealing with
high-dimensional data such as embeddings. A table full of embeddings can be considered a matrix, aka a 2-dimensional
array with rows and columns, where the embedding dimensions are the columns. We can use matrix decomposition methods,
such as Principal Component Analysis (PCA) and Singular Value Decomposition (SVD), to reduce the dimensionality of
embeddings.

Matrix decomposition involves breaking down a matrix into simpler, constituent matrices. The most common decomposition
techniques for this purpose are:

- **Principal Component Analysis (PCA)**: Reduces dimensionality by projecting data onto a lower-dimensional subspace
  that captures the most variance.
- **Singular Value Decomposition (SVD)**: Factorizes a matrix into three matrices, capturing the essential features in a
  reduced form.

## Dimensionality Reduction with PostgresML

PostgresML allows in-database execution of matrix decomposition techniques, enabling efficient dimensionality reduction
directly within the database environment.

## Step-by-Step Guide to Using Matrix Decomposition

### Preparing the data

We'll create a set of embeddings using modern embedding model with 384 dimensions.

```postgresql
CREATE TABLE documents_with_embeddings
(
    id        serial PRIMARY KEY,
    body      text,
    embedding float[] GENERATED ALWAYS AS (pgml.normalize_l2(pgml.embed('intfloat/e5-small-v2', body))) STORED
);
```

!!! generic

!!! code_block time="46.823"

```postgresql
INSERT INTO documents_with_embeddings (body)
VALUES -- embedding vectors are automatically generated
       ('Example text data'),
       ('Another example document'),
       ('Some other thing'),
       ('We need a few more documents'),
       ('At least as many documents as dimensions in the reduction'),
       ('Which normally isn''t a problem'),
       ('Unless you''re typing out a bunch of demo data');
```

!!!

!!! results

```postgresql
INSERT 0 3
```

!!!

!!!

!!! generic

!!! code_block time="14.259ms"

```postgresql
CREATE VIEW just_embeddings AS
SELECT embedding
FROM documents_with_embeddings;
```

!!!

!!! results

```postgresql
 CREATE VIEW
```

!!!

!!!

### Decomposition

Models can be trained using `pgml.train` on unlabeled data to identify important features within the data. To decompose
a dataset into it's principal components, we can use the table or a view. Since decomposition is an unsupervised
algorithm, we don't need a column that represents a label as one of the inputs to `pgml.train`.

Train a simple model to find reduce dimensions for 384, to the 3:

!!! generic

!!! code_block time="48.087 ms"

```postgresql
SELECT *
FROM pgml.train('Embedding Components', 'decomposition', 'just_embeddings', hyperparams => '{"n_components": 3}');
```

!!!

!!! results

```postgresql
INFO:  Metrics: {"cumulative_explained_variance": 0.69496775, "fit_time": 0.008234134, "score_time": 0.001717504}
INFO:  Deploying model id: 2

       project        |     task      | algorithm | deployed
----------------------+---------------+-----------+----------
 Embedding Components | decomposition | pca       | t
```

!!!

!!!

Note that the input vectors have been reduced from 384 dimensions to 3 that explain 69% of the variance across all
samples. That's a more than 100x size reduction, while preserving 69% of the information. These 3 dimensions may be
plenty for a course grained first pass ranking with a vector database distance function, like cosine similarity. You can
then choose to use the full embeddings, or some other reduction, or the raw text with a reranker model to improve final
relevance over the baseline with all the extra time you have now that you've reduced the cost of initial nearest
neighbor recall 100x.

You can check out the components for any vector in this space using the reduction model:

!!! generic

!!! code_block time="14.259ms"

```postgresql
SELECT pgml.decompose('Embedding Components', embedding) AS pca
FROM just_embeddings
LIMIT 10;
```

!!!

!!! results

```postgresql
 CREATE VIEW
```

!!!

!!!

Exercise for the reader: Where is the sweet spot for number of dimensions, yet preserving say, 99% of the relevance
data? How much of the cumulative explained variance do you need to preserve 100% to return the top N results for the
reranker, if you feed the reranker top K using cosine similarity or another vector distance function?
