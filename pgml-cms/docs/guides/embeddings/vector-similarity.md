# Vector Similarity

Similar embeddings should represent similar concepts. If we have one embedding created from a user query and a bunch of other embeddings from documents, we can find documents that are most similar to the query by calculating the similarity between the query and each document. Embedding similarity (≈) is defined as the distance between the two vectors.

There are several ways to measure the distance between two vectors, that have tradeoffs in latency and accuracy. If two vectors are identical (=), then the distance between them is 0. If the distance is small, then they are similar (≈).  Here, we explore a few of the more common ones here with details on how they work, to help you choose. It's worth taking the time to understand the differences between these simple formulas, because they are the inner loop that accounts for almost all computation when doing nearest neighbor search. 

They are listed here in order of computational complexity, although modern hardware accelerated implementations can typically compare on the order of 100,000 vectors per second per processor, depending on how many dimensions the vectors have. Modern CPUs may also have tens to hundreds of cores, and GPUs have tens of thousands, to further parallelize searches across large numbers of vectors. 

!!! note

If you just want the cliff notes: [Normalize your vectors](vector-normalization) and use the inner product as your distance metric between two vectors. This is implemented as: `pgml.dot_product(a, b)`

!!!

All of these distance measures are implemented by PostgresML for the native Postgres `ARRAY[]` types, and separately implemented by pgvector as operators for its `VECTOR` types using operators.

## Manhattan Distance

You can think of this distance metric as how long it takes you to walk from one building in Manhattan to another, when you can only walk along streets that go the 4 cardinal directions, with no diagonals. It's the fastest distance measure to implement, because it just adds up all the pairwise element differences. It's also referred to as the L1 distance.

!!! tip

Most applications should use Euclidean Distance instead, unless accuracy has relatively little value, and nanoseconds are important to your user experience.

!!!

**Algorithm**


{% tabs %}

{% tab title="JavaScript" %}

```javascript
function manhattanDistance(x, y) {
    let result = 0;
    for (let i = 0; i < x.length; i++) {
        result += x[i] - y[i];
    }
    return result;
}

let x = [1, 2, 3];
let y = [1, 2, 3];
manhattanDistance(x, y)
```

{% endtab %}

{% tab title="Python" %}

```python
def manhattan_distance(x, y):
    return sum([x-y for x,y in zip(x,y)])    

x = [1, 2, 3]
y = [1, 2, 3]
manhattan_distance(x, y)
```

{% endtab %}
{% endtabs %}

An optimized version is provided by:

!!! code_block time="1191.069 ms" 

```postgresql
WITH query AS (
    SELECT vector
    FROM test_data
    LIMIT 1
)
SELECT id, pgml.distance_l1(query.vector, test_data.vector)
FROM test_data, query
ORDER BY distance_l1;
```

!!!

The equivalent pgvector operator is `<+>`.


## Euclidean Distance

This is a simple refinement of Manhattan Distance that applies the Pythagorean theorem to find the length of the straight line between the two points. It's also referred to as the L2 distance. It involves squaring the differences and then taking the final square root, which is a more expensive operation, so it may be slightly slower, but is also a more accurate representation in high dimensional spaces. When finding nearest neighbors, the final square root can computation can be omitted, but there are still twice as many operations in the inner loop. 


!!! tip

Most applications should use Inner product for better accuracy with less computation, unless you can't afford to normalize your vectors before indexing for some extremely write heavy application.

!!!

**Algorithm**

{% tabs %}
{% tab title="JavaScript" %}

```javascript
function euclideanDistance(x, y) {
    let result = 0;
    for (let i = 0; i < x.length; i++) {
        result += Math.pow(x[i] - y[i], 2);
    }
    return Math.sqrt(result);
}

let x = [1, 2, 3];
let y = [1, 2, 3];
euclideanDistance(x, y)
```

{% endtab %}

{% tab title="Python" %}

```python
def euclidean_distance(x, y):
    return math.sqrt(sum([(x-y) * (x-y) for x,y in zip(x,y)]))    

x = [1, 2, 3]
y = [1, 2, 3]
euclidean_distance(x, y)
```

{% endtab %}
{% endtabs %}

An optimized version is provided by:

!!! code_block time="1359.114 ms"

```postgresql
WITH query AS (
    SELECT vector
    FROM test_data
    LIMIT 1
)
SELECT id, pgml.distance_l2(query.vector, test_data.vector)
FROM test_data, query
ORDER BY distance_l2;
```

!!!

The equivalent pgvector operator is `<->`.

## Inner product

The inner product (the dot product in Euclidean space) can be used to find how similar any two vectors are, by measuring the overlap of each element, which compares the direction they point. Two completely different (orthogonal) vectors have an inner product of 0. If vectors point in opposite directions, the inner product will be negative. Positive numbers indicate the vectors point in the same direction, and are more similar.

This metric is as fast to compute as the Euclidean Distance, but may provide more relevant results if all vectors are normalized. If vectors are not normalized, it will bias results toward vectors with larger magnitudes, and you should consider using the cosine distance instead. 

