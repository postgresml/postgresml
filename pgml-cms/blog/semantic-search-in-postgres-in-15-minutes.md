
---
description: >-
  Dive into the world of Postgres and learn how to implement semantic search in nothing but SQL.
featured: true
image: ".gitbook/assets/image (2) (2).png"
tags: ["Engineering"]
---

# Semantic Search in Postgres in 15 Minutes

<div align="left">

<figure><img src=".gitbook/assets/montana.jpg" alt="Author" width="125"><figcaption></figcaption></figure>

</div>

Silas Marvin

June 15, 2024

PostgresML gives the ability to generate, store, and search over embeddings directly in your Postgres database.

<figure><img src=".gitbook/assets/image (2) (2).png" alt=""><figcaption><p>PostgresML is a composition engine that provides advanced AI capabilities.</p></figcaption></figure>

## What is and is not Semantic Search

Semantic search is a new form of machine learning powered search that doesn’t rely on any form of keyword matching, but transforms text into embeddings and performs nearest neighbors search. 

It is not a complete replacement for full text search. In many cases full text search is capable of outperforming semantic search. Specifically, if a user knows the exact phrase in a document they want to match, full text search is faster and guaranteed to return the correct result while semantic search is only likely to return the correct result. Full text search and semantic search can be combined to create powerful and robust search systems.

Semantic search is not just for machine learning engineers. The actual system behind semantic search is relatively easy to implement and thanks to new Postgres extensions like pgml and pgvector, is readily available to SQL developers. Just as it is expected for modern SQL developers to be familiar with and capable of implementing full text search, soon SQL developers will be expected to implement semantic search.

## Embeddings 101

Semantic search is powered by embeddings. To understand how semantic search works, we must have a basic understanding of embeddings. 

Embeddings are vectors. Given some text and some embedding model, we can convert text to vectors:

!!! generic

!!! code block time="10.493 ms"

```postgresql
SELECT pgml.embed('mixedbread-ai/mxbai-embed-large-v1', 'test');
```

!!!

!!!

Here we are using the pgml.embed SQL function to generate an embedding using the `mixedbread-ai/mxbai-embed-large-v1` model. 

The output size of the vector varies per model. This specific model outputs vectors with 1024 dimensions. This means each vector contains 1024 floating point numbers. 

The vector this model outputs is not random. It is designed to capture the semantic meaning of the text. What this really means, is that sentences that are closer together in meaning will be closer together in vector space. 

Let’s look at a more simple example. Assume we have a model called `simple-embedding-model`, and it outputs vectors with 2 dimensions. Let’s embed the following three phrases: `I like Postgres`, `I like SQL`, `Rust is the best`. 

!!! generic

!!! code block time="10.493 ms"

```postgresql
SELECT pgml.embed('simple-embedding-model', 'I like Postgres') as embedding;

SELECT pgml.embed('simple-embedding-model', 'I like SQL') as embedding;

SELECT pgml.embed('simple-embedding-model', 'Rust is the best') as embedding;
```

!!!

!!! results

embedding
---------
[0.1, 0.2]

embedding
---------
[0.12, 0.25]

embedding
---------
[-0.8, -0.9]

!!!

!!!

Notice how similar the vectors produced by the text `I like Postgres` and `I like SQL` are compared to `Rust is the best`. 

This is a simple example, but the same idea holds true when translating to real models like `mixedbread-ai/mxbai-embed-large-v1`. 

## What Does it Mean to be Close?

We can use the idea that text that is more similar in meaning will be closer together in the vector space to perform search. 

For instance let’s say that we have the following documents:

```text
Document1: The pgml.transform function is a PostgreSQL function for calling LLMs in the database.

Document2: I think tomatos are incredible on burgers.
```

And a user is looking for the answer to the question: `What is the pgml.transform function?`. If we embed the user query and all of the documents using a model like `mixedbread-ai/mxbai-embed-large-v1`, we can compare the query embedding to all of the document embeddings, and select the document that has the closest embedding as the answer. 

