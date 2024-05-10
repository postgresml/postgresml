---
description: Embeddings are a key building block with many applications in modern AI/ML systems. They are particularly valuable for handling various types of unstructured data like text, images, and more, providing a pathway to richer insights and improved performance. A common use case for embeddings is to provide semantic search capabilities that go beyond traditional keyword matching to the underlying meaning in the data.
---

# Embeddings

As the demand for sophisticated data analysis and machine learning capabilities within databases grows, so does the need for efficient and scalable solutions. PostgresML offers a powerful platform for integrating machine learning directly into PostgreSQL, enabling you to perform complex computations and predictive analytics without ever leaving your database. 

Embeddings are a key building block with many applications in modern AI/ML systems. They are particularly valuable for handling various types of unstructured data like text, images, and more, providing a pathway to richer insights and improved performance. A common use case for embeddings is to provide semantic search capabilities that go beyond traditional keyword matching to the underlying meaning in the data. 

This guide will introduce you to the fundamentals of embeddings within PostgresML. Whether you are looking to enhance text processing capabilities, improve image recognition functions, or simply incorporate more advanced machine learning models into your database, embeddings can play a pivotal role. By integrating these capabilities directly within PostgreSQL, you benefit from streamlined operations, reduced data movement, and the ability to leverage the full power of SQL alongside advanced machine learning techniques.

Throughout this guide, we will cover:

* [In-database Generation]()
* [Dimensionality Reduction]()
* [Re-ranking nearest neighbors]()
* [Indexing w/ IVFFLat vs HNSW]()
* [Aggregation]()
* [Personalization]()

# Embeddings are Vectors

Embeddings are represented mathematically as a vector, and can be stored in the native Postgres [`ARRAY[]`](https://www.postgresql.org/docs/current/arrays.html) datatype which is compatible with many application programming languages' native datatype. Modern CPUs and GPUs offer hardware acceleration for common array operations, which can give substantial performance benefits when operating at scale, but which are typically unused in a Postgres database. This is referred to as "vectorization" to enable these instruction sets. You'll need to ensure you're compiling your full stack with support for your hardware to get the most bang for your buck, or you can get full acceleration in a PostgresML cloud database. 

!!! warning

Other cloud providers claim to offer embeddings "inside the database", but if you benchmark their calls you'll see that their implementations are 10-100x slower than PostgresML. The reason is they aren't actually running inside the database. They are thin wrapper functions that making network calls to other datacenters to compute the embeddings. PostgresML is the only cloud that puts GPU hardware in the database for full acceleration, and it shows.

!!!
