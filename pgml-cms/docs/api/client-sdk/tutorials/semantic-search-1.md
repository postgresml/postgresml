---
description: Example for Semantic Search
---

# Semantic Search Using Instructor Model

This tutorial demonstrates using the `pgml` SDK to create a collection, add documents, build a pipeline for vector search, make a sample query, and archive the collection when finished.  In this tutorial we use [Alibaba-NLP/gte-base-en-v1.5](https://huggingface.co/Alibaba-NLP/gte-base-en-v1.5).

[Link to full JavaScript implementation](https://github.com/postgresml/postgresml/blob/master/pgml-sdks/pgml/javascript/examples/question_answering.js)

[Link to full Python implementation](https://github.com/postgresml/postgresml/blob/master/pgml-sdks/pgml/python/examples/question_answering.py)

## Imports and Setup

The SDK is imported and environment variables are loaded.

{% tabs %}
{% tab title="JavaScript" %}
```js
const pgml = require("pgml");
require("dotenv").config();
```
{% endtab %}

{% tab title="Python" %}
```python
from pgml import Collection, Pipeline
from datasets import load_dataset
from time import time
from dotenv import load_dotenv
from rich.console import Console
import asyncio
```
{% endtab %}
{% endtabs %}

## Initialize Collection

A collection object is created to represent the search collection.

{% tabs %}
{% tab title="JavaScript" %}
```js
const main = async () => { // Open the main function, we close it at the bottom
  // Initialize the collection
  const collection = pgml.newCollection("qa_collection");
```
{% endtab %}

{% tab title="Python" %}
```python
async def main(): # Start the main function, we end it after archiving
    load_dotenv()
    console = Console()

    # Initialize collection
    collection = Collection("squad_collection")
```
{% endtab %}
{% endtabs %}

## Create Pipeline

A pipeline encapsulating a model and splitter is created and added to the collection.

{% tabs %}
{% tab title="JavaScript" %}
```js
  // Add a pipeline
  const pipeline = pgml.newPipeline("qa_pipeline", {
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
    # Create and add pipeline
    pipeline = Pipeline(
        "squadv1",
        {
            "text": {
                "splitter": {"model": "recursive_character"},
                "semantic_search": {
                    "model": "Alibaba-NLP/gte-base-en-v1.5",
                },
            }
        },
    )
    await collection.add_pipeline(pipeline)
```
{% endtab %}
{% endtabs %}

## Upsert Documents

Documents are upserted into the collection and indexed by the pipeline.

{% tabs %}
{% tab title="JavaScript" %}
```js
  // Upsert documents, these documents are automatically split into chunks and embedded by our pipeline
  const documents = [
    {
      id: "Document One",
      text: "PostgresML is the best tool for machine learning applications!",
    },
    {
      id: "Document Two",
      text: "PostgresML is open source and available to everyone!",
    },
  ];
  await collection.upsert_documents(documents);
```
{% endtab %}

{% tab title="Python" %}
```python
    # Prep documents for upserting
    data = load_dataset("squad", split="train")
    data = data.to_pandas()
    data = data.drop_duplicates(subset=["context"])
    documents = [
        {"id": r["id"], "text": r["context"], "title": r["title"]}
        for r in data.to_dict(orient="records")
    ]

    # Upsert documents
    await collection.upsert_documents(documents[:200])
```
{% endtab %}
{% endtabs %}

## Query

A vector similarity search query is made on the collection.

{% tabs %}
{% tab title="JavaScript" %}
```js
  // Perform vector search
  const query = "What is the best tool for building machine learning applications?";
  const queryResults = await collection.vector_search(
    {
      query: {
        fields: {
          text: { query: query }
        }
      }, limit: 1
    }, pipeline);
  console.log(queryResults);
```
{% endtab %}

{% tab title="Python" %}
```python
    # Query for answer
    query = "Who won more than 20 grammy awards?"
    console.print("Querying for context ...")
    start = time()
    results = await collection.vector_search(
        {
            "query": {
                "fields": {
                    "text": {
                        "query": query,
                        "parameters": {
                            "instruction": "Represent the Wikipedia question for retrieving supporting documents: "
                        },
                    },
                }
            },
            "limit": 5,
        },
        pipeline,
    )
    end = time()
    console.print("\n Results for '%s' " % (query), style="bold")
    console.print(results)
    console.print("Query time = %0.3f" % (end - start))
```
{% endtab %}
{% endtabs %}

## Archive Collection

The collection is archived when finished.

{% tabs %}
{% tab title="JavaScript" %}
```js
  await collection.archive();
} // Close the main function
```
{% endtab %}

{% tab title="Python" %}
```python
    await collection.archive()
# The end of the main function
```
{% endtab %}
{% endtabs %}

## Main

Boilerplate to call main() async function.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
main().then(() => console.log("Done!"));
```
{% endtab %}

{% tab title="Python" %}
```python
if __name__ == "__main__":
    asyncio.run(main())
```
{% endtab %}
{% endtabs %}
