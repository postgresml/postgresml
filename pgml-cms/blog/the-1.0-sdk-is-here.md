---
featured: false
tags:
  - product
description: >-
  Our official pgml SDK has been stabilized and released for Python and
  JavaScript.
---

# The 1.0 SDK is Here

<div align="left">

<figure><img src=".gitbook/assets/silas.jpg" alt="Author" width="158"><figcaption></figcaption></figure>

</div>

Silas Marvin

March 4, 2023

## Announcing the Release of our Official PGML 1.0 SDK

We have spent the last few months stabilizing and finalizing the 1.0 version of our SDK in both JavaScript and Python.

This release comes with a bunch of performance improvements and new features. To highlight a few of the capabilities of our new SDK:

* Create Collections for storing, searching over, and managing groups of documents
* Define powerful and flexible Pipelines to dictate ingesting, splitting, embedding, and indexing of documents
* Search over documents and document chunks using semantic search, full text search, or hybrid semantic and full text search with extensive options for filtering on additional metadata
* Utilize almost any of the powerful embedding models available on HuggingFace
* It's all SQL! Get hands on with an ER diagram of your Collection and query from it however you want

Our SDK has been built specifically with the task of searching in mind. [We use it power the search on our own website](https://github.com/postgresml/postgresml/blob/6ba605d67016a1177d410d1eb91ae8763b4784c4/pgml-dashboard/src/utils/markdown.rs#L1243), [and to perform RAG with our ChatBot demo](https://github.com/postgresml/postgresml/blob/b3b5f03eb6c54bec88120617d5175279273d81d1/pgml-dashboard/src/api/chatbot.rs#L527).

## Why It's Exciting

Our SDK is no different from any other companies. It abstracts away some complexities of managing SQL tables, building complex queries, and other boring and repetitive tasks, but the SDK itself is not groundbreaking.

We think our SDK release is exciting because the underlying technology we use is something worth being excited about. Our SDK relies on our open source postgres extension to perform machine learning tasks using SQL. The lightning fast document embedding and magic-like hybrid search are all relatively simple SQL queries utilizing our postgres extension. Everything happens locally in your database without using any network calls.

What does it actually look like? Given some Collection and Pipeline defined below:

{% tabs %}
{% tab title="JavaScript" %}
```javascript
// Create Collection and Pipeline
const collection = pgml.newCollection("my_collection");
const pipeline = pgml.newPipeline("my_pipeline", {
  text: {
    splitter: { model: "recursive_character" },
    semantic_search: {
      model: "Alibaba-NLP/gte-base-en-v1.5",
    },
  },
});
await collection.add_pipeline(pipeline);

// Upsert a document
const documents = [
  { id: "document_one", text: "Here is some hidden value 1000" }
];
await collection.upsert_documents(documents);

// Search over our collection
const results = await collection.vector_search(
  {
    query: {
      fields: {
        text: {
          query: "What is the hidden value?"
        },
      },
    },
    limit: 5,
  },
  pipeline,
);
console.log(results);
```
{% endtab %}

{% tab title="Python" %}
```python
# Create Collection and Pipeline
collection = Collection("my_collection")
pipeline = Pipeline(
    "my_pipeline",
    {
        "text": {
            "splitter": {"model": "recursive_character"},
            "semantic_search": {
                "model": "Alibaba-NLP/gte-base-en-v1.5",
            },
        },
    },
)

# Upsert a document
documents = [{"id": "document_one", "text": "Here is some hidden value 1000"}]
await collection.upsert_documents(documents)

# Search over our collection
results = await collection.vector_search(
    {
        "query": {
            "fields": {
                "text": {"query": "What is the hidden value?"},
            },
        },
        "limit": 5,
    },
    pipeline,
)
print(results)
```
{% endtab %}
{% endtabs %}

The SQL for the vector\_search is actually just:

```postgresql
WITH "pipeline" (
    "schema"
) AS (
    SELECT
        "schema"
    FROM
        "my_collection"."pipelines"
    WHERE
        "name" = 'my_pipeline'
),
"text_embedding" (
    "embedding"
) AS (
    SELECT
        pgml.embed (transformer => (
                SELECT
                    SCHEMA #>> '{text,semantic_search,model}'
                FROM pipeline), text => 'What is the hidden value?', kwargs => '{}') AS "embedding"
)
SELECT
    "document",
    "chunk",
    "score"
FROM (
    SELECT
        1 - (embeddings.embedding <=> (
                SELECT
                    embedding
                FROM "text_embedding")::vector) AS score,
        "documents"."id",
        "chunks"."chunk",
        "documents"."document"
    FROM
        "my_collection_my_pipeline"."text_embeddings" AS "embeddings"
        INNER JOIN "my_collection_my_pipeline"."text_chunks" AS "chunks" ON "chunks"."id" = "embeddings"."chunk_id"
        INNER JOIN "my_collection"."documents" AS "documents" ON "documents"."id" = "chunks"."document_id"
    ORDER BY
        embeddings.embedding <=> (
            SELECT
                embedding
            FROM "text_embedding")::vector ASC
    LIMIT 5) AS "s"
ORDER BY
    "score" DESC
LIMIT 5

```

> NOTE: This SQL is programmatically generated and built to work in situations where the query is searching over more than one field. That is why you see a redundant limit and sort. It doesn't tangibly affect the speed of the query in this case

In fact, you can see every SQL query the SDK runs if you enable debug logging.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
pgml.init_logger("DEBUG");
```
{% endtab %}

{% tab title="Python" %}
```python
pgml.init_logger("DEBUG");
```
{% endtab %}
{% endtabs %}

Want to see an ER diagram of your collection?

{% tabs %}
{% tab title="JavaScript" %}
```javascript
console.log(await collection.generate_er_diagram(pipeline));
```
{% endtab %}

{% tab title="Python" %}
```python
print(await collection.generate_er_diagram(pipeline))
```
{% endtab %}
{% endtabs %}

The above code prints out PlantUML script. Paste it into their online interpreter and checkout [the resulting diagram](https://www.plantuml.com/plantuml/uml/lPD1hjiW48Rtd6BqDbqz7w2hTnE4OMgJ08DWS9B6lNinbaELjceNqSk6\_F-WcUz7uu\_CAd7nJdo1sHe4dX5o93wqjaax55MgXQo1c6Xqw3DSBC-WmkJGW4vqoV0DaKK-sn1LKXwS3SYtY429Pn820rk-mLkSl1iqEOUQBONy1Yh3Pcgu2wY\_EkKhZ7QoWPj-Vs-7JgWOZLHSosmzLdGV6mSLRWvyfu3jSb0UjsjuvQPLdRLipaZaK8LcrYod2Y6V1sPpbWkcNEcE7Zywlx\_9JZyOqiNNqXxZeLuO9LD96cKfhTbsDFiOLRrJfZ3-7J7QYCu6t14VwhDVE-iPlVedhgpgO1osZbBF9Pnt-AvVXj-VylT5Q9Ea3GQlVoWSYVy\_2VeHZR5Xwccwzwf47VovqsDKjPVAI6bZtp-zTHs6TUtR8KJVvLQx\_\_huelzlvNLz3YC-C9ZYtKy0)[.](https://www.plantuml.com/plantuml/uml/lPD1hjiW48Rtd6BqDbqz7w2hTnE4OMgJ08DWS9B6lNinbaELjceNqSk6\_F-WcUz7uu\_CAd7nJdo1sHe4dX5o93wqjaax55MgXQo1c6Xqw3DSBC-WmkJGW4vqoV0DaKK-sn1LKXwS3SYtY429Pn820rk-mLkSl1iqEOUQBONy1Yh3Pcgu2wY\_EkKhZ7QoWPj-Vs-7JgWOZLHSosmzLdGV6mSLRWvyfu3jSb0UjsjuvQPLdRLipaZaK8LcrYod2Y6V1sPpbWkcNEcE7Zywlx\_9JZyOqiNNqXxZeLuO9LD96cKfhTbsDFiOLRrJfZ3-7J7QYCu6t14VwhDVE-iPlVedhgpgO1osZbBF9Pnt-AvVXj-VylT5Q9Ea3GQlVoWSYVy\_2VeHZR5Xwccwzwf47VovqsDKjPVAI6bZtp-zTHs6TUtR8KJVvLQx\_\_huelzlvNLz3YC-C9ZYtKy0)

Thanks for reading about the release of our 1.0 SDK. We hope you are as excited about it as we are!
