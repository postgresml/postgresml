---
description: Meet Korvus, our new open-source tool that simplifies and unifies the entire RAG pipeline into a single database query.
featured: true
tags: [product]
image: ".gitbook/assets/Blog-Image_Korvus-Release.jpg"
---

# Introducing Korvus: The All-in-One RAG Pipeline for PostgresML 

<div align="left">

<figure><img src=".gitbook/assets/image.png" alt="Author" width="100"><figcaption></figcaption></figure>

</div>

Cassandra Stumer

July 10, 2024

You’re probably all too familiar with the complexities of building and maintaining RAG pipelines. The multiple services, the API calls, the data movement. Managing and scaling efficient infrastructure is the woefully painful and un-sexy side of building any ML/AI system. It’s also the most crucial factor when it comes to delivering real-world, production applications. That’s why we perform machine learning directly in PostgreSQL.

After hard-earned wisdom gained scaling the ML platform at Instacart, our team is bullish on in-database machine learning winning out as the AI infrastructure of the future. We know from experience that moving the compute to your database is far more efficient, effective and scalable than continuously moving your data to the models. That’s why we built PostgresML.

While we’re big Postgres fans, we asked ourselves: what if we could simplify all of that for folks who need a robust, production-grade RAG pipeline, but aren’t into SQL? Korvus is our answer. It's an extension of what we've been doing with PostgresML, but abstracts away the complexity of SQL-based operations. That way, more builders and users can reap the benefits of a unified, in-database RAG pipeline. 

Why is RAG better with Korvus? Korvus provides a high-level interface in multiple programming languages that unifies the entire RAG pipeline into a single database query. Yes, you read that right - one query to handle embedding generation, vector search, reranking, and text generation. <i>One query to rule them all</i>. 

Here's what's under the hood: Korvus’ core operations are built on optimized SQL queries. You’ll get high-performance, customizable search capabilities with minimal infrastructure concerns – and you can do it all in Python, JavaScript or Rust.

!!! info

Open a [GitHub issue](https://github.com/postgresml/korvus/issues) to vote on support for another language and we will add it to our roadmap.

!!!

Performing RAG directly where your data resides with optimized queries not only produces a faster app for users; but also gives you the ability to inspect, understand, and even customize these queries if you need to.

Plus, when you build on Postgres, you can leverage its vast ecosystem of extensions. The capabilities are robust; “just use Postgres” is a common saying for a reason. There’s truly an extension for everything, and extensions like pgvector, pgml and pgvectorscale couple all the performance and scalability you'd expect from Postgres with sophisticated ML/AI operations.

We're releasing Korvus as open-source software, and yes, it can run locally in Docker for those of you who like to tinker. In our (admittedly biased) opinion – it’s easiest to run Korvus on our serverless cloud. The PostgresML cloud comes with GPUs, and it’s preloaded with the extensions you’ll need to get started. Plus, you won’t have to manage a database. 

Once set up locally or in the PostgresML cloud, getting started with Korvus is easy!

!!! generic

!!! code_block

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

    # Perform RAG
    query = "Is Korvus fast?"
    print(f"Querying for response to: {query}")
    results = await collection.rag(
        {
            "CONTEXT": {
                "vector_search": {
                    "query": {
                        "fields": {"text": {"query": query}},
                    },
                    "document": {"keys": ["id"]},
                    "limit": 1,
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
                        "content": f"Given the context\n:{{CONTEXT}}\nAnswer the question briefly: {query}",
                    },
                ],
                "max_tokens": 100,
            },
        },
        pipeline,
    )
    print(results)

asyncio.run(main())
```

!!! 

!!! results

```json
{
    'rag': ['Yes, Korvus is incredibly fast!'],
    'sources': {
        'CONTEXT': [
            {
                'chunk': 'Korvus is incredibly fast and easy to use.',
                'document': {'id': '1'},
                'rerank_score': None,
                'score': 0.7542821004154432
            }
        ]
    }
}
```

!!!

!!!

Give it a spin, and let us know what you think. We're always here to geek out about databases and machine learning, so don't hesitate to reach out if you have any questions or ideas. We welcome you to: 

- [Join our Discord server](https://discord.gg/DmyJP3qJ7U)
- [Follow us on Twitter](https://twitter.com/postgresml)
- [Contribute to the project on GitHub](https://github.com/postgresml/korvus)

We're excited to see what you'll build with Korvus. Whether you're working on advanced search systems, content recommendation engines, or any other RAG-based application, we believe Korvus can significantly streamline your architecture and boost your performance.

Here's to simpler architectures and more powerful queries!
