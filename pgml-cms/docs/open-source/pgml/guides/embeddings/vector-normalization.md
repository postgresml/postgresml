# Vector Normalization

Vector normalization converts a vector into a unit vector — that is, a vector that retains the same direction but has a magnitude (or length) of 1. This process is essential for various computational techniques where the magnitude of a vector may influence the outcome undesirably, such as when calculating the inner product instead of cosine similarity or when needing to compare vectors based solely on direction.

## Purpose and Benefits

- **Cosine Similarity**: In machine learning and data science, normalized vectors are crucial when using the inner product, instead of the more expensive cosine similarity metric. Inner product inherently requires vectors of unit length to accurately measure angles between vectors. L2 Normalized vectors indexed with the inner product can reduce computational complexity 3x in the inner loop compared to cosine similarity, while yielding otherwise identical results. 

- **Directionality**: Normalization strips away the magnitude of the vector, leaving a descriptor of direction only. This is useful when direction matters more than length, such as in feature scaling in machine learning where you want to normalize features to have equal influence regardless of their absolute values.

- **Stability in Computations**: When vectors are normalized, numerical computations involving them are often more stable and less susceptible to problems due to very large or very small scale factors.

## Storing and Normalizing Data

Assume you've created a table in your database that stores embeddings generated using [pgml.embed()](/docs/open-source/pgml/api/pgml.embed), although you can normalize any vector.

```postgresql
CREATE TABLE documents (
   id SERIAL PRIMARY KEY,
   body TEXT,
   embedding FLOAT[] GENERATED ALWAYS AS (pgml.embed('intfloat/e5-small-v2', body)) STORED
);
```

Example of inserting text and its corresponding embedding

```postgresql
INSERT INTO documents (body)
VALUES -- embedding vectors are automatically generated
    ('Example text data'),
    ('Another example document'),
    ('Some other thing');
```

You could create a new table from your documents and their embeddings, that uses normalized embeddings.  

```postgresql
CREATE TABLE documents_normalized_vectors AS 
SELECT 
    id AS document_id, 
    pgml.normalize_l2(embedding) AS normalized_l2_embedding
FROM documents;
```

Another valid approach would be to just store the normalized embedding in the documents table.

```postgresql
CREATE TABLE documents (
   id SERIAL PRIMARY KEY,
   body TEXT,
   embedding FLOAT[] GENERATED ALWAYS AS (pgml.normalize_l2(pgml.embed('intfloat/e5-small-v2', body))) STORED
);
```

## Normalization Functions
   Normalization is critical for ensuring that the magnitudes of feature vectors do not distort the performance of machine learning algorithms.

- **L1 Normalization (Manhattan Norm)**: This function scales the vector so that the sum of the absolute values of its components is equal to 1. It's useful when differences in magnitude are important but the components represent independent dimensions.
    
    ```postgresql
    SELECT pgml.normalize_l1(embedding) FROM documents;
    ```
  
- **L2 Normalization (Euclidean Norm)**: Scales the vector so that the sum of the squares of its components is equal to 1. This is particularly important for cosine similarity calculations in machine learning.

    ```postgresql
    SELECT pgml.normalize_l2(embedding) FROM documents;
    ```
  
- **Max Normalization**: Scales the vector such that the maximum absolute value of any component is 1. This normalization is less common but can be useful when the maximum value represents a bounded capacity.

    ```postgresql
    SELECT pgml.normalize_max(embedding) FROM documents;
    ```

## Querying and Using Normalized Vectors
   After normalization, you can use these vectors for various applications, such as similarity searches, clustering, or as input for further machine learning models within PostgresML.

```postgresql
-- Querying for similarity using l2 normalized dot product, which is equivalent to cosine similarity
WITH normalized_vectors AS (
   SELECT id, pgml.normalize_l2(embedding) AS norm_vector
   FROM documents
)
SELECT a.id, b.id, pgml.dot_product(a.norm_vector, b.norm_vector)
FROM normalized_vectors a, normalized_vectors b
WHERE a.id <> b.id;
```

## Considerations and Best Practices
   
- **Performance**: Normalization can be computationally intensive, especially with large datasets. Consider batch processing and appropriate indexing.
- **Storage**: Normalized vectors might not need to be persisted if they are only used transiently, which can save storage or IO latency.
