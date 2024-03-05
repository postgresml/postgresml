---
description: Pipelines are composed of a model, splitter, and additional optional arguments. 
---

# Pipelines

`Pipeline`s define the schema for the transformation of documents. Different `Pipeline`s can be used for different tasks.&#x20;

## Defining Schema

New `Pipeline`s require schema. Here are a few examples of variations of schema along with common use cases.&#x20;

For the following section we will assume we have documents that have the structure:

```json
{
  "id": "Each document has a unique id",
  "title": "Each document has a title",
  "body": "Each document has some body text"
}
```

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const pipeline = pgml.newPipeline("test_pipeline", {
  title: {
    full_text_search: { configuration: "english" },
  },
  body: {
    splitter: { model: "recursive_character" },
    semantic_search: {
      model: "hkunlp/instructor-base",
      parameters: {
        instruction: "Represent the Wikipedia document for retrieval: ",
      }
    },
  },
});
```
{% endtab %}

{% tab title="Python" %}
```python
pipeline = Pipeline(
    "test_pipeline",
    {
        "title": {
            "full_text_search": {"configuration": "english"},
        },
        "body": {
            "splitter": {"model": "recursive_character"},
            "semantic_search": {
                "model": "hkunlp/instructor-base",
                "parameters": {
                    "instruction": "Represent the Wikipedia document for retrieval: ",
                },
            },
        },
    },
)
```
{% endtab %}
{% endtabs %}

This `Pipeline` does two things. For each document in the `Collection`, it converts all `title`s into tsvectors enabling full text search, and splits and embeds the `body` text enabling semantic search using vectors. This kind of `Pipeline` would be great for site search utilizing hybrid keyword and semantic search.

For a more simple RAG use case, the following `Pipeline` would work well.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const pipeline = pgml.newPipeline("test_pipeline", {
  body: {
    splitter: { model: "recursive_character" },
    semantic_search: {
      model: "hkunlp/instructor-base",
      parameters: {
        instruction: "Represent the Wikipedia document for retrieval: ",
      }
    },
  },
});
```
{% endtab %}

{% tab title="Python" %}
```python
pipeline = Pipeline(
    "test_pipeline",
    {
        "body": {
            "splitter": {"model": "recursive_character"},
            "semantic_search": {
                "model": "hkunlp/instructor-base",
                "parameters": {
                    "instruction": "Represent the Wikipedia document for retrieval: ",
                },
            },
        },
    },
)
```
{% endtab %}
{% endtabs %}

This `Pipeline` splits and embeds the `body` text enabling semantic search using vectors. This is a very popular `Pipeline` for RAG.&#x20;

We support most every open source model on [Hugging Face](https://huggingface.co/), and OpenAI's embedding models. To use a model from OpenAI specify the `source` as `openai`, and make sure and set the environment variable `OPENAI_API_KEY`.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const pipeline = pgml.newPipeline("test_pipeline", {
  body: {
    splitter: { model: "recursive_character" },
    semantic_search: {
      model: "text-embedding-ada-002",
      source: "openai"
    },
  },
});
```
{% endtab %}

{% tab title="Python" %}
```python
pipeline = Pipeline(
    "test_pipeline",
    {
        "body": {
            "splitter": {"model": "recursive_character"},
            "semantic_search": {"model": "text-embedding-ada-002", "source": "openai"},
        },
    },
)
```
{% endtab %}
{% endtabs %}

## Customizing the Indexes

By default the SDK uses HNSW indexes to efficiently perform vector recall. The default HNSW index sets `m` to 16 and `ef_construction` to 64. These defaults can be customized in the `Pipeline` schema. See [pgvector](https://github.com/pgvector/pgvector) for more information on vector indexes.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const pipeline = pgml.newPipeline("test_pipeline", {
  body: {
    splitter: { model: "recursive_character" },
    semantic_search: {
      model: "intfloat/e5-small",
      hnsw: {
        m: 100,
        ef_construction: 200
      }
    },
  },
});
```
{% endtab %}

{% tab title="Python" %}
```python
pipeline = Pipeline(
    "test_pipeline",
    {
        "body": {
            "splitter": {"model": "recursive_character"},
            "semantic_search": {
                "model": "intfloat/e5-small",
                "hnsw": {"m": 100, "ef_construction": 200},
            },
        },
    },
)
```
{% endtab %}
{% endtabs %}

## Adding Pipelines to a Collection

The first time a `Pipeline` is added to a `Collection` it will automatically chunk and embed any documents already in that `Collection`.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
await collection.add_pipeline(pipeline)
```
{% endtab %}

{% tab title="Python" %}
```python
await collection.add_pipeline(pipeline)
```
{% endtab %}
{% endtabs %}

> Note: After a `Pipeline` has been added to a `Collection` instances of the `Pipeline` object can be created without specifying a schema:

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const pipeline = pgml.newPipeline("test_pipeline")
```
{% endtab %}

{% tab title="Python" %}
```python
pipeline = Pipeline("test_pipeline")
```
{% endtab %}
{% endtabs %}

## Searching with Pipelines

There are two different forms of search that can be done after adding a `Pipeline` to a `Collection`&#x20;

* [Vector Search](https://postgresml.org/docs/introduction/apis/client-sdks/search)
* [Document Search](https://postgresml.org/docs/introduction/apis/client-sdks/document-search)

See their respective pages for more information on searching.

## **Disable a Pipeline**

`Pipelines` can be disabled or removed to prevent them from running automatically when documents are upserted.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const pipeline = pgml.newPipeline("test_pipeline")
const collection = pgml.newCollection("test_collection")
await collection.disable_pipeline(pipeline)
```
{% endtab %}

{% tab title="Python" %}
```python
pipeline = Pipeline("test_pipeline")
collection = Collection("test_collection")
await collection.disable_pipeline(pipeline)
```
{% endtab %}
{% endtabs %}

Disabling a `Pipeline` prevents it from running automatically, but leaves all tsvectors, chunks, and embeddings already created by that `Pipeline` in the database.

## **Enable a Pipeline**

Disabled `Pipeline`s can be re-enabled.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const pipeline = pgml.newPipeline("test_pipeline")
const collection = pgml.newCollection("test_collection")
await collection.enable_pipeline(pipeline)
```
{% endtab %}

{% tab title="Python" %}
```python
pipeline = Pipeline("test_pipeline")
collection = Collection("test_collection")
await collection.enable_pipeline(pipeline)
```
{% endtab %}
{% endtabs %}

Enabling a `Pipeline` will cause it to automatically run  on all documents it may have missed while disabled.

## **Remove a Pipeline**

{% tabs %}
{% tab title="JavaScript" %}
<pre class="language-javascript"><code class="lang-javascript">const pipeline = pgml.newPipeline("test_pipeline")
<strong>const collection = pgml.newCollection("test_collection")
</strong>await collection.remove_pipeline(pipeline)
</code></pre>
{% endtab %}

{% tab title="Python" %}
```python
pipeline = Pipeline("test_pipeline")
collection = Collection("test_collection")
await collection.remove_pipeline(pipeline)
```
{% endtab %}
{% endtabs %}

Removing a `Pipeline` deletes it and all associated data from the database. Removed `Pipelines` cannot be re-enabled but can be recreated.
