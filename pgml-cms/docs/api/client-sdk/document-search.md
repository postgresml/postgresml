# Document Search

SDK is specifically designed to provide powerful, flexible document search. `Pipeline`s are required to perform search. See the [Pipelines](https://postgresml.org/docs/api/client-sdk/pipelines) for more information about using `Pipeline`s.

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

## Doing Document Search

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const results = await collection.search(
  {
    query: {
      full_text_search: { abstract: { query: "What is the best database?", boost: 1.2 } },
      semantic_search: {
        abstract: {
          query: "What is the best database?", boost: 2.0,
        },
        body: {
          query: "What is the best database?", boost: 1.25, parameters: {
            instruction:
              "Represent the Wikipedia question for retrieving supporting documents: ",
          }
        },
      },
      filter: { user_id: { $eq: 1 } },
    },
    limit: 10
  },
  pipeline,
);
```
{% endtab %}

{% tab title="Python" %}
```python
results = await collection.search(
    {
        "query": {
            "full_text_search": {
                "abstract": {"query": "What is the best database?", "boost": 1.2}
            },
            "semantic_search": {
                "abstract": {
                    "query": "What is the best database?",
                    "boost": 2.0,
                },
                "body": {
                    "query": "What is the best database?",
                    "boost": 1.25,
                    "parameters": {
                        "instruction": "Represent the Wikipedia question for retrieving supporting documents: ",
                    },
                },
            },
            "filter": {"user_id": {"$eq": 1}},
        },
        "limit": 10,
    },
    pipeline,
)
```
{% endtab %}
{% endtabs %}

Just like `vector_search`, `search` takes in two arguments. The first is a `JSON` object specifying the `query` and `limit` and the second is the `Pipeline`. The `query` object can have three fields: `full_text_search`, `semantic_search` and `filter`. Both `full_text_search` and `semantic_search` function similarly. They take in the text to compare against, titled`query`, an optional `boost` parameter used to boost the effectiveness of the ranking, and `semantic_search` also takes in an optional `parameters` key which specify parameters to pass to the embedding model when embedding the passed in text.

Lets break this query down a little bit more. We are asking for a maximum of 10 documents ranked by `full_text_search` on the `abstract` and `semantic_search` on the `abstract` and `body`. We are also filtering out all documents that do not have the key `user_id` equal to `1`.  The `full_text_search` provides a score for the `abstract`, and `semantic_search` provides scores for the `abstract` and the `body`. The `boost` parameter is a multiplier applied to these scores before they are summed together and sorted by `score` `DESC`.

The `filter` is structured the same way it is when performing `vector_search` see [filtering with vector\_search](https://postgresml.org/docs/api/client-sdk/search)[ ](https://postgresml.org/docs/api/client-sdk/search#metadata-filtering)for more examples on filtering documents.

## Fine-Tuning Document Search

More information and examples on this coming soon...
