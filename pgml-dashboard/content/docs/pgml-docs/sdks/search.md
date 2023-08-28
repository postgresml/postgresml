# Search

SDK is specifically designed to provide powerful, flexible vector search. Pipelines are required to perform search. See the [pipelines.md](pipelines.md "mention") for more information about using Pipelines.

### **Basic vector search**

{% tabs %}
{% tab title="Python" %}
```python
collection = Collection("test_collection")
pipeline = Pipeline("test_pipeline")
results = await collection.query().vector_recall("Why is PostgresML the best?", pipeline).fetch_all()
```
{% endtab %}

{% tab title="JavaScript" %}
```javascript
collection = pgml.newCollection("test_collection")
pipeline = pgml.newPipeline("test_pipeline")
results = await collection.query().vector_recall("Why is PostgresML the best?", pipeline).fetch_all()
```
{% endtab %}
{% endtabs %}

### **Vector search with custom limit**

{% tabs %}
{% tab title="Python" %}
```python
collection = Collection("test_collection")
pipeline = Pipeline("test_pipeline")
results = await collection.query().vector_recall("Why is PostgresML the best?", pipeline).limit(10).fetch_all()
```
{% endtab %}

{% tab title="JavaScript" %}
```javascript
collection = pgml.newCollection("test_collection")
pipeline = pgml.newPipeline("test_pipeline")
results = await collection.query().vector_recall("Why is PostgresML the best?", pipeline).limit(10).fetch_all()
```
{% endtab %}
{% endtabs %}

### **Metadata Filtering**

We provide powerful and flexible arbitrarly nested metadata filtering based off of [MongoDB Comparison Operators](https://www.mongodb.com/docs/manual/reference/operator/query-comparison/). We support each operator mentioned except the `$nin`.

**Vector search with $eq metadata filtering**

{% tabs %}
{% tab title="Python" %}
```python
collection = Collection("test_collection")
pipeline = Pipeline("test_pipeline")
results = (
    await collection.query()
    .vector_recall("Here is some query", pipeline)
    .limit(10)
    .filter({
        "metadata": {
            "uuid": {
                "$eq": 1
            }    
        }
    })
    .fetch_all()
)
```
{% endtab %}

{% tab title="JavaScript" %}
```javascript
collection = pgml.newCollection("test_collection")
pipeline = pgml.newPipeline("test_pipeline")
results = await collection.query()
    .vector_recall("Here is some query", pipeline)
    .limit(10)
    .filter({
        "metadata": {
            "uuid": {
                "$eq": 1
            }    
        }
    })
    .fetch_all()
```
{% endtab %}
{% endtabs %}

The above query would filter out all documents that do not contain a key `uuid` equal to `1`.

**Vector search with $gte metadata filtering**

{% tabs %}
{% tab title="Python" %}
```python
collection = Collection("test_collection")
pipeline = Pipeline("test_pipeline")
results = (
    await collection.query()
    .vector_recall("Here is some query", pipeline)
    .limit(10)
    .filter({
        "metadata": {
            "index": {
                "$gte": 3
            }    
        }
    })
    .fetch_all()
)
```
{% endtab %}

{% tab title="JavaScript" %}
```javascript
collection = pgml.newCollection("test_collection")
pipeline = pgml.newPipeline("test_pipeline")
results = await collection.query()
    .vector_recall("Here is some query", pipeline)
    .limit(10)
    .filter({
        "metadata": {
            "index": {
                "$gte": 3
            }    
        }
    })
    .fetch_all()
)
```
{% endtab %}
{% endtabs %}

The above query would filter out all documents that do not contain a key `index` with a value greater than `3`.

**Vector search with $or and $and metadata filtering**

{% tabs %}
{% tab title="Python" %}
```python
collection = Collection("test_collection")
pipeline = Pipeline("test_pipeline")
results = (
    await collection.query()
    .vector_recall("Here is some query", pipeline)
    .limit(10)
    .filter({
        "metadata": {
            "$or": [
                {
                    "$and": [
                        {
                            "$eq": {
                                "uuid": 1
                            }    
                        },
                        {
                            "$lt": {
                                "index": 100 
                            }
                        }
                    ] 
                },
                {
                   "special": {
                        "$ne": True
                    } 
                }
            ]    
        }
    })
    .fetch_all()
)
```
{% endtab %}

{% tab title="JavaScript" %}
```javascript
collection = pgml.newCollection("test_collection")
pipeline = pgml.newPipeline("test_pipeline")
results = await collection.query()
    .vector_recall("Here is some query", pipeline)
    .limit(10)
    .filter({
        "metadata": {
            "$or": [
                {
                    "$and": [
                        {
                            "$eq": {
                                "uuid": 1
                            }    
                        },
                        {
                            "$lt": {
                                "index": 100 
                            }
                        }
                    ] 
                },
                {
                   "special": {
                        "$ne": True
                    } 
                }
            ]    
        }
    })
    .fetch_all()
```
{% endtab %}
{% endtabs %}

The above query would filter out all documents that do not have a key `special` with a value `True` or (have a key `uuid` equal to 1 and a key `index` less than 100).

### **Full Text Filtering**

If full text search is enabled for the associated Pipeline, documents can be first filtered by full text search and then recalled by embedding similarity.

{% tabs %}
{% tab title="Python" %}
```python
collection = Collection("test_collection")
pipeline = Pipeline("test_pipeline")
results = (
    await collection.query()
    .vector_recall("Here is some query", pipeline)
    .limit(10)
    .filter({
        "full_text": {
            "configuration": "english",
            "text": "Match Me"
        }
    })
    .fetch_all()
)
```
{% endtab %}

{% tab title="JavaScript" %}
```javascript
collection = pgml.newCollection("test_collection")
pipeline = pgml.newPipeline("test_pipeline")
results = await collection.query()
    .vector_recall("Here is some query", pipeline)
    .limit(10)
    .filter({
        "full_text": {
            "configuration": "english",
            "text": "Match Me"
        }
    })
    .fetch_all()
```
{% endtab %}
{% endtabs %}

The above query would first filter out all documents that do not match the full text search criteria, and then perform vector recall on the remaining documents.
