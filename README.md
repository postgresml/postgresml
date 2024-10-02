<div align="center">
   <picture>
     <source media="(prefers-color-scheme: dark)" srcset="https://github.com/user-attachments/assets/5d5510da-6014-4cf3-849f-566050e053da">
     <source media="(prefers-color-scheme: light)" srcset="https://github.com/user-attachments/assets/aea1c38a-15bf-4270-8365-3d5e6311f5fc">
     <img alt="Logo" src="" width="520">
   </picture>
</div>

<p align="center">
   <p align="center"><b>Generative AI and Simple ML with PostgreSQL.</b></p>
</p>

<p align="center">
| <a href="https://postgresml.org/docs/"><b>Documentation</b></a> | <a href="https://postgresml.org/blog"><b>Blog</b></a> | <a href="https://discord.gg/DmyJP3qJ7U"><b>Discord</b></a> |
</p>

---
PostgresML is a complete ML/AI platform built inside PostgreSQL. Our operating principle is:

Move models to the database, rather than constantly moving data to the models.

Data for ML & AI systems is inherently larger and more dynamic than the models. It's more efficient, manageable and reliable to move models to the database, rather than continuously moving data to the models.

<b> Table of contents </b>
- [Getting started](#getting-started)
    - [PostgresML Cloud](#postgresml-cloud)
    - [Self-hosted](#self-hosted)
    - [Ecosystem](#ecosystem)
- [Large Language Models](#large-language-models)
    - [Hugging Face](#hugging-face)
    - [OpenAI and Other Providers](#openai)
- [RAG](#rag)
    - [Chunk](#chunk)
    - [Embed](#embed)
    - [Rank](#rank)
    - [Transform](#transform)
- [Machine Learning](#machine-learning)

![PostgresML Architecture](https://github.com/user-attachments/assets/508678a7-a34b-4903-9049-a83995eff2fb)

# Getting Started

The only prerequisites for using PostgresML is a Postgres database with our open-source `pgml` extension installed.

## PostgresML Cloud

Our serverless cloud is the easiest and recommend way to get started.

[Sign up for a free PostgresML account](https://postgresml.org/signup). You'll get a free database in seconds, with access to GPUs and state of the art LLMs.

## Self-hosted

If you don't want to use our cloud you can self host it.

```
docker run \
    -it \
    -v postgresml_data:/var/lib/postgresql \
    -p 5433:5432 \
    -p 8000:8000 \
    ghcr.io/postgresml/postgresml:2.7.12 \
    sudo -u postgresml psql -d postgresml
```

For more details, take a look at our [Quick Start with Docker](https://postgresml.org/docs/open-source/pgml/developers/quick-start-with-docker) documentation.

## Ecosystem

We have a number of other tools and libraries that are specifically designed to work with PostgreML. Remeber PostgresML is a postgres extension running inside of Postgres so you can connect with `psql` and use any of your favorite tooling and client libraries like [psycopg](https://www.psycopg.org/psycopg3/) to connect and run queries.

<b>PostgresML Specific Client Libraries:</b>
- [Korvus](https://github.com/postgresml/korvus) - Korvus is a Python, JavaScript, Rust and C search SDK that unifies the entire RAG pipeline in a single database query.
- [postgresml-django](https://github.com/postgresml/postgresml-django) - postgresml-django is a Python module that integrates PostgresML with Django ORM.

<b>Recommended Postgres Poolers:</b>
- [pgcat](https://github.com/postgresml/pgcat) - pgcat is a PostgreSQL pooler with sharding, load balancing and failover support.

# Large Language Models

PostgresML brings models directly to your data, eliminating the need for costly and time-consuming data transfers. This approach significantly enhances performance, security, and scalability for AI-driven applications.

<b>TODO: SOME MORE LANGUAGE ON THIS</b>

# Large Language Models

PostgresML brings models directly to your data, eliminating the need for costly and time-consuming data transfers. This approach significantly enhances performance, security, and scalability for AI-driven applications.

By running models within the database, PostgresML enables:

- Reduced latency and improved query performance
- Enhanced data privacy and security
- Simplified infrastructure management
- Seamless integration with existing database operations

## Hugging Face

PostgresML supports a wide range of state-of-the-art deep learning architectures available on the Hugging Face [model hub](https://huggingface.co/models). This integration allows you to:

- Access thousands of pre-trained models
- Utilize cutting-edge NLP, computer vision, and other AI models
- Easily experiment with different architectures

## OpenAI and Other Providers

While cloud-based LLM providers offer powerful capabilities, making API calls from within the database can introduce latency, security risks, and potential compliance issues. Currently, PostgresML does not directly support integration with remote LLM providers like OpenAI.

# Rag

PostgresML transforms your PostgreSQL database into a powerful vector database for Retrieval-Augmented Generation (RAG) applications. It leverages pgvector for efficient storage and retrieval of embeddings.

Our RAG implementation is built on four key SQL functions:

1. [Chunk](#chunk): Splits text into manageable segments
2. [Embed](#embed): Generates vector embeddings from text using pre-trained models
3. [Rank](#rank): Performs similarity search on embeddings
4. [Transform](#transform): Applies language models for text generation or transformation

For more information on using RAG with PostgresML see our guide on [Unified RAG](https://postgresml.org/docs/open-source/pgml/guides/unified-rag).

## Chunk

The `pgml.chunk` function chunks documents using the specified splitter. This is typically done before embedding.

```postgresql
pgml.chunk(
    splitter TEXT,    -- splitter name
    text TEXT,        -- text to embed
    kwargs JSON       -- optional arguments (see below)
)
```

See [pgml.chunk docs](https://postgresml.org/docs/open-source/pgml/api/pgml.chunk) for more information.

## Embed

The `pgml.embed` function generates embeddings from text using in-database models.

```postgresql
pgml.embed(
    transformer TEXT,
    "text" TEXT,
    kwargs JSONB
)
```
See [pgml.embed docs](https://postgresml.org/docs/open-source/pgml/api/pgml.embed) for more information.

## Rank

The `pgml.rank` function uses [Cross-Encoders](https://www.sbert.net/examples/applications/cross-encoder/README.html) to score sentence pairs.

This is typically used as a re-ranking step when performing search.

```postgresl
pgml.rank(
    transformer TEXT,
    query TEXT,
    documents TEXT[],
    kwargs JSONB
)
```

Docs coming soon.

## Transform

The `pgml.transform` function can be used to generate text.

```postgresql
SELECT pgml.transform(
    task   => TEXT OR JSONB,     -- Pipeline initializer arguments
    inputs => TEXT[] OR BYTEA[], -- inputs for inference
    args   => JSONB              -- (optional) arguments to the pipeline.
)
```

See [pgml.transform docs](https://postgresml.org/docs/open-source/pgml/api/pgml.transform) for more information.

See our [Text Generation guide](https://postgresml.org/docs/open-source/pgml/guides/llms/text-generation) for a guide on generating text.

# Machine Learning

<b>Some highlights:</b>
- [47+ classification and regression algorithms](https://postgresml.org/docs/open-source/pgml/api/pgml.train)
- [8 - 40X faster inference than HTTP based model serving](https://postgresml.org/blog/postgresml-is-8x-faster-than-python-http-microservices)
- [Millions of transactions per second](https://postgresml.org/blog/scaling-postgresml-to-one-million-requests-per-second)
- [Horizontal scalability](https://postgresml.org/docs/open-source/pgcat/)

**Training a classification model**

*Training*
```postgresql
SELECT * FROM pgml.train(
    'Handwritten Digit Image Classifier',
    algorithm => 'xgboost',
    'classification',
    'pgml.digits',
    'target'
);
```

*Inference*
```postgresql
SELECT pgml.predict(
    'My Classification Project',
    ARRAY[0.1, 2.0, 5.0]
) AS prediction;
```

## NLP

The `pgml.transform` function exposes a number of available NLP tasks.

Available tasks are:
- [Text Classification](https://postgresml.org/docs/open-source/pgml/guides/llms/text-classification)
- [Zero-Shot Classification](https://postgresml.org/docs/open-source/pgml/guides/llms/zero-shot-classification)
- [Token Classification](https://postgresml.org/docs/open-source/pgml/guides/llms/token-classification)
- [Translation](https://postgresml.org/docs/open-source/pgml/guides/llms/translation)
- [Summarization](https://postgresml.org/docs/open-source/pgml/guides/llms/summarization)
- [Question Answering](https://postgresml.org/docs/open-source/pgml/guides/llms/question-answering)
- [Text Generation](https://postgresml.org/docs/open-source/pgml/guides/llms/text-generation)
- [Text-to-Text Generation](https://postgresml.org/docs/open-source/pgml/guides/llms/text-to-text-generation)
- [Fill-Mask](https://postgresml.org/docs/open-source/pgml/guides/llms/fill-mask)
