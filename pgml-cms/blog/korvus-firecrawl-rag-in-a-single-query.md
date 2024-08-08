---
description: How to perform all-in-one RAG over any website with Firecrawl and Korvus.
featured: true
tags: [engineering]
image: ".gitbook/assets/Blog-Image_Korvus-Firecrawl.jpg"
---

# Korvus x Firecrawl: RAG in a single query

<div align="left">

<figure><img src=".gitbook/assets/silas.jpg" alt="Author" width="100"><figcaption></figcaption></figure>

</div>

Silas Marvin

July 30, 2024

We’re excited to share a quick guide on how you use the power of Korvus’ single query RAG along with Firecrawl to quickly and easily standup a retrieval augmented generation system with data from any website. 

You’ll learn how to:

1. Use Firecrawl to efficiently scrape web content (we’re using our blog as an example)
2. Process and index the scraped data using Korvus's Pipeline and Collection
3. Perform vector search, text generation and reranking (RAG) in a single query, using open-source models

[Firecrawl](https://firecrawl.dev) is a nifty web scraper that turns websites into clean, structured markdown data — perfect to create a knowledge base for RAG applications. 

[Korvus](https://github.com/postgresml/korvus) is the Python, JavaScript, Rust or C SDK for PostgresML. It handles the heavy lifting of document processing, vector search, and response generation in a single query. 

[PostgresML](https://postgresml.org) is an in-database ML/AI engine built by the ML engineers at Instacart. It lets you train, test and deploy models right inside Postgres. With Korvus, you can get all the efficiencies of in-database machine learning without SQL or database management. 

These three tools are all you’ll need to deploy a flexible and powerful RAG stack grounded in web data. Since your data is stored right where you're performing inference, you won’t need a vector database or an additional framework like LlamaIndex or Langchain to tie everything together. Mo’ microservices = more problems. 

Let’s dive in!

## Getting Started

To follow along you will need to set both the `FIRECRAWL_API_KEY` and `KORVUS_DATABASE_URL` env variables.

Sign up at [firecrawl.dev](https://www.firecrawl.dev/) to get your `FIRECRAWL_API_KEY`. 

The easiest way to get your `KORVUS_DATABASE_URL` is by signing up at [postgresml.org](https://postgresml.org) but you can also host postgres with the `pgml` and `pgvector` extensions yourself.

### Some Imports

First, let's break down the initial setup and imports:

```python
from korvus import Collection, Pipeline
from firecrawl import FirecrawlApp
import os
import time
import asyncio
from rich import print

# Initialize the FirecrawlApp with your API key
firecrawl = FirecrawlApp(api_key=os.environ["FIRECRAWL_API_KEY"])
```

Here we're importing `korvus`, `firecrawl`, and some other convenient libraries, and initializing the `FirecrawlApp` with an API key stored in an environment variable. This setup allows us to use Fire Crawl for web scraping.

### Defining the Pipeline and Collection

Next, we define our Pipeline and Collection:

```python
pipeline = Pipeline(
    "v0",
    {
        "markdown": {
            "splitter": {"model": "markdown"},
            "semantic_search": {
                "model": "mixedbread-ai/mxbai-embed-large-v1",
            },
        },
    },
)
collection = Collection("fire-crawl-demo-v0")

# Add our Pipeline to our Collection
async def add_pipeline():
    await collection.add_pipeline(pipeline)
```

This Pipeline configuration tells Korvus how to process our documents. It specifies that we'll be working with markdown content, using a markdown-specific splitter, and the `mixedbread-ai/mxbai-embed-large-v1` model for semantic search embeddings.

See the [Korvus guide to construction Pipelines](https://postgresml.org/docs/open-source/korvus/guides/constructing-pipelines) for more information on Collections and Pipelines.

### Web Crawling with Fire Crawl

The `crawl()` function demonstrates how to use Fire Crawl to scrape a website:

```python
def crawl():
    crawl_url = "https://postgresml.org/blog"
    params = {
        "crawlerOptions": {
            "excludes": [],
            "includes": ["blog/*"],
            "limit": 250,
        },
        "pageOptions": {"onlyMainContent": True},
    }
    job = firecrawl.crawl_url(crawl_url, params=params, wait_until_done=False)
    while True:
        print("Scraping...")
        status = firecrawl.check_crawl_status(job["jobId"])
        if not status["status"] == "active":
            break
        time.sleep(5)
    return status
```

This function initiates a crawl of the PostgresML blog, focusing on blog posts and limiting the crawl to 250 pages. It then periodically checks the status of the crawl job until it's complete.

Alternativly to sleeping, we could set the `wait_until_done` parameter to `True` and the `crawl_url` method would block until the data is ready.


### Processing and Indexing the Crawled Data

After crawling the website, we need to process and index the data for efficient searching. This is done in the `main()` function:

```python
async def main():
    # Add our Pipeline to our Collection
    await add_pipeline()

    # Crawl the website
    results = crawl()

    # Construct our documents to upsert
    documents = [
        {"id": data["metadata"]["sourceURL"], "markdown": data["markdown"]}
        for data in results["data"]
    ]

    # Upsert our documents
    await collection.upsert_documents(documents)
```

This code does the following:
1. Adds the previously defined pipeline to our collection.
2. Crawls the website using the `crawl()` function.
3. Constructs a list of documents from the crawled data, using the source URL as the ID and the markdown content as the document text.
4. Upserts these documents into the collection. The pipeline automatically splits the markdown and generates embeddings for each chunk storing it all in Postgres.

### Performing RAG

With our data indexed, we can now perform RAG:

```python
async def do_rag(user_query):
    results = await collection.rag(
        {
            "CONTEXT": {
                "vector_search": {
                    "query": {
                        "fields": {
                            "markdown": {
                                "query": user_query,
                                "parameters": {
                                    "prompt": "Represent this sentence for searching relevant passages: "
                                },
                            }
                        },
                    },
                    "document": {"keys": ["id"]},
                    "rerank": {
                        "model": "mixedbread-ai/mxbai-rerank-base-v1",
                        "query": user_query,
                        "num_documents_to_rerank": 100,
                    },
                    "limit": 5,
                },
                "aggregate": {"join": "\n\n\n"},
            },
            "chat": {
                "model": "meta-llama/Meta-Llama-3.1-405B-Instruct",
                "messages": [
                    {
                        "role": "system",
                        "content": "You are a question and answering bot. Answer the users question given the context succinctly.",
                    },
                    {
                        "role": "user",
                        "content": f"Given the context\n<context>\n:{{CONTEXT}}\n</context>\nAnswer the question: {user_query}",
                    },
                ],
                "max_tokens": 256,
            },
        },
        pipeline,
    )
    return results
```

This function combines vector search, reranking, and text generation to provide context-aware answers to user queries. It uses the Meta-Llama-3.1-405B-Instruct model for text generation.

This query can be broken down into 4 steps:
1. Perform vector search finding the 100 best matching chunks for the `user_query`
2. Rerank the results of the vector search using the `mixedbread-ai/mxbai-rerank-base-v1` cross-encoder and limit the results to 5
3. Join the reranked results with `\n\n\n` and substitute them in place of the `{{CONTEXT}}` placeholder in the messages
4. Perform text-generation with `meta-llama/Meta-Llama-3.1-405B-Instruct`

This is a complex query and there are more options and parameters to be tuned. See the [Korvus guide to RAG](https://postgresml.org/docs/open-source/korvus/guides/rag) for more information on the `rag` method.

### All Together Now

To tie everything together, we use an interactive loop in our `main()` function:

```python
async def main():
    # ... (previous code for setup and indexing)

    # Now we can search
    while True:
        user_query = input("\n\nquery > ")
        if user_query == "q":
            break
        results = await do_rag(user_query)
        print(results)

asyncio.run(main())
```

This loop allows users to input queries and receive RAG-powered responses based on the crawled and indexed content from the PostgresML blog.

## Conclusion

In this guide, we've demonstrated how to create a powerful RAG system using Fire Crawl and Korvus. Here's a summary of what we've accomplished:

1. Used Fire Crawl to efficiently scrape content from the PostgresML blog.
2. Processed and indexed the scraped data using Korvus's Pipeline and Collection.
3. Implemented RAG with vector search with reranking for accurate information retrieval.

This is just a small example of what can be done with [Fire Crawl](https://firecrawl.dev) and [Korvus](https://github.com/postgresml/korvus). We can't wait to see what you will make!
