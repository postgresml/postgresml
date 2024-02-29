---
description: Example for Semantic Search
---

# Semantic Search

This tutorial demonstrates using the `pgml` SDK to create a collection, add documents, build a pipeline for vector search, make a sample query, and archive the collection when finished. It loads sample data, indexes questions, times a semantic search query, and prints formatted results.

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
from pgml import Collection, Model, Splitter, Pipeline  
from datasets import load_dataset
from dotenv import load_dotenv
import asyncio
```
{% endtab %}
{% endtabs %}

## Initialize Collection

A collection object is created to represent the search collection.

{% tabs %}
{% tab title="JavaScript" %}
```js
const main = async () => {
  const collection = pgml.newCollection("my_javascript_collection");
}
```
{% endtab %}

{% tab title="Python" %}
```python
async def main():
    load_dotenv()
    collection = Collection("my_collection") 
```
{% endtab %}
{% endtabs %}

## Create Pipeline

A pipeline encapsulating a model and splitter is created and added to the collection.

{% tabs %}
{% tab title="JavaScript" %}
```js
const model = pgml.newModel();
const splitter = pgml.newSplitter();
const pipeline = pgml.newPipeline("my_javascript_pipeline", model, splitter);
await collection.add_pipeline(pipeline); 
```
{% endtab %}

{% tab title="Python" %}
```python
model = Model()
splitter = Splitter()
pipeline = Pipeline("my_pipeline", model, splitter)
await collection.add_pipeline(pipeline)
```
{% endtab %}
{% endtabs %}

## Upsert Documents

Documents are upserted into the collection and indexed by the pipeline.

{% tabs %}
{% tab title="JavaScript" %}
```js
const documents = [
  {
    id: "Document One",
    text: "...",
  },
  {
    id: "Document Two", 
    text: "...",
  },
];

await collection.upsert_documents(documents);
```
{% endtab %}

{% tab title="Python" %}
```python
documents = [
    {"id": "doc1", "text": "..."},
    {"id": "doc2", "text": "..."}
]

await collection.upsert_documents(documents)  
```
{% endtab %}
{% endtabs %}

## Query

A vector similarity search query is made on the collection.

{% tabs %}
{% tab title="JavaScript" %}
```js
const queryResults = await collection
  .query()
  .vector_recall(
    "query",
    pipeline,
  )
  .fetch_all();
```
{% endtab %}

{% tab title="Python" %}
```python
results = await collection.query()
    .vector_recall("query", pipeline)
    .fetch_all() 
```
{% endtab %}
{% endtabs %}

## Archive Collection

The collection is archived when finished.

{% tabs %}
{% tab title="JavaScript" %}
```js
await collection.archive();
```
{% endtab %}

{% tab title="Python" %}
```python
await collection.archive()
```
{% endtab %}
{% endtabs %}

## Main

Boilerplate to call main() async function.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
main().then((results) => { 
  console.log("Vector search Results: \n", results); 
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
