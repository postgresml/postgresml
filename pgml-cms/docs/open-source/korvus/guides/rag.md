# RAG

Korvus can perform the entire RAG pipeline including embedding generation, vector search, keyword search, re-ranking and text-generation in on SQL query. 

Korvus will build a SQL query that performs search, builds the context, formats the prompt, and performs text-generation all at once. It builds on syntax already used previously in the [Vector Search guide](/docs/open-source/korvus/guides/vector-search).

`Pipeline`s are required to perform RAG. See [Pipelines ](https://postgresml.org/docs/api/client-sdk/pipelines) for more information on using `Pipeline`s.

This section will assume we have previously ran the following code:

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const collection = korvus.newCollection("test_rag_collection");
const pipeline = korvus.newPipeline("v1", {
  text: {
    splitter: { model: "recursive_character" },
    semantic_search: {
      model: "mixedbread-ai/mxbai-embed-large-v1",
    },
    full_text_search: { configuration: "english" },
  },
});
await collection.add_pipeline(pipeline);
```
{% endtab %}

{% tab title="Python" %}
```python
collection = Collection("test_rag_collection")
pipeline = Pipeline(
    "v1",
    {
        "text": {
            "splitter": {"model": "recursive_character"},
            "semantic_search": {
                "model": "mixedbread-ai/mxbai-embed-large-v1",
            },
            "full_text_search": {"configuration": "english"},
        },
    },
)
await collection.add_pipeline(pipeline);
```
{% endtab %}

{% tab title="Rust" %}
```rust
let mut collection = Collection::new("test_rag_collection", None)?;
let mut pipeline = Pipeline::new(
    "v1",
    Some(
        serde_json::json!(
            {
                "text": {
                    "splitter": {"model": "recursive_character"},
                    "semantic_search": {
                        "model": "mixedbread-ai/mxbai-embed-large-v1",
                    },
                    "full_text_search": {"configuration": "english"},
                },
            }
        )
        .into(),
    ),
)?;
collection.add_pipeline(&mut pipeline).await?;
```
{% endtab %}

{% tab title="C" %}
```cpp
CollectionC * collection = korvus_collectionc_new("test_rag_collection", NULL);
PipelineC *pipeline = korvus_pipelinec_new("v1", "{\
  \"text\": {\
      \"splitter\": {\"model\": \"recursive_character\"},\
      \"semantic_search\": {\
          \"model\": \"mixedbread-ai/mxbai-embed-large-v1\"\
      },\
      \"full_text_search\": {\"configuration\": \"english\"}\
  }\
}");
korvus_collectionc_add_pipeline(collection, pipeline);
```
{% endtab %}
{% endtabs %}

This creates a `Pipeline` that is capable of full text search and semantic search on the `text` of documents.

The RAG method will automatically perform full text and semantic search for us using the same syntax as [Vector Search](/docs/open-source/korvus/guides/vector-search).

## Simple RAG

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const results = await collection.rag(
  {
    CONTEXT: {
      vector_search: {
        query: {
          fields: {
            text: {
              query: "Is Korvus fast?",
              parameters: {
                prompt: "Represent this sentence for searching relevant passages: "
              },
            }
          },
        },
        document: { "keys": ["id"] },
        limit: 5,
      },
      aggregate: { "join": "\n" },
    },
    chat: {
      model: "meta-llama/Meta-Llama-3.1-8B-Instruct",
      messages: [
        {
          role: "system",
          content: "You are a friendly and helpful chatbot",
        },
        {
          role: "user",
          content: "Given the context\n:{CONTEXT}\nAnswer the question: Is Korvus fast?",
        },
      ],
      max_tokens: 100,
    },
  },
  pipeline,
)
```
{% endtab %}

{% tab title="Python" %}
```python
results = await collection.rag(
    {
        "CONTEXT": {
            "vector_search": {
                "query": {
                    "fields": {
                        "text": {
                            "query": "Is Korvus fast?",
                            "parameters": {
                                "prompt": "Represent this sentence for searching relevant passages: "
                            },
                        }
                    },
                },
                "document": {"keys": ["id"]},
                "limit": 5,
            },
            "aggregate": {"join": "\n"},
        },
        "chat": {
            "model": "meta-llama/Meta-Llama-3.1-8B-Instruct",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a friendly and helpful chatbot",
                },
                {
                    "role": "user",
                    "content": "Given the context\n:{CONTEXT}\nAnswer the question: Is Korvus fast?",
                },
            ],
            "max_tokens": 100,
        },
    },
    pipeline,
)
```
{% endtab %}

{% tab title="Rust" %}
```rust
let results = collection.rag(serde_json::json!(
    {
        "CONTEXT": {
            "vector_search": {
                "query": {
                    "fields": {
                        "text": {
                            "query": "Is Korvus fast?",
                            "parameters": {
                                "prompt": "Represent this sentence for searching relevant passages: "
                            },
                        }
                    },
                },
                "document": {"keys": ["id"]},
                "limit": 5,
            },
            "aggregate": {"join": "\n"},
        },
        "chat": {
            "model": "meta-llama/Meta-Llama-3.1-8B-Instruct",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a friendly and helpful chatbot",
                },
                {
                    "role": "user",
                    "content": "Given the context\n:{CONTEXT}\nAnswer the question: Is Korvus fast?",
                },
            ],
            "max_tokens": 100,
        },
    }
).into(), &mut pipeline).await?;
```
{% endtab %}

{% tab title="C" %}
```cpp
char * results = korvus_collectionc_rag(collection, 
  "{\
    \"CONTEXT\": {\
      \"vector_search\": {\
        \"query\": {\
          \"fields\": {\
            \"text\": {\
              \"query\": \"Is Korvus fast?\",\
              \"parameters\": {\
                \"prompt\": \"Represent this sentence for searching relevant passages: \"\
              }\
            }\
          }\
        },\
        \"document\": {\"keys\": [\"id\"]},\
        \"limit\": 5\
      },\
      \"aggregate\": {\"join\": \"\\n\"}\
    },\
    \"chat\": {\
      \"model\": \"meta-llama/Meta-Llama-3.1-8B-Instruct\",\
      \"messages\": [\
        {\
          \"role\": \"system\",\
          \"content\": \"You are a friendly and helpful chatbot\"\
        },\
        {\
          \"role\": \"user\",\
          \"content\": \"Given the context:\\n{CONTEXT}\\nAnswer the question: Is Korvus fast?\"\
        }\
      ],\
      \"max_tokens\": 100\
    }\
  }",
  pipeline
);
```
{% endtab %}
{% endtabs %}

Let's break this down. `rag` takes in a `JSON` object and a `Pipeline`. The `JSON` object specifies what queries to run and what prompt to pass to the model.

In the example above, we specify one vector search query that we use to build the `CONTEXT`. We then specify the `{CONTEXT}` key in the `chat.messages` which will be replaced by the results from the `CONTEXT` search.

For example if the results of the `CONTEXT` search is a list like:
```
[
    "Korvus is super fast",
    "One of the benefits of Korvus is it's speed"
]
```

Then the messages being passed to the model would look like:
```
"messages": [
    {
        "role": "system",
        "content": "You are a friendly and helpful chatbot",
    },
    {
        "role": "user",
        "content": "Given the context\n:\nKorvus is fast\nOne of the benefits of Koruvs is it's speed\nAnswer the question: Is Korvus fast?",
    },
]
```

For more information on performing vector search see the [Vector Search guide](/docs/open-source/korvus/guides/vector-search).

Note that the vector search returns 5 results. The `CONTEXT.vector_search.aggregate` key specifies how to combine these 5 results. In this situation, they are joined together with new lines seperating them.

Note that `mixedbread-ai/mxbai-embed-large-v1` takes in a prompt when creating embeddings for searching against a corpus which we provide in the `LLM_CONTEXT.vector_search.query.fields.text.parameters`.

## Hybrid Search

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const results = await collection.rag(
  {
    LLM_CONTEXT: {
      vector_search: {
        query: {
          fields: {
            text: {
              query: "Is Korvus fast?",
              parameters: {
                prompt: "Represent this sentence for searching relevant passages: "
              },
              full_text_filter: "Korvus"
            }
          },
        },
        document: { "keys": ["id"] },
        limit: 5,
      },
      aggregate: { "join": "\n" },
    },
    chat: {
      model: "meta-llama/Meta-Llama-3.1-8B-Instruct",
      messages: [
        {
          role: "system",
          content: "You are a friendly and helpful chatbot",
        },
        {
          role: "user",
          content: "Given the context\n:{LLM_CONTEXT}\nAnswer the question: Is Korvus fast?",
        },
      ],
      max_tokens: 100,
    },
  },
  pipeline,
)
```
{% endtab %}

{% tab title="Python" %}
```python
results = await collection.rag(
    {
        "LLM_CONTEXT": {
            "vector_search": {
                "query": {
                    "fields": {
                        "text": {
                            "query": "Is Korvus fast?",
                            "parameters": {
                                "prompt": "Represent this sentence for searching relevant passages: "
                            },
                            "full_text_filter": "Korvus",
                        }
                    },
                },
                "document": {"keys": ["id"]},
                "limit": 5,
            },
            "aggregate": {"join": "\n"},
        },
        "chat": {
            "model": "meta-llama/Meta-Llama-3.1-8B-Instruct",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a friendly and helpful chatbot",
                },
                {
                    "role": "user",
                    "content": "Given the context\n:{LLM_CONTEXT}\nAnswer the question: Is Korvus fast?",
                },
            ],
            "max_tokens": 100,
        },
    },
    pipeline,
)
```
{% endtab %}

{% tab title="Rust" %}
```rust
let results = collection.rag(serde_json::json!(
    {
        "LLM_CONTEXT": {
            "vector_search": {
                "query": {
                    "fields": {
                        "text": {
                            "query": "Is Korvus fast?",
                            "parameters": {
                                "prompt": "Represent this sentence for searching relevant passages: "
                            },
                            "full_text_filter": "Korvus"
                        }
                    },
                },
                "document": {"keys": ["id"]},
                "limit": 5,
            },
            "aggregate": {"join": "\n"},
        },
        "chat": {
            "model": "meta-llama/Meta-Llama-3.1-8B-Instruct",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a friendly and helpful chatbot",
                },
                {
                    "role": "user",
                    "content": "Given the context\n:{LLM_CONTEXT}\nAnswer the question: Is Korvus fast?",
                },
            ],
            "max_tokens": 100,
        },
    }
).into(), &mut pipeline).await?;
```
{% endtab %}

{% tab title="C" %}
```cpp
char * results = korvus_collectionc_rag(collection, 
  "{\
    \"LLM_CONTEXT\": {\
      \"vector_search\": {\
        \"query\": {\
          \"fields\": {\
            \"text\": {\
              \"query\": \"Is Korvus fast?\",\
              \"parameters\": {\
                \"prompt\": \"Represent this sentence for searching relevant passages: \"\
              },\
              \"full_text_filter\": \"Korvus\"\
            }\
          }\
        },\
        \"document\": {\"keys\": [\"id\"]},\
        \"limit\": 5\
      },\
      \"aggregate\": {\"join\": \"\\n\"}\
    },\
    \"chat\": {\
      \"model\": \"meta-llama/Meta-Llama-3-8B-Instruct\",\
      \"messages\": [\
        {\
          \"role\": \"system\",\
          \"content\": \"You are a friendly and helpful chatbot\"\
        },\
        {\
          \"role\": \"user\",\
          \"content\": \"Given the context:\\n{LLM_CONTEXT}\\nAnswer the question: Is Korvus fast?\"\
        }\
      ],\
      \"max_tokens\": 100\
    }\
  }",
  pipeline
);
```
{% endtab %}
{% endtabs %}

This is very similar to the example above but note that we renamed `CONTEXT` to `LLM_CONTEXT` this changes nothing. We could call it whatever we want.

The main difference is that we have included the `full_text_filter` key in the `LLM_CONTEXT.vector_search.query.fields.text` object. This restricts us from retrieving chunks that do not contain the string `Korvus`. This utilizes Postgre's full text filter mechanics. For more information see the guide on performing vector search.

## Re-ranking Search Results

Before we pass the results of our `LLM_CONTEXT` to the LLM, we can rerank them:

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const results = await collection.rag(
  {
    LLM_CONTEXT: {
      vector_search: {
        query: {
          fields: {
            text: {
              query: "Is Korvus fast?",
              parameters: {
                prompt: "Represent this sentence for searching relevant passages: "
              },
              full_text_filter: "Korvus"
            }
          },
        },
        document: { "keys": ["id"] },
        rerank: {
            model: "mixedbread-ai/mxbai-rerank-base-v1",
            query: "Is Korvus fast?",
            num_documents_to_rerank: 100
        },
        limit: 5,
      },
      aggregate: { "join": "\n" },
    },
    chat: {
      model: "meta-llama/Meta-Llama-3-8B-Instruct",
      messages: [
        {
          role: "system",
          content: "You are a friendly and helpful chatbot",
        },
        {
          role: "user",
          content: "Given the context\n:{LLM_CONTEXT}\nAnswer the question: Is Korvus fast?",
        },
      ],
      max_tokens: 100,
    },
  },
  pipeline,
)
```
{% endtab %}