!!! tip

This is probably the best all around distance metric. It's computationally simple, but also twice as fast due to optimized assembly intructions. It's also able to places more weight on the dominating dimensions of the vectors which can improve relevance during recall. As long as [your vectors are normalized](vector-normalization).

!!!

**Algorithm**

{% tabs %}
{% tab title="JavaScript" %}

```javascript
function innerProduct(x, y) {
    let result = 0;
    for (let i = 0; i < x.length; i++) {
        result += x[i] * y[i];
    }
    return result;
}

let x = [1, 2, 3];
let y = [1, 2, 3];
innerProduct(x, y)
```

{% endtab %}

{% tab title="Python" %}

```python
def inner_product(x, y):
    return sum([x*y for x,y in zip(x,y)])    

x = [1, 2, 3]
y = [1, 2, 3]
inner_product(x, y)
```

{% endtab %}
{% endtabs %}

An optimized version is provided by:

!!! code_block time="498.649 ms"

```postgresql
WITH query AS (
    SELECT vector
    FROM test_data
    LIMIT 1
)
SELECT id, pgml.dot_product(query.vector, test_data.vector)
FROM test_data, query
ORDER BY dot_product;
```

!!!

The equivalent pgvector operator is `<#>`.


## Cosine Distance

Cosine distance is a popular metric, because it normalizes the vectors, which means it only considers the difference of the angle between the two vectors, not their magnitudes. If you don't know that your vectors have been normalized, this may be a safer bet than the inner product. It is one of the more complicated algorithms to implement, but differences may be negligible w/ modern hardware accelerated instruction sets depending on your workload profile. 

!!! tip

Use PostgresML to [normalize all your vectors](vector-normalization) as a separate processing step to pay that cost only at indexing time, and then switch to the inner product which will provide equivalent distance measures, at 1/3 of the computation in the inner loop. _That's not exactly true on all platforms_, because the inner loop is implemented with optimized assembly that can take advantage of additional hardware acceleration, so make sure to always benchmark on your own hardware. On our hardware, the performance difference is negligible.

!!!

**Algorithm**

{% tabs %}
{% tab title="JavaScript" %}

```javascript
function cosineDistance(a, b) {
    let dotProduct = 0;
    let normA = 0;
    let normB = 0;

    for (let i = 0; i < a.length; i++) {
        dotProduct += a[i] * b[i];
        normA += a[i] * a[i];
        normB += b[i] * b[i];
    }

    normA = Math.sqrt(normA);
    normB = Math.sqrt(normB);

    if (normA === 0 || normB === 0) {
        throw new Error("Norm of one or both vectors is 0, cannot compute cosine similarity.");
    }

    const cosineSimilarity = dotProduct / (normA * normB);
    const cosineDistance = 1 - cosineSimilarity;

    return cosineDistance;
}
```
{% endtab %}

{% tab title="Python" %}

```python
def cosine_distance(a, b):
    dot_product = 0
    normA = 0
    normB = 0

    for a, b in zip(a, b):
        dot_product += a * b
        normA += a * a
        normB += b * b

    normA = math.sqrt(normA)
    normB = math.sqrt(normB)

    if normA == 0 or normB == 0:
        raise ValueError("Norm of one or both vectors is 0, cannot compute cosine similarity.")

    cosine_similarity = dot_product / (normA * normB)
    cosine_distance = 1 - cosine_similarity

    return cosine_distance
```

{% endtab %}
{% endtabs %}

The optimized version is provided by:

!!! code_block time="508.587 ms"

```postgresql
WITH query AS (
    SELECT vector
    FROM test_data
    LIMIT 1
)
SELECT id, 1 - pgml.cosine_similarity(query.vector, test_data.vector) AS cosine_distance
FROM test_data, query
ORDER BY cosine_distance;
```

!!!

Or you could reverse order by `cosine_similarity` for the same ranking:

!!! code_block time="502.461 ms"

```postgresql
WITH query AS (
    SELECT vector
    FROM test_data
    LIMIT 1
)
SELECT id, pgml.cosine_similarity(query.vector, test_data.vector)
FROM test_data, query
ORDER BY cosine_similarity DESC;
```

!!!

The equivalent pgvector operator is `<=>`.

## Benchmarking

You should benchmark and compare the computational cost of these distance metrics to see how much they algorithmic differences matters for latency using the same vector sizes as your own data. We'll create some test data to demonstrate the relative costs associated with each distance metric.

!!! code_block

```postgresql
\timing on
```

!!!

!!! code_block

```postgresql
CREATE TABLE test_data (
    id BIGSERIAL NOT NULL,
    vector FLOAT4[]
);
```

!!!

Insert 10k vectors, that have 1k dimensions each

!!! code_block

```postgresql
INSERT INTO test_data (vector) 
SELECT array_agg(random())
FROM generate_series(1,10000000) i
GROUP BY i % 10000;
```

!!!
