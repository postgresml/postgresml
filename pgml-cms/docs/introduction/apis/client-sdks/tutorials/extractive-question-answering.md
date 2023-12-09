# Extractive Question Answering

Here is the documentation for the JavaScript and Python code snippets performing end-to-end question answering:

## Imports and Setup

The SDK and datasets are imported. Builtins are used in Python for transforming text.

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

A collection is created to hold context passages.

{% tabs %}
{% tab title="JavaScript" %}
```js
const collection = pgml.newCollection("my_javascript_eqa_collection");
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
  "my_javascript_eqa_pipeline",
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

Context passages from SQuAD are upserted into the collection.

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

A vector search query retrieves context passages.

{% tabs %}
{% tab title="JavaScript" %}
```js
const queryResults = await collection
  .query()
  .vector_recall(query, pipeline)
  .fetch_all();

const context = queryResults
  .map(result => result[1])
  .join("\n");
```
{% endtab %}

{% tab title="Python" %}
```python
results = await collection.query()
  .vector_recall(query, pipeline) 
  .fetch_all()

context = " ".join(results[0][1]) 
```
{% endtab %}
{% endtabs %}

## Query for Answer

The context is passed to a QA model to extract the answer.

{% tabs %}
{% tab title="JavaScript" %}
```js
const builtins = pgml.newBuiltins();

const answer = await builtins.transform("question-answering", [
  JSON.stringify({question, context})
]);
```
{% endtab %}

{% tab title="Python" %}
```python
builtins = Builtins()

answer = await builtins.transform(
  "question-answering", 
  [{"question": query, "context": context}]
)
```
{% endtab %}
{% endtabs %}
