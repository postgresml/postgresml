# Vector Distances
There are many distance functions that can be used to measure the similarity or differences between vectors. We list a few of the more common ones here with details on how they work, to help you choose. They are listed here in order of computational complexity, although modern hardware accelerated implementations can typically compare on the order of 100,000 vectors per second per processor. Modern CPUs may have tens to hundreds of cores, and GPUs have tens of thousands. 

## Manhattan Distance

You can think of this distance metric as how long it takes you to walk from one building in Manhattan to another, when you can only walk along streets that go the 4 cardinal directions, with no diagonals. It's the fastest distance measure to implement, because it just adds up all the pairwise element differences. It's also referred to as the L1 distance.

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

## Euclidean Distance

This is a simple refinement of Manhattan Distance that applies the Pythagorean theorem to find the length of the straight line between the two points. It involves squaring the differences and then taking the final square root, which is a more expensive operation, so it may be slightly slower, but is also a more accurate representation in high dimensional spaces. It's also referred to as the L2 distance.

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

## Inner product

The inner product (the dot product in Euclidean space) can be used to find how similar any two vectors are, by measuring the overlap of each element, which compares the direction they point. Two completely different (orthogonal) vectors have an inner product of 0. If vectors point in opposite directions, the inner product will be negative. Positive numbers indicate the vectors point in the same direction, and are more similar.

This metric is as fast to compute as the Euclidean Distance, but may provide more relevant results if all vectors are normalized. If vectors are not normalized, it will bias results toward vectors with larger magnitudes, and you should consider using the cosine distance instead. 

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


## Cosine Distance

Cosine distance is a popular metric, because it normalizes the vectors, which means it only considers the difference of the angle between the two vectors, not their magnitudes. If you don't know that your vectors have been normalized, this may be a safer bet than the inner product. It is one of the more complicated algorithms to implement, but differences may be negligible w/ modern hardware accelerated instruction sets depending on your workload profile. 

You can also use PostgresML to [normalize all your vectors](vector_normalization.md) as a separate processing step to pay that cost only at indexing time, and then the inner product will provide equivalent distance measures.

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
