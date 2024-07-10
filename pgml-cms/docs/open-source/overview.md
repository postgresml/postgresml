---
description: Overview of the PostgresML SQL API and SDK.
---

# Open Source Overview

PostgresML maintains three open source projects:
- [pgml](pgml/)
- [Korvus](korvus/)
- [pgcat](pgcat/)

## PGML

`pgml` is a PostgreSQL extension which adds SQL functions to the database where it's installed. The functions work with modern machine learning algorithms and latest open source LLMs while maintaining a stable API signature. They can be used by any application that connects to the database.

See the [`pgml` docs](pgml/) for more information about `pgml`.

## Korvus

Korvus is an all-in-one, open-source RAG (Retrieval-Augmented Generation) pipeline built for Postgres. It combines LLMs, vector memory, embedding generation, reranking, summarization and custom models into a single query, maximizing performance and simplifying your search architecture.

See the [Korvus docs](korvus/) for more information about Korvus.

## PgCat

PgCat is PostgreSQL connection pooler and proxy which scales PostgreSQL (and PostgresML) databases beyond a single instance

See the [PgCat docs](pgcat/) for more information about PgCat.