{% tab title="Python" %}
```python
results = await collection.rag(
    {
        "LLM_CONTEXT": {
            "vector_search": {
                "query": {
                    "fields": {
                        "text": {
                            "query": "Is Korvus fast?",
                            "parameters": {
                                "prompt": "Represent this sentence for searching relevant passages: "
                            },
                            "full_text_filter": "Korvus",
                        }
                    },
                },
                "document": {"keys": ["id"]},
                "rerank": {
                    "model": "mixedbread-ai/mxbai-rerank-base-v1",
                    "query": "Is Korvus fast?",
                    "num_documents_to_rerank": 100,
                },
                "limit": 5,
            },
            "aggregate": {"join": "\n"},
        },
        "chat": {
            "model": "meta-llama/Meta-Llama-3-8B-Instruct",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a friendly and helpful chatbot",
                },
                {
                    "role": "user",
                    "content": "Given the context\n:{LLM_CONTEXT}\nAnswer the question: Is Korvus fast?",
                },
            ],
            "max_tokens": 100,
        },
    },
    pipeline,
)
```
{% endtab %}

{% tab title="Rust" %}
```rust
let results = collection.rag(serde_json::json!(
    {
        "LLM_CONTEXT": {
            "vector_search": {
                "query": {
                    "fields": {
                        "text": {
                            "query": "Is Korvus fast?",
                            "parameters": {
                                "prompt": "Represent this sentence for searching relevant passages: "
                            },
                            "full_text_filter": "Korvus"
                        }
                    },
                },
                "document": {"keys": ["id"]},
                "rerank": {
                    "model": "mixedbread-ai/mxbai-rerank-base-v1",
                    "query": "Is Korvus fast?",
                    "num_documents_to_rerank": 100
                },
                "limit": 5,
            },
            "aggregate": {"join": "\n"},
        },
        "chat": {
            "model": "meta-llama/Meta-Llama-3-8B-Instruct",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a friendly and helpful chatbot",
                },
                {
                    "role": "user",
                    "content": "Given the context\n:{LLM_CONTEXT}\nAnswer the question: Is Korvus fast?",
                },
            ],
            "max_tokens": 100,
        },
    }
).into(), &mut pipeline).await?;
```
{% endtab %}

