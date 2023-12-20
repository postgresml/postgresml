# Summarizing Question Answering

Here are the Python and JavaScript examples for text summarization using `pgml` SDK

## Imports and Setup

The SDK and datasets are imported. Builtins are used for transformations.

{% tabs %}
{% tab title="JavaScript" %}
```js
const pgml = require("pgml");
require("dotenv").config();
```
{% endtab %}

{% tab title="Python" %}
```python
from pgml import Collection, Model, Splitter, Pipeline, Builtins  
from datasets import load_dataset
from dotenv import load_dotenv
```
{% endtab %}
{% endtabs %}

## Initialize Collection

A collection is created to hold text passages.

{% tabs %}
{% tab title="JavaScript" %}
```js
const collection = pgml.newCollection("my_javascript_sqa_collection"); 
```
{% endtab %}

{% tab title="Python" %}
```python
collection = Collection("squad_collection")
```
{% endtab %}
{% endtabs %}

## Create Pipeline

A pipeline is created and added to the collection.

{% tabs %}
{% tab title="JavaScript" %}
```js
const pipeline = pgml.newPipeline(
  "my_javascript_sqa_pipeline",
  pgml.newModel(),
  pgml.newSplitter(), 
);

await collection.add_pipeline(pipeline);
```
{% endtab %}

{% tab title="Python" %}
```python
model = Model()
splitter = Splitter()
pipeline = Pipeline("squadv1", model, splitter)  
await collection.add_pipeline(pipeline)
```
{% endtab %}
{% endtabs %}

## Upsert Documents

Text passages are upserted into the collection.

{% tabs %}
{% tab title="JavaScript" %}
```js
const documents = [
  {
    id: "...", 
    text: "...",
  }
];

await collection.upsert_documents(documents);
```
{% endtab %}

{% tab title="Python" %}
```python
data = load_dataset("squad")

documents = [
  {"id": ..., "text": ...}
  for r in data
]

await collection.upsert_documents(documents) 
```
{% endtab %}
{% endtabs %}

## Query for Context

A vector search retrieves a relevant text passage.

{% tabs %}
{% tab title="JavaScript" %}
```js
const queryResults = await collection
  .query()
  .vector_recall(query, pipeline) 
  .fetch_all();

const context = queryResults[0][1];
```
{% endtab %}

{% tab title="Python" %}
```python
results = await collection.query()
  .vector_recall(query, pipeline)
  .fetch_all()

context = results[0][1]  
```
{% endtab %}
{% endtabs %}

## Summarize Text

The text is summarized using a pretrained model.

{% tabs %}
{% tab title="JavaScript" %}
```js
const builtins = pgml.newBuiltins();

const summary = await builtins.transform(
  {task: "summarization", 
   model: "sshleifer/distilbart-cnn-12-6"},
  [context]
);
```


{% endtab %}

{% tab title="Python" %}
```python
builtins = Builtins()

summary = await builtins.transform(
  {"task": "summarization", 
   "model": "sshleifer/distilbart-cnn-12-6"},
  [context]
)
```
{% endtab %}
{% endtabs %}
