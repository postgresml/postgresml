# Document Search

Korvus is specifically designed to provide powerful, flexible document search. `Pipeline`s are required to perform search. See the [Pipelines](docs/api/client-sdk/pipelines) for more information about using `Pipeline`s.

This section will assume we have previously ran the following code:

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const pipeline = korvus.newPipeline("test_pipeline", {
  abstract: {
    semantic_search: {
      model: "mixedbread-ai/mxbai-embed-large-v1",
    },
    full_text_search: { configuration: "english" },
  },
  body: {
    splitter: { model: "recursive_character" },
    semantic_search: {
      model: "Alibaba-NLP/gte-base-en-v1.5",
    },
  },
});
const collection = korvus.newCollection("test_collection");
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
                "model": "mixedbread-ai/mxbai-embed-large-v1",
            },
            "full_text_search": {"configuration": "english"},
        },
        "body": {
            "splitter": {"model": "recursive_character"},
            "semantic_search": {
                "model": "Alibaba-NLP/gte-base-en-v1.5",
            },
        },
    },
)
collection = Collection("test_collection")
```
{% endtab %}

{% tab title="Rust" %}
```rust
let mut pipeline = Pipeline::new(
    "test_pipeline",
    Some(
        serde_json::json!(
            {
                "abstract": {
                    "semantic_search": {
                        "model": "mixedbread-ai/mxbai-embed-large-v1",
                    },
                    "full_text_search": {"configuration": "english"},
                },
                "body": {
                    "splitter": {"model": "recursive_character"},
                    "semantic_search": {
                        "model": "Alibaba-NLP/gte-base-en-v1.5",
                    },
                },
            }
        )
        .into(),
    ),
)?;
let mut collection = Collection::new("test_collection", None)?;
collection.add_pipeline(&mut pipeline).await?;
```
{% endtab %}

{% tab title="C" %}
```cpp
PipelineC *pipeline = korvus_pipelinec_new("test_pipeline", "{\
    \"abstract\": {\
        \"semantic_search\": {\
            \"model\": \"mixedbread-ai/mxbai-embed-large-v1\"\
        },\
        \"full_text_search\": {\"configuration\": \"english\"}\
    },\
    \"body\": {\
        \"splitter\": {\"model\": \"recursive_character\"},\
        \"semantic_search\": {\
            \"model\": \"Alibaba-NLP/gte-base-en-v1.5\"\
        }\
    }\
}");
CollectionC * collection = korvus_collectionc_new("test_collection", NULL);
korvus_collectionc_add_pipeline(collection, pipeline);
```
{% endtab %}
{% endtabs %}

This creates a `Pipeline` that is capable of full text search and semantic search on the `abstract` and semantic search on the `body` of documents.

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
            prompt:
              "Represent this sentence for searching relevant passages: ",
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
                        "prompt": "Represent this sentence for searching relevant passages: ",
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


{% tab title="Rust" %}
```rust
let results = collection
    .search(serde_json::json!({
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
                        "prompt": "Represent this sentence for searching relevant passages: ",
                    },
                },
            },
            "filter": {"user_id": {"$eq": 1}},
        },
        "limit": 10,
    }).into(), &mut pipeline)
    .await?;
```
{% endtab %}

{% tab title="C" %}
```cpp
char * results = korvus_collectionc_search(collection, "\
     \"query\": {\
        \"full_text_search\": {\
            \"abstract\": {\"query\": \"What is the best database?\", \"boost\": 1.2}\
        },\
        \"semantic_search\": {\
            \"abstract\": {\
                \"query\": \"What is the best database?\",\
                \"boost\": 2.0\
            },\
            \"body\": {\
                \"query\": \"What is the best database?\",\
                \"boost\": 1.25,\
                \"parameters\": {\
                    \"prompt\": \"Represent this sentence for searching relevant passages: \"\
                }\
            }\
        },\
        \"filter\": {\"user_id\": {\"$eq\": 1}}\
    },\
    \"limit\": 10\
", pipeline);
```
{% endtab %}
{% endtabs %}

Just like `vector_search`, `search` takes in two arguments. The first is a `JSON` object specifying the `query` and `limit` and the second is the `Pipeline`. 

The `query` object can have three fields: 

- `full_text_search`
- `semantic_search`
- `filter` 

Both `full_text_search` and `semantic_search` function similarly. They take in the text to compare against, titled `query`, an optional `boost` parameter used to boost the effectiveness of the ranking, and `semantic_search` also takes in an optional `parameters` key which specify parameters to pass to the embedding model when embedding the passed in text.

The `filter` is structured the same way it is when performing `vector_search` see [filtering with vector_search](/docs/open-source/korvus/guides/vector-search#filtering) for more examples on filtering documents.

Lets break this query down a little bit more. We are asking for a maximum of 10 documents ranked by `full_text_search` on the `abstract` and `semantic_search` on the `abstract` and `body`. We are also filtering out all documents that do not have the key `user_id` equal to `1`.  The `full_text_search` provides a score for the `abstract`, and `semantic_search` provides scores for the `abstract` and the `body`. The `boost` parameter is a multiplier applied to these scores before they are summed together and sorted by `score` `DESC`.


## Fine-Tuning Document Search

More information and examples on this coming soon...
