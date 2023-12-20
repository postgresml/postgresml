# Semantic Search using Instructor model

This shows using instructor models in the `pgml` SDK for more advanced use cases.

## Imports and Setup

{% tabs %}
{% tab title="JavaScript" %}
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
```
{% endtab %}
{% endtabs %}

## Initialize Collection

{% tabs %}
{% tab title="JavaScript" %}
```js
const collection = pgml.newCollection("my_javascript_qai_collection"); 
```
{% endtab %}

{% tab title="Python" %}
```python
collection = Collection("squad_collection_1")
```
{% endtab %}
{% endtabs %}

## Create Pipeline

{% tabs %}
{% tab title="JavaScript" %}
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
{% endtab %}

{% tab title="Python" %}
```python
model = Model("hkunlp/instructor-base", parameters={
    "instruction": "Represent the Wikipedia document for retrieval: "  
})

pipeline = Pipeline("squad_instruction", model, Splitter())
await collection.add_pipeline(pipeline)
```
{% endtab %}
{% endtabs %}

## Upsert Documents

{% tabs %}
{% tab title="JavaScript" %}
<pre class="language-js"><code class="lang-js">const documents = [
  {
    id: "...",
    text: "...",
  },
];

<strong>await collection.upsert_documents(documents);
</strong></code></pre>
{% endtab %}

{% tab title="Python" %}
```python
data = load_dataset("squad")

documents = [
    {"id": ..., "text": ...} for r in data
]

await collection.upsert_documents(documents) 
```
{% endtab %}
{% endtabs %}

## Query

{% tabs %}
{% tab title="JavaScript" %}
```js
const queryResults = await collection
  .query()
  .vector_recall(query, pipeline, { 
    instruction:  
      "Represent the Wikipedia question for retrieving supporting documents: ",
  })
  .fetch_all();
```
{% endtab %}

{% tab title="Python" %}
```python
results = await collection.query()
  .vector_recall(query, pipeline, {
    "instruction": "Represent the Wikipedia question for retrieving supporting documents: "
  })
  .fetch_all()
```
{% endtab %}
{% endtabs %}
