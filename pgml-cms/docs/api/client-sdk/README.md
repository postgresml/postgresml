---
description: PostgresML client SDK for JavaScript, Python and Rust implements common use cases and PostgresML connection management.
---

# Client SDK

The client SDK can be installed using standard package managers for JavaScript, Python, and Rust. Since the SDK is written in Rust, the JavaScript and Python packages comes with no additional dependencies.


## Installation

Installing the SDK into your project is as simple as:

{% tabs %}
{% tab title="JavaScript " %}
```bash
npm i pgml
```
{% endtab %}

{% tab title="Python " %}
```bash
pip install pgml
```
{% endtab %}
{% endtabs %}

## Getting started

The SDK uses the database to perform all of its functionality. Before continuing, make sure you've created a [PostgresML database](https://postgresml.org/signup) and have the `DATABASE_URL` connection string handy.

### Connect to PostgresML

The SDK automatically manages connections to PostgresML. The connection string can be specified as an argument to the collection constructor, or as an environment variable.

If your app follows the twelve-factor convention, we recommend you configure the connection in the environment using the `PGML_DATABASE_URL` variable:

```bash
export PGML_DATABASE_URL=postgres://user:password@sql.cloud.postgresml.org:6432/pgml_database
```

### Create a collection

The SDK is written in asynchronous code, so you need to run it inside an async runtime. Both Python and JavaScript support async functions natively.

{% tabs %}
{% tab title="JavaScript " %}
```javascript
const pgml = require("pgml");

const main = async () => {
  const collection = pgml.newCollection("sample_collection");
}
```
{% endtab %}

{% tab title="Python" %}
```python
from pgml import Collection, Pipeline
import asyncio

async def main():
    collection = Collection("sample_collection")
```
{% endtab %}
{% endtabs %}

The example above imports the `pgml` module and creates a collection object. By itself, the collection tracks document contents and identifiers, but once we add a pipeline, we can instruct the SDK to perform additional tasks when documents and inserted and retrieved.


### Create a pipeline

Continuing the example, we will create a pipeline named `sample_pipeline`, which will use in-database embedding generation to automatically chunk and embed documents:

{% tabs %}
{% tab title="JavaScript" %}
```javascript
// Add this code to the end of the main function from the above example.
const pipeline = pgml.newPipeline("sample_pipeline", {
  text: {
    splitter: { model: "recursive_character" },
    semantic_search: {
      model: "intfloat/e5-small",
    },
  },
});

await collection.add_pipeline(pipeline);
```
{% endtab %}

{% tab title="Python" %}
```python
# Add this code to the end of the main function from the above example.
pipeline = Pipeline(
    "test_pipeline",
    {
        "text": {
            "splitter": { "model": "recursive_character" },
            "semantic_search": {
                "model": "intfloat/e5-small",
            },
        },
    },
)

await collection.add_pipeline(pipeline)
```
{% endtab %}
{% endtabs %}

The pipeline configuration is a key/value object, where the key is the name of a column in a document, and the value is the action the SDK should perform on that column. 

In this example, the documents contain a column called `text` which we are instructing the SDK to chunk the contents of using the recursive character splitter, and to embed those chunks using the Hugging Face `intfloat/e5-small` embeddings model.

### Add documents

Once the pipeline is configured, we can start adding documents:

{% tabs %}
{% tab title="JavaScript" %}
```javascript
// Add this code to the end of the main function from the above example.
const documents = [
  {
    id: "Document One",
    text: "document one contents...",
  },
  {
    id: "Document Two",
    text: "document two contents...",
  },
];

await collection.upsert_documents(documents);
```
{% endtab %}

{% tab title="Python" %}
```python
# Add this code to the end of the main function in the above example.
documents = [
    {
        "id": "Document One",
        "text": "document one contents...",
    },
    {
        "id": "Document Two",
        "text": "document two contents...",
    },
]

await collection.upsert_documents(documents)
```
{% endtab %}
{% endtabs %}

If the same document `id` is used, the SDK computes the difference between existing and new documents and only updates the chunks that have changed.

### Search documents

Now that the documents are stored, chunked and embedded, we can start searching the collection:

{% tabs %}
{% tab title="JavaScript" %}
```javascript
// Add this code to the end of the main function in the above example.
const results = await collection.vector_search(
  {
    query: {
      fields: {
        text: {
          query: "Something about a document...",
        },
      },
    },
    limit: 2,
  },
  pipeline,
);

console.log(results);
```
{% endtab %}

{% tab title="Python" %}
```python
# Add this code to the end of the main function in the above example.
results = await collection.vector_search(
    {
        "query": {
            "fields": {
                "text": {
                    "query": "Something about a document...",
                },
            },
        },
        "limit": 2,
    },
    pipeline,
)

print(results)
```
{% endtab %}
{% endtabs %}

We are using built-in vector search, powered by embeddings and the PostgresML [pgml.embed()](../sql-extension/pgml.embed) function, which embeds the `query` argument, compares it to the embeddings stored in the database, and returns the top two results, ranked by cosine similarity.

### Run the example

Since the SDK is using async code, both JavaScript and Python need a little bit of code to run it correctly:

{% tabs %}
{% tab title="JavaScript" %}
```javascript
main().then(() => {
  console.log("SDK example complete");
});
```
{% endtab %}

{% tab title="Python" %}
```python
if __name__ == "__main__":
    asyncio.run(main())
```
{% endtab %}
{% endtabs %}

Once you run the example, you should see something like this in the terminal:

```bash
[
    {
        "chunk": "document one contents...",
        "document": {"id": "Document One", "text": "document one contents..."},
        "score": 0.9034339189529419,
    },
    {
        "chunk": "document two contents...",
        "document": {"id": "Document Two", "text": "document two contents..."},
        "score": 0.8983734250068665,
    },
]
```

