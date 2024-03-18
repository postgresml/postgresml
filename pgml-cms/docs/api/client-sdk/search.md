# Vector Search

SDK is specifically designed to provide powerful, flexible vector search. `Pipeline`s are required to perform search. See [Pipelines ](https://postgresml.org/docs/introduction/apis/client-sdks/pipelines)for more information about using `Pipeline`s.

This section will assume we have previously ran the following code:

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const pipeline = pgml.newPipeline("test_pipeline", {
  abstract: {
    semantic_search: {
      model: "intfloat/e5-small",
    },
    full_text_search: { configuration: "english" },
  },
  body: {
    splitter: { model: "recursive_character" },
    semantic_search: {
      model: "hkunlp/instructor-base",
      parameters: {
        instruction: "Represent the Wikipedia document for retrieval: ",
      }
    },
  },
});
const collection = pgml.newCollection("test_collection");
await collection.add_pipeline(pipeline);
```
{% endtab %}

{% tab title="Python" %}
```python
pipeline = Pipeline(
    "test_pipeline",
    {
        "abstract": {
            "semantic_search": {
                "model": "intfloat/e5-small",
            },
            "full_text_search": {"configuration": "english"},
        },
        "body": {
            "splitter": {"model": "recursive_character"},
            "semantic_search": {
                "model": "hkunlp/instructor-base",
                "parameters": {
                    "instruction": "Represent the Wikipedia document for retrieval: ",
                },
            },
        },
    },
)
collection = Collection("test_collection")
```
{% endtab %}
{% endtabs %}

This creates a `Pipeline` that is capable of full text search and semantic search on the `abstract` and semantic search on the `body` of documents.

## **Doing vector search**

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const results = await collection.vector_search(
  {
    query: {
      fields: {
        body: {
          query: "What is the best database?", parameters: {
            instruction:
              "Represent the Wikipedia question for retrieving supporting documents: ",
          }
        },
      },
    },
    limit: 5,
  },
  pipeline,
);
```
{% endtab %}

{% tab title="Python" %}
```python
results = await collection.vector_search(
    {
        "query": {
            "fields": {
                "body": {
                    "query": "What is the best database?",
                    "parameters": {
                        "instruction": "Represent the Wikipedia question for retrieving supporting documents: ",
                    },
                },
            },
        },
        "limit": 5,
    },
    pipeline,
)
```
{% endtab %}
{% endtabs %}

Let's break this down. `vector_search` takes in a `JSON` object and a `Pipeline`. The `JSON` object currently supports two keys: `query` and `limit` . The `limit` limits how many chunks should be returned, the `query` specifies the actual query to perform. Let's see another more complicated example:

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const query = "What is the best database?";
const results = await collection.vector_search(
  {
    query: {
      fields: {
        abstract: {
          query: query,
          full_text_filter: "database"
        },
        body: {
          query: query, parameters: {
            instruction:
              "Represent the Wikipedia question for retrieving supporting documents: ",
          }
        },
      },
    },
    limit: 5,
  },
  pipeline,
);
```
{% endtab %}

{% tab title="Python" %}
```python
query = "What is the best database?"
results = await collection.vector_search(
    {
        "query": {
            "fields": {
                "abastract": {
                    "query": query,
                    "full_text_filter": "database",
                },
                "body": {
                    "query": query,
                    "parameters": {
                        "instruction": "Represent the Wikipedia question for retrieving supporting documents: ",
                    },
                },
            },
        },
        "limit": 5,
    },
    pipeline,
)

```
{% endtab %}
{% endtabs %}

The `query` in this example is slightly more intricate. We are doing vector search over both the `abstract` and `body` keys of our documents. This means our search may return chunks from both the `abstract` and `body` of our documents.  We are also filtering out all `abstract` chunks that do not contain the text `"database"` we can do this because we enabled `full_text_search` on the `abstract` key in the `Pipeline` schema. Also note that the model used for embedding the `body` takes parameters, but not the model used for embedding the `abstract`.

## **Filtering**

We provide powerful and flexible arbitrarly nested filtering based off of [MongoDB Comparison Operators](https://www.mongodb.com/docs/manual/reference/operator/query-comparison/). We support each operator mentioned except the `$nin`.

**Vector search with $eq filtering**

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const results = await collection.vector_search(
  {
    query: {
      fields: {
        body: {
          query: "What is the best database?", parameters: {
            instruction:
              "Represent the Wikipedia question for retrieving supporting documents: ",
          }
        },
      },
      filter: {
        user_id: {
          $eq: 1
        }
      }
    },
    limit: 5,
  },
  pipeline,
);
```
{% endtab %}

{% tab title="Python" %}
```python
results = await collection.vector_search(
    {
        "query": {
            "fields": {
                "body": {
                    "query": "What is the best database?",
                    "parameters": {
                        "instruction": "Represent the Wikipedia question for retrieving supporting documents: ",
                    },
                },
            },
            "filter": {"user_id": {"$eq": 1}},
        },
        "limit": 5,
    },
    pipeline,
)
```
{% endtab %}
{% endtabs %}

The above query would filter out all chunks from documents that do not contain a key `user_id` equal to `1`.

**Vector search with $gte filtering**

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const results = await collection.vector_search(
  {
    query: {
      fields: {
        body: {
          query: "What is the best database?", parameters: {
            instruction:
              "Represent the Wikipedia question for retrieving supporting documents: ",
          }
        },
      },
      filter: {
        user_id: {
          $gte: 1
        }
      }
    },
    limit: 5,
  },
  pipeline,
);
```
{% endtab %}

{% tab title="Python" %}
```python
results = await collection.vector_search(
    {
        "query": {
            "fields": {
                "body": {
                    "query": "What is the best database?",
                    "parameters": {
                        "instruction": "Represent the Wikipedia question for retrieving supporting documents: ",
                    },
                },
            },
            "filter": {"user_id": {"$gte": 1}},
        },
        "limit": 5,
    },
    pipeline,
)
```
{% endtab %}
{% endtabs %}

The above query would filter out all documents that do not contain a key `user_id` with a value greater than or equal to `1`.

**Vector search with $or and $and filtering**

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const results = await collection.vector_search(
  {
    query: {
      fields: {
        body: {
          query: "What is the best database?", parameters: {
            instruction:
              "Represent the Wikipedia question for retrieving supporting documents: ",
          }
        },
      },
      filter: {
        $or: [
          {
            $and: [
              {
                $eq: {
                  user_id: 1
                }
              },
              {
                $lt: {
                  user_score: 100
                }
              }
            ]
          },
          {
            special: {
              $ne: true
            }
          }
        ]
      }
    },
    limit: 5,
  },
  pipeline,
);
```
{% endtab %}

{% tab title="Python" %}
```python
results = await collection.vector_search(
    {
        "query": {
            "fields": {
                "body": {
                    "query": "What is the best database?",
                    "parameters": {
                        "instruction": "Represent the Wikipedia question for retrieving supporting documents: ",
                    },
                },
            },
            "filter": {
                "$or": [
                    {"$and": [{"$eq": {"user_id": 1}}, {"$lt": {"user_score": 100}}]},
                    {"special": {"$ne": True}},
                ],
            },
        },
        "limit": 5,
    },
    pipeline,
)
```
{% endtab %}
{% endtabs %}

The above query would filter out all documents that do not have a key `special` with a value `True` or (have a key `user_id` equal to 1 and a key `user_score` less than 100).
