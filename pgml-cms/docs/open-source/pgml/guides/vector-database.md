---
description: Use PostgreSQL as your vector database to store, index and search vectors with the pgvector extension.
---

# Vector database

Vectors are lists of numbers representing a measurement in multidimensional space. There are many types of vectors, e.g. embeddings used for semantic search, but ultimately they are all just arrays of floating points.

In Postgres, a vector is just another data type that can be stored in regular tables and queried together with other columns. At PostgresML, we're using _pgvector_, a Postgres extension that implements the _vector_ data type, and many vector operations like inner product, cosine distance, and approximate nearest neighbor (ANN) search.

### Installing pgvector

If you're using our [cloud](https://postgresml.org/signup) or our Docker image, your database has _pgvector_ installed already. If you're self-hosting PostgresML, take a look at our [Self-hosting](/docs/open-source/pgml/developers/self-hosting/) documentation.

### Working with vectors

Vectors can be stored in columns, just like any other data type. To add a vector column to your table, you need to specify the size of the vector. All vectors in a single column must be the same size, since there are no useful operations between vectors of different sizes.

#### Adding a vector column

Using the example from [Tabular data](../../../introduction/import-your-data/storage-and-retrieval/README.md), let's add a vector column to our USA House Prices table:

{% tabs %}
{% tab title="SQL" %}

```postgresql
ALTER TABLE usa_house_prices
ADD COLUMN embedding VECTOR(384);
```

{% endtab %}

{% tab title="Output" %}

```
ALTER TABLE
```

{% endtab %}
{% endtabs %}

#### Generating embeddings

At first, the column is empty. To generate embeddings, we can use the PostgresML [pgml.embed()](/docs/open-source/pgml/api/pgml.embed) function and generate an embedding of another column in the same (or different) table. This is where machine learning inside the database really shines:

{% tabs %}
{% tab title="SQL" %}

```postgresql
UPDATE usa_house_prices
SET embedding = pgml.embed(
    'Alibaba-NLP/gte-base-en-v1.5',
    address
);
```

{% endtab %}
{% tab title="Output" %}

```
UPDATE 5000
```

{% endtab %}
{% endtabs %}

That's it. We just created 5,000 embeddings of the values stored in the address column, all with just one SQL query. Let's take a look at what we created:

{% tabs %}
{% tab title="SQL" %}

```postgresql
SELECT
    address,
    (embedding::real[])[1:5] 
FROM usa_house_prices
WHERE address = '1 Infinite Loop, Cupertino, California';

```

{% endtab %}
{% tab title="Output" %}

```
                           address                                 |                           embedding                            
----------------------------------------+----------------------------------------------------------------
 1 Infinite Loop, Cupertino, California | {-0.009034249,-0.055827666,-0.09911688,0.005093358,0.04053181}
(1 row)
```

{% endtab %}
{% endtabs %}

The vectors contain 384 values each, but that won't fit on our screen, so we selected the first 5 values using the Postgres array slice notation `[1:5]` (Postgres array indices start at one, not zero).

### Searching vectors

If your dataset is small enough, searching vectors doesn't require approximation. You can find the exact nearest neighbor match using any of the distance functions supported by _pgvector_: L2, cosine distance, inner product and cosine similarity.

Each distance function is implemented with its own operator and can be used as part of all SQL queries:

| Distance function | Operator        | Index operator      |
| ----------------- | --------------- | ------------------- |
| L2                | `<->`           | `vector_in_ops`     |
| Inner product     | `<#>`           | `vector_l2_ops`     |
| Cosine distance   | `<=>`           | `vector_cosine_ops` |
| Cosine similarity | `1 - (a <=> b)` | `vector_cosine_ops` |

For example, if we wanted to find three closest matching addresses to `1 Infinite Loop` using cosine distance:

{% tabs %}
{% tab title="SQL" %}

```postgresql
SELECT address
FROM usa_house_prices
ORDER BY 
    embedding <=> pgml.embed(
        'Alibaba-NLP/gte-base-en-v1.5', 
        '1 Infinite Loop'
    )::vector(384)
LIMIT 3;
```

{% endtab %}
{% tab title="Output" %}

```
                address                 
----------------------------------------
 1 Infinite Loop, Cupertino, California
 615 Larry Loop                        
 Warrenberg, PR 37943
(5 rows)
```

{% endtab %}
{% endtabs %}

This query uses [pgml.embed()](/docs/open-source/pgml/api/pgml.embed) to generate an embedding on the fly and finds the exact closest neighbors to that embedding in the entire dataset.

### Approximate nearest neighbors

This example dataset only has 5,000 rows which, for Postgres, is really easy to scan. In the real world, these datasets grow to become very large and searching the entire table becomes too slow to be practical. When that happens, we can get closest matches using approximation. Approximate nearest neighbors, or ANN, is a commonly used technique to organize vectors to find results that are "close enough".