These are big embeddings, and we can’t simply eyeball which one is the closest. How do we actually measure the similarity / distance between different vectors? There are four popular methods for measuring the distance between vectors available in PostgresML:
- L2 distance
- (negative) inner product
- cosine distance
- L1 distance

For most use cases we recommend using the cosine distance as defined by the formula:

INSERT IMAGE

Where A and B are two vectors. 

This is a somewhat confusing formula but lucky for us pgvector provides an operator that computes this for us.

!!! generic

!!! code block

```postgresql
SELECT '[1,2,3]'::vector <=> '[2,3,4]'::vector;
```

!!!

!!! results

   cosine_distance    
----------------------
 0.007416666029069763
(1 row)

!!!

!!!

The other distance functions have similar formulas and also provide convenient operators to use. It may be worth testing the other operators and seeing which performs better for your use case. For more information on the other distance functions see our guide on [embeddings](https://postgresml.org/docs/guides/embeddings/vector-similarity).

Back to our search example outlined above, we can compute the cosine distance between our query embedding and our documents.

!!! generic

!!! code block

```postgresql
SELECT pgml.embed('mixedbread-ai/mxbai-embed-large-v1', 'What is the pgml.transform function?')::vector <=> pgml.embed('mixedbread-ai/mxbai-embed-large-v1', 'The pgml.transform function is a PostgreSQL function for calling LLMs in the database.')::vector as cosine_distance;
SELECT pgml.embed('mixedbread-ai/mxbai-embed-large-v1', 'What is the pgml.transform function?')::vector <=> pgml.embed('mixedbread-ai/mxbai-embed-large-v1', 'I think tomatos are incredible on burgers.')::vector as cosine_distance;
```

!!!

!!! results

  cosine_distance   
--------------------
 0.1114425936213167

  cosine_distance   
--------------------
 0.7383001059221699

!!!

!!!

Notice that the cosine distance between `What is the pgml.transform function?` and `The pgml.transform function is a PostgreSQL function for calling LLMs in the database.` is much smaller than the cosine distance between `What is the pgml.transform function?` and `I think tomatos are incredible on burgers.`.

## Making it Fast!

It is inefficient to compute the embeddings for our documents for every search request. Instead, we want to embed our documents once, and search against our stored embeddings. 

We can store our embedding vectors with the vector type given by pgvector. 

!!! generic

!!! code block

```postgresql
CREATE TABLE text_and_embeddings (
    id SERIAL PRIMARY KEY, 
    text text, 
    embedding vector (1024)
);
INSERT INTO text_and_embeddings(text, embedding) 
VALUES 
  ('The pgml.transform function is a PostgreSQL function for calling LLMs in the database.', pgml.embed('mixedbread-ai/mxbai-embed-large-v1', 'The pgml.transform function is a PostgreSQL function for calling LLMs in the database.')),
  ('I think tomatos are incredible on burgers.', pgml.embed('mixedbread-ai/mxbai-embed-large-v1', 'I think tomatos are incredible on burgers.'))
;
```

!!!

!!!

We can search this table using the following query: 

!!! generic

!!! code block time="10.493 ms"

```postgresql
WITH embedded_query AS (
    SELECT
        pgml.embed('mixedbread-ai/mxbai-embed-large-v1', 'What is the pgml.transform function?', '{"prompt": "Represent this sentence for searching relevant passages: "}')::vector embedding
)
SELECT
    text,
    (
        SELECT
            embedding
        FROM embedded_query) <=> text_and_embeddings.embedding cosine_distance
FROM
  text_and_embeddings
ORDER BY
    text_and_embeddings.embedding <=> (
        SELECT
            embedding
        FROM embedded_query)
LIMIT 1;
```

!!!

!!! results

                                          text                                          |   cosine_distance   
----------------------------------------------------------------------------------------+---------------------
 The pgml.transform function is a PostgreSQL function for calling LLMs in the database. | 0.13467974993681486
(1 row)

!!!

!!!

This query is fast for now, but as the table scales it will greatly slow down because we have not indexed the vector column. 


!!! generic

!!! code block time="10.493 ms"

```postgresql
INSERT INTO text_and_embeddings (text, embedding) 
SELECT md5(random()::text), pgml.embed('mixedbread-ai/mxbai-embed-large-v1', md5(random()::text)) 
FROM generate_series(1, 10000);

WITH embedded_query AS (
    SELECT
        pgml.embed('mixedbread-ai/mxbai-embed-large-v1', 'What is the pgml.transform function?', '{"prompt": "Represent this sentence for searching relevant passages: "}')::vector embedding
)
SELECT
    text,
    (
        SELECT
            embedding
        FROM embedded_query) <=> text_and_embeddings.embedding cosine_distance
FROM
  text_and_embeddings
ORDER BY
    text_and_embeddings.embedding <=> (
        SELECT
            embedding
        FROM embedded_query)
LIMIT 1;
```

!!!

!!! results

                                          text                                          |   cosine_distance   
----------------------------------------------------------------------------------------+---------------------
 The pgml.transform function is a PostgreSQL function for calling LLMs in the database. | 0.13467974993681486
(1 row)

!!!

!!!

This somewhat less than ideal performance can be fixed by indexing the vector column. There are two types of indexes available in pgvector: IVFFlat and HNSW.

IVFFlat indexes clusters the table into sublists, and when searching, only searches over a fixed number of the sublists. For example in the case above, if we were to add an IVFFlat index with 10 lists:

!!! generic

!!! code block time="10.493 ms"

```postgresql
CREATE INDEX ON text_and_embeddings USING ivfflat (embedding vector_cosine_ops) WITH (lists = 10);
```

!!!

!!!

Now let's try searching again.

!!! generic

!!! code block time="10.493 ms"

```postgresql
WITH embedded_query AS (
    SELECT
        pgml.embed('mixedbread-ai/mxbai-embed-large-v1', 'What is the pgml.transform function?', '{"prompt": "Represent this sentence for searching relevant passages: "}')::vector embedding
)
SELECT
    text,
    (
        SELECT
            embedding
        FROM embedded_query) <=> text_and_embeddings.embedding cosine_distance
FROM
  text_and_embeddings
ORDER BY
    text_and_embeddings.embedding <=> (
        SELECT
            embedding
        FROM embedded_query)
LIMIT 1;
```

!!!

!!! results

                                          text                                          |   cosine_distance   
----------------------------------------------------------------------------------------+---------------------
 The pgml.transform function is a PostgreSQL function for calling LLMs in the database. | 0.13467974993681486
(1 row)

!!!

!!!

We can see it is about a 10x speedup because we are only searching over 1/10th of the original vectors. 

HNSW indexes are a bit more complicated. It is essentially a graph with edges linked by proximity in the vector space. For more information you can check out this [writeup](https://www.pinecone.io/learn/series/faiss/hnsw/). 

HNSW indexes typically have better and faster recall but require more compute when inserting. We recommend using HNSW indexes for most use cases.

!!! generic

!!! code block time="10.493 ms"

```postgresql
DROP index text_and_embeddings_embedding_idx;

CREATE INDEX ON text_and_embeddings USING hnsw (embedding vector_cosine_ops);
```

!!!

!!!

Now let's try searching again.

!!! generic

!!! code block time="10.493 ms"

```postgresql
WITH embedded_query AS (
    SELECT
        pgml.embed('mixedbread-ai/mxbai-embed-large-v1', 'What is the pgml.transform function?', '{"prompt": "Represent this sentence for searching relevant passages: "}')::vector embedding
)
SELECT
    text,
    (
        SELECT
            embedding
        FROM embedded_query) <=> text_and_embeddings.embedding cosine_distance
FROM
  text_and_embeddings
ORDER BY
    text_and_embeddings.embedding <=> (
        SELECT
            embedding
        FROM embedded_query)
LIMIT 1;
```

!!!

!!! results

                                          text                                          |   cosine_distance   
----------------------------------------------------------------------------------------+---------------------
 The pgml.transform function is a PostgreSQL function for calling LLMs in the database. | 0.13467974993681486
(1 row)

!!!

!!!

That was even faster!
