# Pipelines

Pipelines are composed of a Model, Splitter, and additional optional arguments. Collections can have any number of Pipelines. Each Pipeline is ran everytime documents are upserted.

## Models

Models are used for embedding chuncked documents. We support most every open source model on [Hugging Face](https://huggingface.co/), and also OpenAI's embedding models.

### **Create a default Model "intfloat/e5-small" with default parameters: {}**

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const model = pgml.newModel()
```
{% endtab %}

{% tab title="Python" %}
```python
model = Model()
```
{% endtab %}
{% endtabs %}

### **Create a Model with custom parameters**

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const model = pgml.newModel(
  "hkunlp/instructor-base",
  "pgml",
  { instruction: "Represent the Wikipedia document for retrieval: " }
)
```
{% endtab %}

{% tab title="Python" %}
```python
model = Model(
    name="hkunlp/instructor-base",
    parameters={"instruction": "Represent the Wikipedia document for retrieval: "}    
)
```
{% endtab %}
{% endtabs %}

### **Use an OpenAI model**

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const model = pgml.newModel("text-embedding-ada-002", "openai")
```
{% endtab %}

{% tab title="Python" %}
```python
model = Model(name="text-embedding-ada-002", source="openai")
```
{% endtab %}
{% endtabs %}

## Splitters

Splitters are used to split documents into chunks before embedding them. We support splitters found in [LangChain](https://www.langchain.com/).

### **Create a default Splitter "recursive\_character" with default parameters: {}**

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const splitter = pgml.newSplitter()
```
{% endtab %}

{% tab title="Python" %}
```python
splitter = Splitter()
```
{% endtab %}
{% endtabs %}

### **Create a Splitter with custom parameters**

{% tabs %}
{% tab title="JavaScript" %}
```javascript
splitter = pgml.newSplitter(
  "recursive_character",
  { chunk_size: 1500, chunk_overlap: 40 }
)
```
{% endtab %}

{% tab title="Python" %}
```python
splitter = Splitter(
    name="recursive_character", 
    parameters={"chunk_size": 1500, "chunk_overlap": 40}
)
```
{% endtab %}
{% endtabs %}

## Adding Pipelines to a Collection

When adding a Pipeline to a collection it is required that Pipeline has a Model and Splitter.

The first time a Pipeline is added to a Collection it will automatically chunk and embed any documents already in that Collection.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const model = pgml.newModel()
const splitter = pgml.newSplitter()
const pipeline = pgml.newPipeline("test_pipeline", model, splitter)
await collection.add_pipeline(pipeline)
```
{% endtab %}

{% tab title="Python" %}
```python
model = Model()
splitter = Splitter()
pipeline = Pipeline("test_pipeline", model, splitter)
await collection.add_pipeline(pipeline)
```
{% endtab %}
{% endtabs %}

### Enabling full text search

Pipelines can take additional arguments enabling full text search. When full text search is enabled, in addition to automatically chunking and embedding, the Pipeline will create the necessary tsvectors to perform full text search.

For more information on full text search please see: [Postgres Full Text Search](https://www.postgresql.org/docs/15/textsearch.html).

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const model = pgml.newModel()
const splitter = pgml.newSplitter()
const pipeline = pgml.newPipeline("test_pipeline", model, splitter, {
  full_text_search: {
    active: true,
    configuration: "english"
  }
})
await collection.add_pipeline(pipeline)
```
{% endtab %}

{% tab title="Python" %}
```python
model = Model()
splitter = Splitter()
pipeline = Pipeline("test_pipeline", model, splitter, {
    "full_text_search": {
        "active": True,
        "configuration": "english"
    }
})
await collection.add_pipeline(pipeline)
```
{% endtab %}
{% endtabs %}

### Customizing the HNSW Index

By default the SDK uses HNSW indexes to efficiently perform vector recall. The default HNSW index sets `m` to 16 and `ef_construction` to 64. These defaults can be customized when the Pipeline is created.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const model = pgml.newModel()
const splitter = pgml.newSplitter()
const pipeline = pgml.newPipeline("test_pipeline", model, splitter, {
  hnsw: {
    m: 16,
    ef_construction: 64
  }
})
await collection.add_pipeline(pipeline)
```
{% endtab %}

{% tab title="Python" %}
```python
model = Model()
splitter = Splitter()
pipeline = Pipeline("test_pipeline", model, splitter, {
    "hnsw": {
        "m": 16,
        "ef_construction": 64
    }
})
await collection.add_pipeline(pipeline)
```
{% endtab %}
{% endtabs %}

## Searching with Pipelines

Pipelines are a required argument when performing vector search. After a Pipeline has been added to a Collection, the Model and Splitter can be omitted when instantiating it.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const pipeline = pgml.newPipeline("test_pipeline")
const collection = pgml.newCollection("test_collection")
const results = await collection.query().vector_recall("Why is PostgresML the best?", pipeline).fetch_all()    
```
{% endtab %}

{% tab title="Python" %}
```python
pipeline = Pipeline("test_pipeline")
collection = Collection("test_collection")
results = await collection.query().vector_recall("Why is PostgresML the best?", pipeline).fetch_all()    
```
{% endtab %}
{% endtabs %}

## **Disable a Pipeline**

Pipelines can be disabled or removed to prevent them from running automatically when documents are upserted.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const pipeline = pgml.newPipeline("test_pipeline")
const collection = pgml.newCollection("test_collection")
await collection.disable_pipeline(pipeline)
```
{% endtab %}

{% tab title="Python" %}
```python
pipeline = Pipeline("test_pipeline")
collection = Collection("test_collection")
await collection.disable_pipeline(pipeline)
```
{% endtab %}
{% endtabs %}

Disabling a Pipeline prevents it from running automatically, but leaves all chunks and embeddings already created by that Pipeline in the database.

## **Enable a Pipeline**

Disabled pipelines can be re-enabled.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const pipeline = pgml.newPipeline("test_pipeline")
const collection = pgml.newCollection("test_collection")
await collection.enable_pipeline(pipeline)
```
{% endtab %}

{% tab title="Python" %}
```python
pipeline = Pipeline("test_pipeline")
collection = Collection("test_collection")
await collection.enable_pipeline(pipeline)
```
{% endtab %}
{% endtabs %}

Enabling a Pipeline will cause it to automatically run and chunk and embed all documents it may have missed while disabled.

## **Remove a Pipeline**

{% tabs %}
{% tab title="JavaScript" %}
<pre class="language-javascript"><code class="lang-javascript">const pipeline = pgml.newPipeline("test_pipeline")
<strong>const collection = pgml.newCollection("test_collection")
</strong>await collection.remove_pipeline(pipeline)
</code></pre>
{% endtab %}

{% tab title="Python" %}
```python
pipeline = Pipeline("test_pipeline")
collection = Collection("test_collection")
await collection.remove_pipeline(pipeline)
```
{% endtab %}
{% endtabs %}

Removing a Pipeline deletes it and all associated data from the database. Removed Pipelines cannot be re-enabled but can be recreated.
