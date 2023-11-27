---
description: Example for Semantic Search
---

# Semantic Search

This tutorial demonstrates using the `pgml` SDK to create a collection, add documents, build a pipeline for vector search, make a sample query, and archive the collection when finished. It loads sample data, indexes questions, times a semantic search query, and prints formatted results.



### Imports and Setup

**Python**

```python
from pgml import Collection, Model, Splitter, Pipeline  
from datasets import load_dataset
from dotenv import load_dotenv
import asyncio
```

**JavaScript**

```js
const pgml = require("pgml");

require("dotenv").config();
```

The SDK is imported and environment variables are loaded.

### Initialize Collection

**Python**

```python
async def main():

  load_dotenv()

  collection = Collection("my_collection") 
```

**JavaScript**

```js
const main = async () => {

  const collection = pgml.newCollection("my_javascript_collection");

}
```

A collection object is created to represent the search collection.

### Create Pipeline

**Python**

```python
  model = Model()
  splitter = Splitter()

  pipeline = Pipeline("my_pipeline", model, splitter)

  await collection.add_pipeline(pipeline)
```

**JavaScript**

```js
  const model = pgml.newModel();

  const splitter = pgml.newSplitter();

  const pipeline = pgml.newPipeline("my_javascript_pipeline", model, splitter);

  await collection.add_pipeline(pipeline); 
```

A pipeline encapsulating a model and splitter is created and added to the collection.

### Upsert Documents

**Python**

```python
  documents = [
    {"id": "doc1", "text": "..."},
    {"id": "doc2", "text": "..."}
  ]

  await collection.upsert_documents(documents)  
```

**JavaScript**

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

Documents are upserted into the collection and indexed by the pipeline.

### Query

**Python**

```python
  results = await collection.query()
    .vector_recall("query", pipeline)
    .fetch_all() 
```

**JavaScript**

```js
  const queryResults = await collection
    .query()
    .vector_recall(
      "query",
      pipeline,
    )
    .fetch_all();
```

A vector similarity search query is made on the collection.

### Archive Collection

**Python**

```python
  await collection.archive()
```

**JavaScript**

```js
  await collection.archive();
```

The collection is archived when finished.

Let me know if you would like me to modify or add anything!

### Main

**Python**

```python
if __name__ == "__main__":
    asyncio.run(main())
```

**JavaScript**

```javascript
main().then((results) => { 
console.log("Vector search Results: \n", results); 
});
```

Boilerplate to call main() async function.

Let me know if you would like me to modify or add anything to this markdown documentation. Happy to iterate on it!
