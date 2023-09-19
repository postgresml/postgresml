# Summarizing Question Answering

Here are the Python and JavaScript examples for text summarization using `pgml` SDK

### Imports and Setup

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

The SDK and datasets are imported. Builtins are used for transformations.

### Initialize Collection

**Python**

```python
collection = Collection("squad_collection")
```

**JavaScript**

```js
const collection = pgml.newCollection("my_javascript_sqa_collection"); 
```

A collection is created to hold text passages.

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
  "my_javascript_sqa_pipeline",
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

Text passages are upserted into the collection.

### Query for Context

**Python**

```python
results = await collection.query()
  .vector_recall(query, pipeline)
  .fetch_all()

context = results[0][1]  
```

**JavaScript**

```js
const queryResults = await collection
  .query()
  .vector_recall(query, pipeline) 
  .fetch_all();

const context = queryResults[0][1];
```

A vector search retrieves a relevant text passage.

### Summarize Text

**Python**

```python
builtins = Builtins()

summary = await builtins.transform(
  {"task": "summarization", 
   "model": "sshleifer/distilbart-cnn-12-6"},
  [context]
)
```

**JavaScript**

```js
const builtins = pgml.newBuiltins();

const summary = await builtins.transform(
  {task: "summarization", 
   model: "sshleifer/distilbart-cnn-12-6"},
  [context]
);
```

The text is summarized using a pretrained model.
