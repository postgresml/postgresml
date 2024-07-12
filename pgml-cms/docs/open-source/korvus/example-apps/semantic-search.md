# Semantic Search

This example demonstrates using the `korvus` SDK to create a collection, add documents, build a pipeline for vector search and make a sample query.

[Link to full JavaScript implementation](https://github.com/postgresml/korvus/blob/main/korvus/javascript/examples/semantic_search.js)

[Link to full Python implementation](https://github.com/postgresml/korvus/blob/main/korvus/python/examples/semantic_search.py)

## The Code

{% tabs %}
{% tab title="JavaScript" %}
```js
const korvus = require("korvus");

// Initialize our Collection
const collection = korvus.newCollection("semantic-search-demo");

// Initialize our Pipeline
// Our Pipeline will split and embed the `text` key of documents we upsert
const pipeline = korvus.newPipeline("v1", {
  text: {
    splitter: { model: "recursive_character" },
    semantic_search: {
      model: "mixedbread-ai/mxbai-embed-large-v1",
    }
  },
});

const main = async () => {
  // Add our Pipeline to our Collection
  await collection.add_pipeline(pipeline);

  // Upsert our documents
  // The `text` key of our documents will be split and embedded per our Pipeline specification above
  let documents = [
    {
      id: "1",
      text: "Korvus is incredibly fast and easy to use.",
    },
    {
      id: "2",
      text: "Tomatoes are incredible on burgers.",
    },
  ]
  await collection.upsert_documents(documents)

  // Perform vector_search
  // We are querying for the string "Is Korvus fast?"
  // Notice that the `mixedbread-ai/mxbai-embed-large-v1` embedding model takes a prompt paramter when embedding for search
  // We specify that we only want to return the `id` of documents. If the `document` key was blank it would return the entire document with every result
  // Limit the results to 5. In our case we only have two documents in our Collection so we will only get two results
  const results = await collection.vector_search(
    {
      query: {
        fields: {
          text: {
            query: "Is Korvus fast?",
            parameters: {
              prompt:
                "Represent this sentence for searching relevant passages: ",
            }
          },
        },
      },
      document: {
        keys: [
          "id"
        ]
      },
      limit: 5,
    },
    pipeline);
  console.log(results)
}

main().then(() => console.log("DONE!"))
```
{% endtab %}

{% tab title="Python" %}
```python
from korvus import Collection, Pipeline
from rich import print
import asyncio

# Initialize our Collection
collection = Collection("semantic-search-demo")

# Initialize our Pipeline
# Our Pipeline will split and embed the `text` key of documents we upsert
pipeline = Pipeline(
    "v1",
    {
        "text": {
            "splitter": {"model": "recursive_character"},
            "semantic_search": {
                "model": "mixedbread-ai/mxbai-embed-large-v1",
            },
        },
    },
)


async def main():
    # Add our Pipeline to our Collection
    await collection.add_pipeline(pipeline)

    # Upsert our documents
    # The `text` key of our documents will be split and embedded per our Pipeline specification above
    documents = [
        {
            "id": "1",
            "text": "Korvus is incredibly fast and easy to use.",
        },
        {
            "id": "2",
            "text": "Tomatoes are incredible on burgers.",
        },
    ]
    await collection.upsert_documents(documents)

    # Perform vector_search
    # We are querying for the string "Is Korvus fast?"
    # Notice that the `mixedbread-ai/mxbai-embed-large-v1` embedding model takes a prompt paramter when embedding for search
    # We specify that we only want to return the `id` of documents. If the `document` key was blank it would return the entire document with every result
    # Limit the results to 5. In our case we only have two documents in our Collection so we will only get two results
    results = await collection.vector_search(
        {
            "query": {
                "fields": {
                    "text": {
                        "query": "Is Korvus fast?",
                        "parameters": {
                            "prompt": "Represent this sentence for searching relevant passages: ",
                        },
                    },
                },
            },
            "document": {"keys": ["id"]},
            "limit": 5,
        },
        pipeline,
    )
    print(results)


asyncio.run(main())
```
{% endtab %}

{% endtabs %}

Running this example outputs:

```json
[
    {'chunk': 'Korvus is incredibly fast and easy to use.', 'document': {'id': '1'}, 'rerank_score': None, 'score': 0.7855310349374217},
    {'chunk': 'Tomatoes are incredible on burgers.', 'document': {'id': '2'}, 'rerank_score': None, 'score': 0.3634796874710092}
]
```

Notice how much higher the score for `Korvus is incredibly fast and easy to use.` is compared to `Tomatoes are incredible on burgers.`. This means our semantic search is working!
