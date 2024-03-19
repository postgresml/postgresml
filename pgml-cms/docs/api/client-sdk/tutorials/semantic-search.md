---
description: Example for Semantic Search
---

# Semantic Search

This tutorial demonstrates using the `pgml` SDK to create a collection, add documents, build a pipeline for vector search, make a sample query, and archive the collection when finished.

[Link to full JavaScript implementation](../../../../../pgml-sdks/pgml/javascript/examples/semantic\_search.js)

[Link to full Python implementation](../../../../../pgml-sdks/pgml/python/examples/semantic\_search.py)

## Imports and Setup

The SDK is imported and environment variables are loaded.

{% tabs %}
{% tab title="JavasScript" %}
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
  const collection = pgml.newCollection("semantic_search_collection");
```
{% endtab %}

{% tab title="Python" %}
```python
async def main(): # Start the main function, we end it after archiving
    load_dotenv()
    console = Console()

    # Initialize collection
    collection = Collection("quora_collection")
```
{% endtab %}
{% endtabs %}

## Create Pipeline

A pipeline encapsulating a model and splitter is created and added to the collection.

{% tabs %}
{% tab title="JavaScript" %}
```js
  // Add a pipeline
  const pipeline = pgml.newPipeline("semantic_search_pipeline", {
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
    # Create and add pipeline
    pipeline = Pipeline(
        "quorav1",
        {
            "text": {
                "splitter": {"model": "recursive_character"},
                "semantic_search": {"model": "intfloat/e5-small"},
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
    # Prep documents for upserting
    dataset = load_dataset("quora", split="train")
    questions = []
    for record in dataset["questions"]:
        questions.extend(record["text"])

    # Remove duplicates and add id
    documents = []
    for i, question in enumerate(list(set(questions))):
        if question:
            documents.append({"id": i, "text": question})

    # Upsert documents
    await collection.upsert_documents(documents[:2000])
```
{% endtab %}
{% endtabs %}

## Query

A vector similarity search query is made on the collection.

{% tabs %}
{% tab title="JavaScript" %}
```js
  // Perform vector search
  const query = "Something that will match document one first";
  const queryResults = await collection.vector_search(
    {
      query: {
        fields: {
          text: { query: query }
        }
      }, limit: 2
    }, pipeline);
  console.log("The results");
  console.log(queryResults);
```
{% endtab %}

{% tab title="Python" %}
```python
    # Query
    query = "What is a good mobile os?"
    console.print("Querying for %s..." % query)
    start = time()
    results = await collection.vector_search(
        {"query": {"fields": {"text": {"query": query}}}, "limit": 5}, pipeline
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
