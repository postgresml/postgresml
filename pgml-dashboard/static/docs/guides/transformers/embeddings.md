# Embeddings
Embeddings are a numeric representation of text. They are used to represent words and sentences as vectors, an array of numbers. Embeddings can be used to find similar pieces of text, by comparing the similarity of the numeric vectors using a distance measure, or they can be used as input features for other machine learning models, since most algorithms can't use text directly.

Many pretrained LLMs can be used to generate embeddings from text within PostgresML. You can browse all the [models](https://huggingface.co/models?library=sentence-transformers) available to find the best solution on Hugging Face.

PostgresML provides a simple interface to generate embeddings from text in your database. You can use the `pgml.embed` function to generate embeddings for a column of text. The function takes a transformer name and a text value. The transformer will automatically be downloaded and cached for reuse.

## Long Form Examples
For a deeper dive, check out the following articles we've written illustrating the use of embeddings:

- [Generating LLM embeddings in the database with open source models](/blog/generating-llm-embeddings-with-open-source-models-in-postgresml)
- [Tuning vector recall while generating query embeddings on the fly](/blog/tuning-vector-recall-while-generating-query-embeddings-in-the-database)

## API

```sql linenums="1" title="embed.sql"
pgml.embed(
    transformer TEXT, -- huggingface sentence-transformer name
    text TEXT,        -- input to embed
    kwargs JSON       -- optional arguments (see below)
)
```

## Example

Let's use the `pgml.embed` function to generate embeddings for tweets, so we can find similar ones. We will use the `distilbert-base-uncased` model. This model is a small version of the `bert-base-uncased` model. It is a good choice for short texts like tweets.
To start, we'll load a dataset that provides tweets classified into different topics.
```postgresql linenums="1"
SELECT pgml.load_dataset('tweet_eval', 'sentiment');
```

View some tweets and their topics.
```postgresql linenums="1"
SELECT *
FROM pgml.tweet_eval
LIMIT 10;
```

Get a preview of the embeddings for the first 10 tweets. This will also download the model and cache it for reuse, since it's the first time we've used it.
```postgresql linenums="1"
SELECT text, pgml.embed('distilbert-base-uncased', text)
FROM pgml.tweet_eval
LIMIT 10;
```


It will take a few minutes to generate the embeddings for the entire dataset. We'll save the results to a new table.
```postgresql linenums="1"
CREATE TABLE tweet_embeddings AS
SELECT text, pgml.embed('distilbert-base-uncased', text) AS embedding
FROM pgml.tweet_eval;
```

Now we can use the embeddings to find similar tweets. We'll use the `pgml.cosign_similarity` function to find the tweets that are most similar to a given tweet (or any other text input).

```postgresql linenums="1"
WITH query AS (
    SELECT pgml.embed('distilbert-base-uncased', 'Star Wars christmas special is on Disney') AS embedding
)
SELECT text, pgml.cosine_similarity(tweet_embeddings.embedding, query.embedding) AS similarity
FROM tweet_embeddings, query
ORDER BY similarity DESC
LIMIT 50;
```

On small datasets (<100k rows), a linear search that compares every row to the query will give sub-second results, which may be fast enough for your use case. For larger datasets, you may want to consider various indexing strategies offered by additional extensions.

- [Cube](https://www.postgresql.org/docs/current/cube.html) is a built-in extension that provides a fast indexing strategy for finding similar vectors. By default it has an arbitrary limit of 100 dimensions, unless Postgres is compiled with a larger size.
- [PgVector](https://github.com/pgvector/pgvector) supports embeddings up to 2000 dimensions out of the box, and provides a fast indexing strategy for finding similar vectors.

```
CREATE EXTENSION vector;
CREATE TABLE items (text TEXT, embedding VECTOR(768));
INSERT INTO items SELECT text, embedding FROM tweet_embeddings;
CREATE INDEX ON items USING ivfflat (embedding vector_cosine_ops);
WITH query AS (
    SELECT pgml.embed('distilbert-base-uncased', 'Star Wars christmas special is on Disney')::vector AS embedding
)
SELECT * FROM items, query ORDER BY items.embedding <=> query.embedding LIMIT 10;
```
