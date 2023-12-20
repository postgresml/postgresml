---
description: Store, index and query vectors, with pgvector
---

# Vector Database

Vectors are lists of numbers representing a measurement in multidimensional space. There are many types of vectors, e.g. embeddings used for vector search, but ultimately they are all just arrays of floating points.

In Postgres, a vector is just another data type that can be stored in regular tables and queried together with other columns. At PostgresML, we're using `pgvector`, a Postgres extension that implements the `vector` data type, many vector operations like inner product and cosine distance, and approximate nearest neighbor (ANN) search.

### Installing pgvector

If you're using our Cloud or our Docker image, your database has `pgvector` installed already. If you're self-hosting PostgresML, take a look at our [Self-hosting](../resources/developer-docs/self-hosting/) documentation.

### Storing vectors

Vectors can be stored in columns, just like any other data type. To add a vector column to your table, you need to specify the size of the vector. All vectors in a single column must be the same size since there are no mathematical operations to compare vectors of different sizes.

#### Adding a vector column

Using the example from [Tabular data](../resources/data-storage-and-retrieval/tabular-data.md), let's add a vector column to our USA House Prices table:

```sql
ALTER TABLE usa_house_prices
ADD COLUMN embedding VECTOR(384);
```

At first, the column is empty. To get some vectors, let's use the PostgresML `pgml.embed()` function and generate an embedding of the "Address" column. This is where machine learning inside the database really shines:

```sql
UPDATE
    usa_house_prices
SET embedding = pgml.embed('intfloat/e5-small', "Address");
```

```sql
UPDATE 5000
```

That's it. We just embedding 5,000 "Address" values with a single SQL query. Let's take a look at what we got:

```sql
SELECT
    "Address",
    (embedding::real[])[1:5] 
FROM usa_house_prices
WHERE
    "Address" = '1 Infinite Loop, Cupertino, California';
    
                Address                 |                           embedding                            
----------------------------------------+----------------------------------------------------------------
 1 Infinite Loop, Cupertino, California | {-0.009034249,-0.055827666,-0.09911688,0.005093358,0.04053181}
(1 row)
```

The vectors contain 384 values each, but that won't fit on our screen, so we're selecting the first 5 values using the Postgres array slice notation `[1:5]`. Fun fact: Postgres array indices start at one, not zero.

### Searching vectors

If your dataset is small enough, searching vectors doesn't require approximation. You can find the exact nearest neighbor match using any of the distance functions supported by `pgvector`: L2, cosine distance, inner product and cosine similarity.

Each distance function is implemented with its own operator and can be used in any SQL query:

| Distance function | Operator        | Index operator      |
| ----------------- | --------------- | ------------------- |
| L2                | `<->`           | `vector_in_ops`     |
| Inner product     | `<#>`           | `vector_l2_ops`     |
| Cosine distance   | `<=>`           | `vector_cosine_ops` |
| Cosine similarity | `1 - (a <=> b)` | `vector_cosine_ops` |

For example, let's find three (3) closest matching address to `1 Infinite Loop` using cosine distance:

```sql
SELECT
    "Address"
FROM usa_house_prices
ORDER BY 
    embedding <=> pgml.embed('intfloat/e5-small', '1 Infinite Loop')::vector(384)
LIMIT 3;
```

```sql
                Address                 
----------------------------------------
 1 Infinite Loop, Cupertino, California
 615 Larry Loop                        
 Warrenberg, PR 37943
(5 rows)
```

This query uses `pgml.embed()` to generate an embedding on the fly and finds the exact closest neighbors to that embedding in the entire USA House Prices dataset.

### Approximate nearest neighbors

This dataset only has 5,000 rows which, for Postgres, is really easy to scan. In the real world, these datasets grow to become very large and searching the entire table becomes too slow to be practical. When that happens, we can get closest matches using approximation. Approximate nearest neighbors, or ANN, is a commonly used technique to organize vectors to be able to find results that are "close enough".

`pgvector` implements two ANN algorithms: IVFFlat and HNSW. Both have their pros and cons and can be used in production to search millions of vectors.

### IVFFlat

IVFFlat splits the list of vectors into roughly equal parts, grouped around centroids calculated using k-nearest neighbors (KNN). Once split, the lists are stored in a B-tree index, ordered by the centroid.

When searching for a nearest neighbor match, `pgvector` picks the closest centroid to the candidate vector, fetches all the vectors from the list, sorts them, and fetches the closest neighbors. Since the list represents only a fraction of all the vectors, using an IVFFlat index is considerably faster than scanning the entire table.

The number of lists in an IVFFlat index is configurable when creating the index. The more lists are created, the faster you can search it, but the nearest neighbor approximation becomes less precise. The best number of lists for a dataset is typically its square root, e.g. if a dataset has 5,000,000 vectors, the number of lists should be:

```sql
SELECT round(sqrt(5000000)) AS lists;
 lists 
-------
  2236
```

#### Creating an IVFFlat index

You can create an IVFFlat index with just one query:

```sql
CREATE INDEX ON
    usa_house_prices
USING ivfflat(embedding vector_cosine_ops)
WITH (lists = 71);
```

71 is the approximate square root of 5,000 rows we have in that table. With the index created, if we `EXPLAIN` the query we just ran, we'll get an "Index Scan" on the cosine distance index:

```sql
EXPLAIN SELECT
    "Address"
FROM usa_house_prices
ORDER BY 
    embedding <=> pgml.embed('intfloat/e5-small', '1 Infinite Loop')::vector(384)
LIMIT 3;

Limit  (cost=38.03..38.32 rows=3 width=55)
   ->  Index Scan using usa_house_prices_embedding_idx on usa_house_prices  (cost=38.03..327.23 rows=5001 width=55)
         Order By: (embedding <=> '[-0.033770584,-0.033374995, ...])
```

It's important to create an IVFFlat index after you have added a representative sample of vectors into your table. Without a representative sample, the calculated centroids will be incorrect and the approximation of nearest neighbors inaccurate.

#### Maintaining an IVFFlat index

IVFFlat is a simple algorithm and constructs an index quickly. Splitting, sorting and solving KNN is optimized using the Postgres query engine and vectorized CPU operations (e.g. AVX512 on modern CPUs) built into `pgvector`. When queried, the index provides good recall acceleration and approximation for typical use cases.

On the other hand, because of the nature of centroids, if the dataset changes significantly, the original KNN calculation becomes inaccurate. In that case, an IVFFlat index should be rebuilt which Postgres makes pretty easy:

```sql
REINDEX INDEX CONCURRENTLY usa_house_prices_embedding_idx;
```

As of this writing, `pgvector` doesn't provide monitoring tools for index degradation. The application user should monitor recall from their vector search operations, and if the recall starts dropping, issue a reindex.

### HNSW

Home Navigable Small Worlds, or HNSW, is a modern ANN algorithm that constructs a multilayer graph using a greedy search with local minimums. Constructing HNSW requires multiple passes over the same data, so the time and memory cost of building it are higher, but it does have faster and better recall than IVFFlat.

#### Creating an HNSW index

You can create an HNSW index with just one query:

```sql
CREATE INDEX ON
    usa_house_prices
USING hnsw(embedding vector_cosine_ops);
```

#### Maintaining an HNSW index

HNSW requires much less maintenance than IVFFlat. When new vectors are added, they are automatically inserted at the optimal place in the graph. However, as the graph gets bigger, rebalancing it becomes more expensive, and inserting new rows becomes slower.

We address this trade-off and how to solve this problem in [Partitioning](../resources/data-storage-and-retrieval/partitioning.md).

###