{% tab title="C" %}
```cpp
char * results = korvus_collectionc_rag(collection,
  "{\
    \"LLM_CONTEXT\": {\
      \"vector_search\": {\
        \"query\": {\
          \"fields\": {\
            \"text\": {\
              \"query\": \"Is Korvus fast?\",\
              \"parameters\": {\
                \"prompt\": \"Represent this sentence for searching relevant passages: \"\
              },\
              \"full_text_filter\": \"Korvus\"\
            }\
          }\
        },\
        \"document\": {\"keys\": [\"id\"]},\
            \"rerank\": {\
            \"model\": \"mixedbread-ai/mxbai-rerank-base-v1\",\
            \"query\": \"Is Korvus fast?\",\
            \"num_documents_to_rerank\": 100\
        },\
        \"limit\": 5\
      },\
      \"aggregate\": {\"join\": \"\\n\"}\
    },\
    \"chat\": {\
      \"model\": \"meta-llama/Meta-Llama-3-8B-Instruct\",\
      \"messages\": [\
        {\
          \"role\": \"system\",\
          \"content\": \"You are a friendly and helpful chatbot\"\
        },\
        {\
          \"role\": \"user\",\
          \"content\": \"Given the context:\\n{LLM_CONTEXT}\\nAnswer the question: Is Korvus fast?\"\
        }\
      ],\
      \"max_tokens\": 100\
    }\
  }",
  pipeline
);
```
{% endtab %}
{% endtabs %}

