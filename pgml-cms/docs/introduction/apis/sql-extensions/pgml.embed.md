# pgml.embed()

Embeddings are a numeric representation of text. They are used to represent words and sentences as vectors, an array of numbers. Embeddings can be used to find similar pieces of text, by comparing the similarity of the numeric vectors using a distance measure, or they can be used as input features for other machine learning models, since most algorithms can't use text directly.

Many pretrained LLMs can be used to generate embeddings from text within PostgresML. You can browse all the [models](https://huggingface.co/models?library=sentence-transformers) available to find the best solution on Hugging Face.

## API

```sql
pgml.embed(
    transformer TEXT, -- huggingface sentence-transformer name
    text TEXT,        -- input to embed
    kwargs JSON       -- optional arguments (see below)
)
```

## Example

Let's use the `pgml.embed` function to generate embeddings for tweets, so we can find similar ones. We will use the `distilbert-base-uncased` model from :hugging: HuggingFace. This model is a small version of the `bert-base-uncased` model. It is a good choice for short texts like tweets. To start, we'll load a dataset that provides tweets classified into different topics.

```sql
SELECT pgml.load_dataset('tweet_eval', 'sentiment');
```

View some tweets and their topics.

```sql
SELECT *
FROM pgml.tweet_eval
LIMIT 10;
```

Get a preview of the embeddings for the first 10 tweets. This will also download the model and cache it for reuse, since it's the first time we've used it.

```sql
SELECT text, pgml.embed('distilbert-base-uncased', text)
FROM pgml.tweet_eval
LIMIT 10;
```

It will take a few minutes to generate the embeddings for the entire dataset. We'll save the results to a new table.

```sql
CREATE TABLE tweet_embeddings AS
SELECT text, pgml.embed('distilbert-base-uncased', text) AS embedding
FROM pgml.tweet_eval;
```
