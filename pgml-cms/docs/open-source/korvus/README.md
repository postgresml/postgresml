---
description: PostgresML client SDK for JavaScript, Python and Rust implements common use cases and PostgresML connection management.
---

# Client SDK

The client SDK can be installed using standard package managers for JavaScript, Python, and Rust. Since the SDK is written in Rust, the JavaScript and Python packages come with no additional dependencies.


## Installation

Installing the SDK into your project is as simple as:

{% tabs %}
{% tab title="JavaScript" %}
```bash
npm i korvus
```
{% endtab %}

{% tab title="Python" %}
```bash
pip install korvus
```
{% endtab %}

{% tab title="Rust" %}
```bash
cargo add korvus
```
{% endtab %}

{% tab title="C" %}

First clone the `korvus` repository and navigate to the `korvus/c` directory:
```bash
git clone https://github.com/postgresml/korvus
cd korvus/korvus/c
```

Then build the bindings
```bash
make bindings
```

This will generate the `korvus.h` file and a `.so` on linux and `.dyblib` on MacOS.
{% endtab %}
{% endtabs %}

## Getting started

The SDK uses the database to perform most of its functionality. Before continuing, make sure you created a [PostgresML database](https://postgresml.org/signup) and have the `DATABASE_URL` connection string handy.

### Connect to PostgresML

The SDK automatically manages connections to PostgresML. The connection string can be specified as an argument to the collection constructor, or as an environment variable.

If your app follows the twelve-factor convention, we recommend you configure the connection in the environment using the `KORVUS_DATABASE_URL` variable:

```bash
export KORVUS_DATABASE_URL=postgres://user:password@sql.cloud.postgresml.org:6432/korvus_database
```

### Create a collection

The SDK is written in asynchronous code, so you need to run it inside an async runtime. Both Python, JavaScript and Rust support async functions natively.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const korvus = require("korvus");

const main = async () => {
  const collection = korvus.newCollection("sample_collection");
}
```
{% endtab %}

{% tab title="Python" %}
```python
from korvus import Collection, Pipeline
import asyncio

async def main():
    collection = Collection("sample_collection")
```
{% endtab %}

{% tab title="Rust" %}
```rust
use korvus::{Collection, Pipeline};
use anyhow::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut collection = Collection::new("sample_collection", None)?;
}
```
{% endtab %}

{% tab title="C" %}
```cpp
#include <stdio.h>
#include "korvus.h"

int main() {
  CollectionC * collection = korvus_collectionc_new("sample_collection", NULL);
}
```
{% endtab %}
{% endtabs %}

The above example imports the `korvus` module and creates a collection object. By itself, the collection only tracks document contents and identifiers, but once we add a pipeline, we can instruct the SDK to perform additional tasks when documents and are inserted and retrieved.


### Create a pipeline

Continuing the example, we will create a pipeline called `sample_pipeline`, which will use in-database embeddings generation to automatically chunk and embed documents:

{% tabs %}
{% tab title="JavaScript" %}
```javascript
// Add this code to the end of the main function from the above example.
const pipeline = korvus.newPipeline("sample_pipeline", {
  text: {
    splitter: { model: "recursive_character" },
    semantic_search: {
      model: "Alibaba-NLP/gte-base-en-v1.5",
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
    "sample_pipeline",
    {
        "text": {
            "splitter": { "model": "recursive_character" },
            "semantic_search": {
                "model": "Alibaba-NLP/gte-base-en-v1.5",
            },
        },
    },
)

await collection.add_pipeline(pipeline)
```
{% endtab %}

{% tab title="Rust" %}
```rust
// Add this code to the end of the main function from the above example.
let mut pipeline = Pipeline::new(
    "sample_pipeline",
    Some(
        serde_json::json!({
            "text": {
                "splitter": { "model": "recursive_character" },
                "semantic_search": {
                    "model": "Alibaba-NLP/gte-base-en-v1.5",
                },
            },
        })
        .into(),
    ),
)?;

collection.add_pipeline(&mut pipeline).await?;
```
{% endtab %}

{% tab title="C" %}
```cpp
// Add this code to the end of the main function from the above example.
PipelineC * pipeline = korvus_pipelinec_new("sample_pipeline", "{\"text\": {\"splitter\": {\"model\": \"recursive_character\"},\"semantic_search\": {\"model\": \"Alibaba-NLP/gte-base-en-v1.5\"}}}");

korvus_collectionc_add_pipeline(collection, pipeline);
```
{% endtab %}
{% endtabs %}

The pipeline configuration is a key/value object, where the key is the name of a column in a document, and the value is the action the SDK should perform on that column. 

In this example, the documents contain a column called `text` which we are instructing the SDK to chunk the contents of using the recursive character splitter, and to embed those chunks using the Hugging Face `Alibaba-NLP/gte-base-en-v1.5` embeddings model.

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

{% tab title="Rust" %}
```rust
// Add this code to the end of the main function in the above example.
let documents = vec![
    serde_json::json!({
        "id": "Document One",
        "text": "document one contents...",
    })
    .into(),
    serde_json::json!({
        "id": "Document Two",
        "text": "document two contents...",
    })
    .into(),
];

collection.upsert_documents(documents, None).await?;
```
{% endtab %}

{% tab title="C" %}
```cpp
// Add this code to the end of the main function in the above example.
char * documents_to_upsert[2] = {"{\"id\": \"Document One\", \"text\": \"document one contents...\"}", "{\"id\": \"Document Two\", \"text\": \"document two contents...\"}"};

korvus_collectionc_upsert_documents(collection, documents_to_upsert, 2, NULL);
```
{% endtab %}
{% endtabs %}

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

{% tab title="Rust" %}
```rust
// Add this code to the end of the main function in the above example.
let results = collection
    .vector_search(
        serde_json::json!({
            "query": {
                "fields": {
                    "text": {
                        "query": "Something about a document...",
                    },
                },
            },
            "limit": 2,
        })
        .into(),
        &mut pipeline,
    )
    .await?;

println!("{:?}", results);

Ok(())
```
{% endtab %}

{% tab title="C" %}
```cpp
// Add this code to the end of the main function in the above example.
r_size = 0;
char** results = korvus_collectionc_vector_search(collection, "{\"query\": {\"fields\": {\"text\": {\"query\": \"Something about a document...\"}}}, \"limit\": 2}", pipeline, &r_size);
printf("\n\nPrinting results:\n");
for (i = 0; i < r_size; ++i) {
  printf("Result %u -> %s\n", i, results[i]);
}

korvus_pipelinec_delete(pipeline);
korvus_collectionc_delete(collection);
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

Note that `Rust` and `C` example do not require any additional code to run correctly.

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