This utilizes the re-ranking capabilities found in the `vector_search` method. For more information check out our guides on [Re-ranking](/docs/open-source/korvus/guides/vector-search#re-ranking) and [Vector Search](/docs/open-source/korvus/guides/vector-search).

## Raw SQL queries / Multi-variable Context

So far we have only used the `CONTEXT` or `LLM_CONTEXT` variables individually for vector search, but we can combine them together or specify a RAW sql query.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const results = await collection.rag(
  {
    LLM_CONTEXT: {
      vector_search: {
        query: {
          fields: {
            text: {
              query: "Is Korvus fast?",
              parameters: {
                prompt: "Represent this sentence for searching relevant passages: "
              },
              full_text_filter: "Korvus"
            }
          },
        },
        document: { "keys": ["id"] },
        rerank: {
            model: "mixedbread-ai/mxbai-rerank-base-v1",
            query: "Is Korvus fast?",
            num_documents_to_rerank: 100
        },
        limit: 5,
      },
      aggregate: { "join": "\n" },
    },
    CUSTOM_CONTEXT: {sql: "SELECT 'Korvus is super fast!!!'"},
    chat: {
      model: "meta-llama/Meta-Llama-3-8B-Instruct",
      messages: [
        {
          role: "system",
          content: "You are a friendly and helpful chatbot",
        },
        {
          role: "user",
          content: "Given the context\n:{LLM_CONTEXT}\n{CUSTOM_CONTEXT}\nAnswer the question: Is Korvus fast?",
        },
      ],
      max_tokens: 100,
    },
  },
  pipeline,
)
```
{% endtab %}

{% tab title="Python" %}
```python
results = await collection.rag(
    {
        "LLM_CONTEXT": {
            "vector_search": {
                "query": {
                    "fields": {
                        "text": {
                            "query": "Is Korvus fast?",
                            "parameters": {
                                "prompt": "Represent this sentence for searching relevant passages: "
                            },
                            "full_text_filter": "Korvus",
                        }
                    },
                },
                "document": {"keys": ["id"]},
                "rerank": {
                    "model": "mixedbread-ai/mxbai-rerank-base-v1",
                    "query": "Is Korvus fast?",
                    "num_documents_to_rerank": 100,
                },
                "limit": 5,
            },
            "aggregate": {"join": "\n"},
        },
        "CUSTOM_CONTEXT": {"sql": "SELECT 'Korvus is super fast!!!'"},
        "chat": {
            "model": "meta-llama/Meta-Llama-3-8B-Instruct",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a friendly and helpful chatbot",
                },
                {
                    "role": "user",
                    "content": "Given the context\n:{LLM_CONTEXT}\n{CUSTOM_CONTEXT}\nAnswer the question: Is Korvus fast?",
                },
            ],
            "max_tokens": 100,
        },
    },
    pipeline,
)
```
{% endtab %}

{% tab title="Rust" %}
```rust
let results = collection.rag(serde_json::json!(
    {
        "LLM_CONTEXT": {
            "vector_search": {
                "query": {
                    "fields": {
                        "text": {
                            "query": "Is Korvus fast?",
                            "parameters": {
                                "prompt": "Represent this sentence for searching relevant passages: "
                            },
                            "full_text_filter": "Korvus"
                        }
                    },
                },
                "document": {"keys": ["id"]},
                "rerank": {
                    "model": "mixedbread-ai/mxbai-rerank-base-v1",
                    "query": "Is Korvus fast?",
                    "num_documents_to_rerank": 100,
                },
                "limit": 1,
            },
            "aggregate": {"join": "\n"},
        },
        "CUSTOM_CONTEXT": {"sql": "SELECT 'Korvus is super fast!!!'"},
        "chat": {
            "model": "meta-llama/Meta-Llama-3-8B-Instruct",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a friendly and helpful chatbot",
                },
                {
                    "role": "user",
                    "content": "Given the context\n:{LLM_CONTEXT}\n{CUSTOM_CONTEXT}\nAnswer the question: Is Korvus fast?",
                },
            ],
            "max_tokens": 100,
        },
    }
).into(), &mut pipeline).await?;
```
{% endtab %}

{% tab title="C" %}
```cpp
char * results = korvus_collectionc_rag(collection,
  "{\
    \"LLM_CONTEXT\": {\
      \"vector_search\": {\
        \"query\": {\
          \"fields\": {\
            \"text\": {\
              \"query\": \"Is Korvus fast?\",\
              \"parameters\": {\
                \"prompt\": \"Represent this sentence for searching relevant passages: \"\
              },\
              \"full_text_filter\": \"Korvus\"\
            }\
          }\
        },\
        \"document\": {\"keys\": [\"id\"]},\
            \"rerank\": {\
            \"model\": \"mixedbread-ai/mxbai-rerank-base-v1\",\
            \"query\": \"Is Korvus fast?\",\
            \"num_documents_to_rerank\": 100\
        },\
        \"limit\": 1\
      },\
      \"aggregate\": {\"join\": \"\\n\"}\
    },\
    \"CUSTOM_CONTEXT\": {\"sql\": \"SELECT 'Korvus is super fast!!!'\"},\
    \"chat\": {\
      \"model\": \"meta-llama/Meta-Llama-3-8B-Instruct\",\
      \"messages\": [\
        {\
          \"role\": \"system\",\
          \"content\": \"You are a friendly and helpful chatbot\"\
        },\
        {\
          \"role\": \"user\",\
          \"content\": \"Given the context:\\n{LLM_CONTEXT}\\n\\n{CUSTOM_CONTEXT}\\nAnswer the question: Is Korvus fast?\"\
        }\
      ],\
      \"max_tokens\": 100\
    }\
  }",
  pipeline
);
```
{% endtab %}
{% endtabs %}

By specifying the `sql` key instead of `vector_search` in `CUSTOM_CONTEXT` we are performing a raw SQL query. In this case we are selecting the text `Korvus is super fast!!!` but you can perform any sql query that returns a string.

Just like the `LLM_CONTEXT` key, the result of the `CUSTOM_CONTEXT`query will replace the `{CUSTOM_CONTEXT}` placeholder in the `messages`.
