# Semantic Search using Instructor model

This shows using instructor models in the `pgml` SDK for more advanced use cases.

#### Imports and Setup

**Python**

```python
from pgml import Collection, Model, Splitter, Pipeline   
from datasets import load_dataset
from dotenv import load_dotenv
```

**JavaScript**

```js
const pgml = require("pgml");
require("dotenv").config(); 
```

#### Initialize Collection

**Python**

```python
collection = Collection("squad_collection_1")
```

**JavaScript**

```js
const collection = pgml.newCollection("my_javascript_qai_collection"); 
```

#### Create Pipeline

**Python**

```python
model = Model("hkunlp/instructor-base", parameters={
  "instruction": "Represent the Wikipedia document for retrieval: "  
})

pipeline = Pipeline("squad_instruction", model, Splitter())
await collection.add_pipeline(pipeline)
```

**JavaScript**

```js
const model = pgml.newModel("hkunlp/instructor-base", "pgml", {
  instruction: "Represent the Wikipedia document for retrieval: ", 
});

const pipeline = pgml.newPipeline(
  "my_javascript_qai_pipeline",
  model,
  pgml.newSplitter(),
);

await collection.add_pipeline(pipeline);
```

#### Upsert Documents

**Python**

```python
data = load_dataset("squad")

documents = [
  {"id": ..., "text": ...} for r in data
]

await collection.upsert_documents(documents) 
```

**JavaScript**

```js
const documents = [
  {
    id: "...",
    text: "...",
  },
];

await collection.upsert_documents(documents);
```

#### Query

**Python**

```python
results = await collection.query()
  .vector_recall(query, pipeline, {
    "instruction": "Represent the Wikipedia question for retrieving supporting documents: "
  })
  .fetch_all()
```

**JavaScript**

```js
const queryResults = await collection
  .query()
  .vector_recall(query, pipeline, { 
    instruction:  
      "Represent the Wikipedia question for retrieving supporting documents: ",
  })
  .fetch_all();
```