_pgvector_ implements two ANN algorithms: IVFFlat and HNSW. Both have their pros and cons and can be used in production to search millions of vectors.

### IVFFlat

IVFFlat splits the list of vectors into roughly equal parts, grouped around centroids calculated using k-nearest neighbors (KNN). The lists are stored in a B-tree index, ordered by the centroid.

When searching for nearest neighbors, _pgvector_ picks the list with the closest centroid to the candidate vector, fetches all the vectors from that list, sorts them, and returns the closest neighbors. Since the list represents only a fraction of all vectors, using an IVFFlat index is considerably faster than scanning the entire table.

The number of lists in an IVFFlat index is configurable on index creation. The more lists, the faster you can search them, but the nearest neighbor approximation becomes less precise. The best number of lists for a dataset is typically its square root, e.g. if a dataset has 5,000,000 vectors, the number of lists should be:

{% tabs %}
{% tab title="SQL" %}

```postgresql
SELECT round(sqrt(5000000)) AS lists;
```

{% endtab %}
{% tab title="Output" %}

```
 lists 
-------
  2236
```

{% endtab %}
{% endtabs %}

#### Creating an IVFFlat index

You can create an IVFFlat index with just one query:

{% tabs %}
{% tab title="SQL" %}

```postgresql
CREATE INDEX ON usa_house_prices
USING ivfflat(embedding vector_cosine_ops)
WITH (lists = 71);
```

{% endtab %}
{% tab title="Output" %}

```
CREATE INDEX
```

{% endtab %}
{% endtabs %}

71 is the approximate square root of 5,000 rows we have in that table. With the index created, if we `EXPLAIN` the query we just ran, we'll get an index scan on the cosine distance index:

{% tabs %}
{% tab title="SQL" %}

```postgresql
EXPLAIN 
SELECT address
FROM usa_house_prices
ORDER BY 
    embedding <=> pgml.embed(
        'Alibaba-NLP/gte-base-en-v1.5',
        '1 Infinite Loop'
    )::vector(384)
LIMIT 3;
```

{% endtab %}
{% tab title="Output" %}

```
Limit  (cost=38.03..38.32 rows=3 width=55)
   ->  Index Scan using usa_house_prices_embedding_idx on usa_house_prices  (cost=38.03..327.23 rows=5001 width=55)
         Order By: (embedding <=> '[-0.033770584,-0.033374995, ...])
```

{% endtab %}
{% endtabs %}

It's important to create an IVFFlat index after you have added a representative sample of vectors into your table. Without a representative sample, the calculated centroids will be incorrect and the approximation of nearest neighbors inaccurate.

#### Maintaining an IVFFlat index

IVFFlat is a simple algorithm and constructs an index quickly. Splitting, sorting and solving KNN is optimized using the Postgres query engine and vectorized CPU operations (e.g. AVX512 on modern CPUs) built into _pgvector_. When queried, the index provides good performance and approximation for most use cases.

On the other hand, because of the nature of centroids, if the dataset changes in a statistically significant way, the original KNN calculation becomes inaccurate. In that case, an IVFFlat index should be rebuilt which Postgres makes pretty easy:

{% tabs %}
{% tab title="SQL" %}

```postgresql
REINDEX INDEX CONCURRENTLY usa_house_prices_embedding_idx;
```

{% endtab %}
{% tab title="Output" %}

```
REINDEX
```

{% endtab %}
{% endtabs %}

As of this writing, _pgvector_ doesn't provide monitoring tools for index degradation. The user should monitor recall from their vector search operations, and if it starts dropping, run a reindex.

### HNSW

Home Navigable Small Worlds, or HNSW, is a modern ANN algorithm that constructs a multilayer graph using a greedy search with local minimums. Constructing HNSW requires multiple passes over the same data, so the time and memory cost of building it are higher, but it does have faster and better recall than IVFFlat.

#### Creating an HNSW index

You can create an HNSW index with just one query:

{% tabs %}
{% tab title="SQL" %}

```postgresql
CREATE INDEX ON usa_house_prices
USING hnsw(embedding vector_cosine_ops);
```

{% endtab %}
{% tab title="Output" %}

```
CREATE INDEX
```

{% endtab %}
{% endtabs %}

#### Maintaining an HNSW index

HNSW requires little to no maintenance. When new vectors are added, they are automatically inserted at the optimal place in the graph. However, as the graph gets bigger, rebalancing it becomes more expensive, and inserting new rows becomes slower. We address this trade-off and how to solve this problem in [Partitioning](../../../introduction/import-your-data/storage-and-retrieval/partitioning.md).
