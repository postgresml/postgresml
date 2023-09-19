---
description: Database that stores and manages vectors
---

# Vector Database

A vector database is a type of database that stores and manages vectors, which are mathematical representations of data points in a multi-dimensional space. Vectors can be used to represent a wide range of data types, including images, text, audio, and numerical data. It is designed to support efficient searching and retrieval of vectors, using methods such as nearest neighbor search, clustering, and indexing. These methods enable applications to find vectors that are similar to a given query vector, which is useful for tasks such as image search, recommendation systems, and natural language processing.

Using a vector database involves three key steps:&#x20;

1. Creating embeddings
2. Indexing your embeddings using different algorithms
3. Querying the index using embeddings for your queries.&#x20;

Let's break down each step in more detail.

### Step 1: Creating embeddings using transformers

To create embeddings for your data, you first need to choose a transformer that can generate embeddings from your input data. Some popular transformer options include BERT, GPT-2, and T5. Once you've selected a transformer, you can use it to generate embeddings for your data.

In the following section, we will demonstrate how to use PostgresML to generate embeddings for a dataset of tweets commonly used in sentiment analysis. To generate the embeddings, we will use the `pgml.embed` function, which was discussed in [embeddings.md](machine-learning/natural-language-processing/embeddings.md "mention"). These embeddings will then be inserted into a table called tweet\_embeddings.

```sql
SELECT pgml.load_dataset('tweet_eval', 'sentiment');

SELECT * 
FROM pgml.tweet_eval
LIMIT 10;

CREATE TABLE tweet_embeddings AS
SELECT text, pgml.embed('distilbert-base-uncased', text) AS embedding 
FROM pgml.tweet_eval;

SELECT * from tweet_embeddings limit 2;
```

_Result_

| text                                                                                                                    | embedding                                     |
| ----------------------------------------------------------------------------------------------------------------------- | --------------------------------------------- |
| "QT @user In the original draft of the 7th book, Remus Lupin survived the Battle of Hogwarts. #HappyBirthdayRemusLupin" | {-0.1567948312,-0.3149209619,0.2163394839,..} |
| "Ben Smith / Smith (concussion) remains out of the lineup Thursday, Curtis #NHL #SJ"                                    | {-0.0701668188,-0.012231146,0.1304316372,.. } |

### Step 2: Indexing your embeddings using different algorithms

After you've created embeddings for your data, you need to index them using one or more indexing algorithms. There are several different types of indexing algorithms available, including B-trees, k-nearest neighbors (KNN), and approximate nearest neighbors (ANN). The specific type of indexing algorithm you choose will depend on your use case and performance requirements. For example, B-trees are a good choice for range queries, while KNN and ANN algorithms are more efficient for similarity searches.

On small datasets (<100k rows), a linear search that compares every row to the query will give sub-second results, which may be fast enough for your use case. For larger datasets, you may want to consider various indexing strategies offered by additional extensions.

* [Cube](https://www.postgresql.org/docs/current/cube.html) is a built-in extension that provides a fast indexing strategy for finding similar vectors. By default it has an arbitrary limit of 100 dimensions, unless Postgres is compiled with a larger size.
* [PgVector](https://github.com/pgvector/pgvector) supports embeddings up to 2000 dimensions out of the box, and provides a fast indexing strategy for finding similar vectors.

When indexing your embeddings, it's important to consider the trade-offs between accuracy and speed. Exact indexing algorithms like B-trees can provide precise results, but may not be as fast as approximate indexing algorithms like KNN and ANN. Similarly, some indexing algorithms may require more memory or disk space than others.

In the following, we are creating an index on the tweet\_embeddings table using the ivfflat algorithm for indexing. The ivfflat algorithm is a type of hybrid index that combines an Inverted File (IVF) index with a Flat (FLAT) index.

The index is being created on the embedding column in the tweet\_embeddings table, which contains vector embeddings generated from the original tweet dataset. The `vector_cosine_ops` argument specifies the indexing operation to use for the embeddings. In this case, it's using the `cosine similarity` operation, which is a common method for measuring similarity between vectors.

By creating an index on the embedding column, the database can quickly search for and retrieve records that are similar to a given query vector. This can be useful for a variety of machine learning applications, such as similarity search or recommendation systems.

```
CREATE INDEX ON tweet_embeddings USING ivfflat (embedding vector_cosine_ops);
```

### Step 3: Querying the index using embeddings for your queries

Once your embeddings have been indexed, you can use them to perform queries against your database. To do this, you'll need to provide a query embedding that represents the query you want to perform. The index will then return the closest matching embeddings from your database, based on the similarity between the query embedding and the stored embeddings.

```
WITH query AS (
    SELECT pgml.embed('distilbert-base-uncased', 'Star Wars christmas special is on Disney')::vector AS embedding
)
SELECT * FROM items, query ORDER BY items.embedding <-> query.embedding LIMIT 5;
```

_Result_

| text                                                                                           |
| ---------------------------------------------------------------------------------------------- |
| Happy Friday with Batman animated Series 90S forever!                                          |
| "Fri Oct 17, Sonic Highways is on HBO tonight, Also new episode of Girl Meets World on Disney" |
| tfw the 2nd The Hunger Games movie is on Amazon Prime but not the 1st one I didn't watch       |
| 5 RT's if you want the next episode of twilight princess tomorrow                              |
| Jurassic Park is BACK! New Trailer for the 4th Movie, Jurassic World -                         |
