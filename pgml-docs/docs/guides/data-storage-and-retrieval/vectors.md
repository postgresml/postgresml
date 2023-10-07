# Vectors

Vectors are lists of numbers representing a measurement in multidimensional space. There are many types of vectors, e.g. embeddings used in Large Language Models, but ultimately they are all just arrays of floating points.

In Postgres, a vector is just another data type that can be stored and queried together with other columns and tables. At PostgresML, we use `pgvector`, a Postgres extension that implements approximate nearest neighbor (ANN) search algorithms IVFFlat and Hierarchical Navigable Small Worlds (HNSW).

### Installing pgvector

If you're using our Cloud or our Docker image, your database already has `pgvector` installed. If you're self-hosting PostgresML, take a look at our [Self-hosting](../deploying-postgresml/self-hosting/) documentation for installation instructions.

### Storing vectors

Vectors can be stored in columns, just like any other data type. To add a vector column to your table, you need to specify the size of the vector. All vectors in a single column must be the same size, because there is no mathematical operation to compare vectors of different sizes.

#### Adding a vector column

Using the example from our Tabular data guide, let's add a vector column to our USA House Prices table:

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

```
UPDATE 5000
```

That's it. We just embedding 5,000 "Address" values with a single SQL query. Let's take a look at what we got:

```
postgresml=# SELECT
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

The vectors contain 384 values each, but that won't fit on our screen, so we're selecting the first 5 values using the Postgres array slice notation `[1:5]`. Fun fact, Postgres arrays indices start at one, not zero.

### Searching vectors

If your dataset is small enough, searching vectors does not require approximation. You can find the exact nearest neighbor match using any of the distance functions supported by `pgvector`: L2, cosine distance, inner product and cosine similarity.

Each distance function is implemented with its own operator and can be used in any SQL query.

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

```
                Address                 
----------------------------------------
 1 Infinite Loop, Cupertino, California
 615 Larry Loop                        
 Warrenberg, PR 37943
(5 rows)
```

This query uses `pgml.embed()` to generate an embedding on the fly and finds the closest neighbors to that embedding in the entire US House Prices dataset.

### Approximate nearest neighbors

This dataset only has 5,000 rows which for Postgres is really easy to scan. In the real world, these datasets grow to become very large and searching the entire table becomes too slow to be practical. Nonetheless, we can get closest matches using approximation. Approximate nearest neighbors, or ANN, is a commonly used technique to organize vectors in a way as be able to find results that are "close enough".

`pgvector` implements two ANN algorithms: IVFlat and HNSW. Both have their pros and cons and can be used in production to search millions of vectors efficiently.

### IVFFlat

IVFFlat splits the vectors into roughly equal lists, grouped by centroids calculated using k-nearest neighbors, or KNN. Once split, the lists are stored in a B-tree index ordered by the centroid.

When searching for a nearest neighbor match, `pgvector` picks the closest centroid to the vector, fetches all the values from the list, sorts them, and fetches the closest neighbors by scanning the list. Since the list represents only a fraction of all the vectors in the table, using an IVFFlat index is considerably faster than scanning the whole table.

The number of lists in an IVFFlat index is configurable when creating the index. The more lists are created, the faster a search is, but the nearest neighbor approximation becomes less precise. The best number of lists for a dataset is typically its square root, e.g. if a dataset has 5,000,000 vectors, the number of lists should be:

```
postgresml=# SELECT round(sqrt(5000000)) AS lists;
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

71 is the approximate square root of 5,000 rows we have in that table. Now that we have an index, if we `EXPLAIN` the query we ran above, we'll get an "Index Scan" on the ANN index:

```
postgresml=# EXPLAIN SELECT
    "Address"
FROM usa_house_prices
ORDER BY 
    embedding <=> pgml.embed('intfloat/e5-small', '1 Infinite Loop')::vector(384)
LIMIT 3;

Limit  (cost=38.03..38.32 rows=3 width=55)
   ->  Index Scan using usa_house_prices_embedding_idx on usa_house_prices  (cost=38.03..327.23 rows=5001 width=55)
         Order By: (embedding <=> '[-0.033770584,-0.033374995, ...])
```

#### Maintaining an IVFFlat index

IVFFlat is a simple algorithm and constructing an index is quick. Splitting, sorting and solving KNN is optimized using the Postgres query engine and vectorized CPU operations (e.g. AVX512 on modern CPUs) built into `pgvector`. When queried, the index provides good recall acceleration and approximation for typical use cases.

On the other hand, because of the nature of centroids, if the dataset changes significantly, the original KNN calculation becomes inaccurate. In that case, an IVFFlat index should be rebuilt, which Postgres makes pretty easy:

```sql
REINDEX INDEX CONCURRENTLY usa_house_prices_embedding_idx;
```

The application user should monitor recall from an vector search operations and if the recall starts dropping over time, issue a reindex.

### HNSW

Home Navigable Small Worlds, or HNSW, is a modern ANN algorithm that constructs a multilayer graph using a greedy search algorithm with local minimums. Since HNSW is a graph and requires multiple passes over the same data, the time and memory cost of constructing HNSW are significantly higher, but it produces significantly better results than IVFFlat.

#### Creating an HNSW index

You can create an HNSW index with just one query:

```sql
CREATE INDEX ON
    usa_house_prices
USING hnsw(embedding vector_cosine_ops);
```

#### Maintaining an HNSW index

HNSW requires much less maintenance compared to IVFFlat. When new vectors are added, they are automatically inserted at the optimal place in the graph. However, as graph gets bigger, rebalancing the graph becomes more expensive; as the dataset grows, inserting new rows will become slower.

We address this trade-off and how to solve this problem in Partitioning.
