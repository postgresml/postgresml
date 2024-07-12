# Rag with OpenAI

This example shows how to use third party model provides like OpenAI to perform RAG with Korvus.

Rag is comoposed of two parts:
- Retrieval - Search to get the context
- Augmented Generation - Perform text-generation with the LLM

Korvus can unify the retrieval and augmented generation parts into one SQL query, but if you want to use closed source models, you will have to perform retrieval and augemented generation seperately. 

!!! note

Remeber Korvus only writes SQL queries utilizing pgml to perform embeddings and text-generation in the database. The pgml extension does not support closed source models so neither does Korvus

!!!

Even though we can't used cloused source models with Korvus we can still perform the retrieval part of RAG with Korvus.

## Code

In this code block we create a Collection and a Pipeline, upsert documents into the Collection, but instead of calling the `rag` method, we call the `vector_search` method.

We take the results returned  from the `vector_search` (in this case we limited it to 1) and format a prompt for OpenAI using it. 

{% tabs %}
{% tab title="JavaScript" %}

```js
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

    # After retrieving the context, we build our prompt for gpt-4o and make our completions request
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

Running the example outputs:

```json
```

The example above shows how we can use OpenAI or any other third party LLM to perform RAG.

A bullet point summary:
- Use Korvus to perform search
- Use the third party API provider to generate the text

