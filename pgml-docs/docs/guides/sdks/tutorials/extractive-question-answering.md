# Extractive Question Answering

Here is the documentation for the JavaScript and Python code snippets performing end-to-end question answering:

Imports and Setup

**Python**

```python
from pgml import Collection, Model, Splitter, Pipeline, Builtins
from datasets import load_dataset
from dotenv import load_dotenv
```

**JavaScript**

```js
const pgml = require("pgml");
require("dotenv").config();
```

The SDK and datasets are imported. Builtins are used in Python for transforming text.

### Initialize Collection

**Python**

```python
collection = Collection("squad_collection") 
```

**JavaScript**

```js
const collection = pgml.newCollection("my_javascript_eqa_collection");
```

A collection is created to hold context passages.

### Create Pipeline

**Python**

```python
model = Model()
splitter = Splitter()
pipeline = Pipeline("squadv1", model, splitter)
await collection.add_pipeline(pipeline)
```

**JavaScript**

```js
const pipeline = pgml.newPipeline(
  "my_javascript_eqa_pipeline",
  pgml.newModel(),
  pgml.newSplitter(),
);

await collection.add_pipeline(pipeline);
```

A pipeline is created and added to the collection.

### Upsert Documents

**Python**

```python
data = load_dataset("squad")

documents = [
  {"id": ..., "text": ...} 
  for r in data  
]

await collection.upsert_documents(documents)
```

**JavaScript**

```js
const documents = [
  {
    id: "...",
    text: "...",
  }
];

await collection.upsert_documents(documents);
```

Context passages from SQuAD are upserted into the collection.

### Query for Context

**Python**

```python
results = await collection.query()
  .vector_recall(query, pipeline) 
  .fetch_all()

context = " ".join(results[0][1]) 
```

**JavaScript**

```js
const queryResults = await collection
  .query()
  .vector_recall(query, pipeline)
  .fetch_all();

const context = queryResults
  .map(result => result[1])
  .join("\n");
```

A vector search query retrieves context passages.

### Query for Answer

**Python**

```python
builtins = Builtins()

answer = await builtins.transform(
  "question-answering", 
  [{"question": query, "context": context}]
)
```

**JavaScript**

```js
const builtins = pgml.newBuiltins();

const answer = await builtins.transform("question-answering", [
  JSON.stringify({question, context})
]);
```

The context is passed to a QA model to extract the answer.
