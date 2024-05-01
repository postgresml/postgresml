---
description: >-
  Generate high quality embeddings with faster end-to-end vector operations
  without an additional vector database.
---

# pgml.embed()

The `pgml.embed()` function generates [embeddings](/docs/use-cases/embeddings/) from text, using in-database models downloaded from Hugging Face. Thousands of [open-source models](https://huggingface.co/models?library=sentence-transformers) are available and new and better ones are being published regularly.

## API

```sql
pgml.embed(
    transformer TEXT,
    "text" TEXT,
    kwargs JSONB
)
```

| Argument | Description | Example |
|----------|-------------|---------|
| transformer | The name of a Hugging Face embedding model. | `intfloat/e5-large-v2` |
| text | The text to embed. This can be a string or the name of a column from a PostgreSQL table. | `'I am your father, Luke'` |
| kwargs | Additional arguments that are passed to the model during inference. | |

### Examples

#### Generate embeddings from text

Creating an embedding from text is as simple as calling the function with the text you want to embed:

{% tabs %}
{% tab title="SQL" %}

```postgresql
SELECT * FROM pgml.embed(
  'intfloat/e5-small',
  'No, that''s not true, that''s impossible.'
) AS star_wars_embedding;
```

{% endtab %}
{% endtabs %}

#### Generate embeddings inside a table

SQL functions can be used as part of a query to insert, update, or even automatically generate column values of any table:

{% tabs %}
{% tab title="SQL" %}

```postgresql
CREATE TABLE star_wars_quotes (
  quote TEXT NOT NULL,
  embedding vector(384) GENERATED ALWAYS AS (
    pgml.embed('intfloat/e5-small', quote)
  ) STORED
);

INSERT INTO
  star_wars_quotes (quote)
VALUES
('I find your lack of faith disturbing'),
('I''ve got a bad feeling about this.'),
('Do or do not, there is no try.');
```

{% endtab %}
{% endtabs %}

In this example, we're using [generated columns](https://www.postgresql.org/docs/current/ddl-generated-columns.html) to automatically create an embedding of the `quote` column every time the column value is updated.

#### Using embeddings in queries

Once you have embeddings, you can use them in queries to find text with similar semantic meaning:

{% tabs %}
{% tab title="SQL" %}

```postgresql
SELECT
  quote
FROM
  star_wars_quotes
ORDER BY
  pgml.embed(
    'intfloat/e5-small',
    'Feel the force!',
  ) <=> embedding
  DESC
LIMIT 1;
```

{% endtab %}
{% endtabs %}

This query will return the quote with the most similar meaning to `'Feel the force!'` by generating an embedding of that quote and comparing it to all other embeddings in the table, using vector cosine similarity as the measure of distance.
