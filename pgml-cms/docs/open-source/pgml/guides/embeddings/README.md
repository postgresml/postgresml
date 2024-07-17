---
description: Embeddings are a key building block with many applications in modern AI/ML systems. They are particularly valuable for handling various types of unstructured data like text, images, and more, providing a pathway to richer insights and improved performance. A common use case for embeddings is to provide semantic search capabilities that go beyond traditional keyword matching to the underlying meaning in the data.
---

# Embeddings

As the demand for sophisticated data analysis and machine learning capabilities within databases grows, so does the need for efficient and scalable solutions. PostgresML offers a powerful platform for integrating machine learning directly into PostgreSQL, enabling you to perform complex computations and predictive analytics without ever leaving your database.

Embeddings are a key building block with many applications in modern AI/ML systems. They are particularly valuable for handling various types of unstructured data like text, images, and more, providing a pathway to richer insights and improved performance. They allow computers to operate on natural language and other high level concepts by reducing them to billions of simple arithmetic operations.

## Applications of embeddings

- **Search and Information Retrieval**: Embeddings can transform search queries and documents into vectors, making it easier to find the most relevant documents for a given query based on semantic similarity.
- **Personalization**: In recommendation systems, embeddings can help understand user queries and preferences, enhancing the accuracy of recommendations.
- **Text Generation**: Large language models use embeddings to generate coherent and contextually relevant text, which can be applied in scenarios ranging from chatbots to content creation.
- **Natural Language Understanding (NLU)**: Embeddings enable models to perform tasks such as sentiment analysis, named entity recognition, and summarization by understanding the context and meaning of texts.
- **Translation**: In machine translation, embeddings help models understand the semantic and syntactic structures of different languages, facilitating the translation process.

This guide will introduce you to the fundamentals of embeddings within PostgresML. Whether you are looking to enhance text processing capabilities, improve image recognition functions, or simply incorporate more advanced machine learning models into your database, embeddings can play a pivotal role. By integrating these capabilities directly within PostgreSQL, you benefit from streamlined operations, reduced data movement, and the ability to leverage the full power of SQL alongside advanced machine learning techniques.

In this guide, we will cover:

* [In-database Generation](guides/embeddings/in-database-generation.md)
* [Dimensionality Reduction](guides/embeddings/dimensionality-reduction.md)
* [Aggregation](guides/embeddings/vector-aggregation.md)
* [Similarity](guides/embeddings/vector-similarity.md)
* [Normalization](guides/embeddings/vector-normalization.md)
<!--
* [Indexing w/ pgvector](guides/embeddings/indexing-w-pgvector.md)
* [Re-ranking nearest neighbors](guides/embeddings/re-ranking-nearest-neighbors.md)
* [Proprietary Models](guides/embeddings/proprietary-models.md)
-->

## Embeddings are vectors

In the context of large language models (LLMs), embeddings are representations of words, phrases, or even entire sentences. Each word or text snippet is mapped to a vector in a high-dimensional space. These vectors capture semantic and syntactic nuances, meaning that similar words have vectors that are close together in this space. For instance, "king" and "queen" would be represented by vectors that are closer together than "king" and "apple".

Vectors can be stored in the native Postgres [`ARRAY[]`](https://www.postgresql.org/docs/current/arrays.html) datatype which is compatible with many application programming languages' native datatypes. Modern CPUs and GPUs offer hardware acceleration for common array operations, which can give substantial performance benefits when operating at scale, but which are typically not enabled in a Postgres database. You'll need to ensure you're compiling your full stack with support for your hardware to get the most bang for your buck, or you can leave that up to us, and get full hardware acceleration in a PostgresML cloud database. 

!!! warning

Other cloud providers claim to offer embeddings "inside the database", but [benchmarks](/blog/mindsdb-vs-postgresml.md) show that they are orders of magnitude slower than PostgresML. The reason is they don't actually run inside the database with hardware acceleration. They are thin wrapper functions that make network calls to remote service providers. PostgresML is the only cloud that puts GPU hardware in the database for full acceleration, and it shows.

!!!

## Vectors support arithmetic

Vectors can be operated on mathematically with simple equations. For example, vector addition is defined as the sum of all the pairs of elements in the two vectors. This might be useful to combine two concepts into a single new embedding. For example "frozen" + "rain" should be similar to (≈) "snow" if the embedding model has encoded the nuances of natural language and precipitation. 

Most vector operations are simple enough to implement in a few lines of code. Here's a naive implementation (no hardware acceleration) of vector addition in some popular languages:

{% tabs %}
{% tab title="JavaScript" %}

```javascript
function add_vectors(x, y) {
    let result = [];
    for (let i = 0; i < x.length; i++) {
        result[i] = x[i] + y[i];
    }
    return result;
}

let x = [1, 2, 3];
let y = [1, 2, 3];
add(x, y)
```

{% endtab %}

{% tab title="Python" %}

```python
def add_vectors(x, y):
    return [x+y for x,y in zip(x,y)]    

x = [1, 2, 3]
y = [1, 2, 3]
add(x, y)
```

{% endtab %}
{% endtabs %}


If we pass the vectors for "snow" and "rain" into this function, we'd hope to get a vector similar to "snow" as the result, depending on the quality of the model that was used to create the word embeddings.
