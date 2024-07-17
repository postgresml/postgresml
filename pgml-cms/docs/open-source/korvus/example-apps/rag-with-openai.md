---
description: An example application performing RAG with Korvus and OpenAI.
---

# Rag with OpenAI

This example shows how to use third-party LLM providers like OpenAI to perform RAG with Korvus.

Rag is comoposed of two parts:
- Retrieval - Search to get the context
- Augmented Generation - Perform text-generation with the LLM

Korvus can unify the retrieval and augmented generation parts into one SQL query, but if you want to use closed source models, you will have to perform retrieval and augmented generation seperately. 

!!! note

Remeber Korvus only writes SQL queries utilizing pgml to perform embeddings and text-generation in the database. The pgml extension does not support closed source models so neither does Korvus.

!!!

Even though Korvus can't use closed source models, we can use Korvus for search and use closed source models ourself.

## RAG Code

In this code block we create a Collection and a Pipeline, upsert documents into the Collection, and instead of calling the `rag` method, we call the `vector_search` method.

We take the results returned  from the `vector_search` (in this case we limited it to 1) and format a prompt for OpenAI using it.

See the [Vector Search guide](../guides/vector-search) for more information on using the `vector_search` method.

{% tabs %}
{% tab title="JavaScript" %}

```js
const korvus = require("korvus");
const openai = require("openai");

// Initialize our Collection
const collection = korvus.newCollection("openai-text-generation-demo");

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


// Initialize our client connection to OpenAI
const client = new openai.OpenAI({
  apiKey: process.env['OPENAI_API_KEY'], // This is the default and can be omitted
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
  const query = "Is Korvus fast?"
  const results = await collection.vector_search(
    {
      query: {
        fields: {
          text: {
            query: query,
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
  console.log("Our search results: ")
  console.log(results)

  // After retrieving the context, we build our prompt for gpt-4o and make our completion request
  const context = results[0].chunk
  console.log("Model output: ")
  const chatCompletion = await client.chat.completions.create({
    messages: [{ role: 'user', content: `Answer the question:\n\n${query}\n\nGiven the context:\n\n${context}` }],
    model: 'gpt-4o',
  });
  console.dir(chatCompletion, {depth: 10});
}

main().then(() => console.log("DONE!"))
```

{% endtab %}
{% tab title="Python" %}

```python
from korvus import Collection, Pipeline
from rich import print
from openai import OpenAI
import os
import asyncio

# Initialize our Collection
collection = Collection("openai-text-generation-demo")

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

# Initialize our client connection to OpenAI
client = OpenAI(
    # This is the default and can be omitted
    api_key=os.environ.get("OPENAI_API_KEY"),
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
    # Limit the results to 1. In our case we only want to feed the top result to OpenAI as we know the other result is not going to be relevant to our question
    query = "Is Korvus Fast?"
    results = await collection.vector_search(
        {
            "query": {
                "fields": {
                    "text": {
                        "query": query,
                        "parameters": {
                            "prompt": "Represent this sentence for searching relevant passages: ",
                        },
                    },
                },
            },
            "document": {"keys": ["id"]},
            "limit": 1,
        },
        pipeline,
    )
    print("Our search results: ")
    print(results)

    # After retrieving the context, we build our prompt for gpt-4o and make our completion request
    context = results[0]["chunk"]
    print("Model output: ")
    chat_completion = client.chat.completions.create(
        messages=[
            {
                "role": "user",
                "content": f"Answer the question:\n\n{query}\n\nGiven the context:\n\n{context}",
            }
        ],
        model="gpt-4o",
    )
    print(chat_completion)


asyncio.run(main())
```
{% endtab %}

{% endtabs %}

Running the example outputs:

```json
{
  id: 'chatcmpl-9kHvSowKHra1692aJsZc3G7hHMZKz',
  object: 'chat.completion',
  created: 1720819022,
  model: 'gpt-4o-2024-05-13',
  choices: [
    {
      index: 0,
      message: {
        role: 'assistant',
        content: 'Yes, Korvus is fast according to the provided context.'
      },
      logprobs: null,
      finish_reason: 'stop'
    }
  ],
  usage: { prompt_tokens: 30, completion_tokens: 12, total_tokens: 42 },
  system_fingerprint: 'fp_dd932ca5d1'
}
```

The example above shows how we can use OpenAI or any other third-party LLM to perform RAG.

A bullet point summary:
- Use Korvus to perform search
- Use the third party API provider to generate the text
