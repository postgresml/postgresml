---
description: The key concepts that make up PostgresML.
---

# Overview

PostgresML is a complete MLOps platform built on PostgreSQL.&#x20;

> _Move the models to the database_, _rather than continuously moving the data to the models._

The data for ML & AI systems is inherently larger and more dynamic than the models. It's more efficient, manageable and reliable to move the models to the database, rather than continuously moving the data to the models_._ PostgresML allows you to take advantage of the fundamental relationship between data and models, by extending the database with the following capabilities and goals:

* **Model Serving** - _**GPU accelerated**_ inference engine for interactive applications, with no additional networking latency or reliability costs.
* **Model Store** - Download _**open-source**_ models including state of the art LLMs from HuggingFace, and track changes in performance between versions.
* **Model Training** - Train models with _**your application data**_ using more than 50 algorithms for regression, classification or clustering tasks. Fine tune pre-trained models like LLaMA and BERT to improve performance.
* **Feature Store** - _**Scalable**_ access to model inputs, including vector, text, categorical, and numeric data. Vector database, text search, knowledge graph and application data all in one _**low-latency**_ system.&#x20;

<figure><img src=".gitbook/assets/ml_system.svg" alt="Machine Learning Infrastructure (2.0) by a16z"><figcaption><p>PostgresML handles all of the functions typically performed by a cacophony of services, <a href="https://a16z.com/emerging-architectures-for-modern-data-infrastructure/">described by a16z</a></p></figcaption></figure>

These capabilities are primarily provided by two open-source software projects, that may be used independently, but are designed to be used with the rest of the Postgres ecosystem, including trusted extensions like pgvector and pg\_partman.&#x20;

* **pgml** is an open source extension for PostgreSQL. It adds support for GPUs and the latest ML & AI algorithms _**inside**_ the database with a SQL API and no additional infrastructure, networking latency, or reliability costs.
* **PgCat** is an open source proxy pooler for PostgreSQL. It abstracts the scalability and reliability concerns of managing a distributed cluster of Postgres databases. Client applications connect only to the proxy, which handles load balancing and failover, _**outside**_ of any single database.

<figure><img src=".gitbook/assets/architecture.png" alt="PostgresML architectural diagram" width="275"><figcaption><p>A PostgresML deployment at scale</p></figcaption></figure>

In addition, PostgresML provides [native language SDKs](https://github.com/postgresml/postgresml/tree/master/pgml-sdks/pgml) to implement best practices for common ML & AI applications. The JavaScript and Python SDKs are generated from the core Rust SDK, to provide the same API, correctness and efficiency across all application runtimes.&#x20;

SDK clients can perform advanced machine learning tasks in a single SQL request, without having to transfer additional data, models, hardware or dependencies to the client application. For example:

* Chat with streaming response support from the latest LLMs
* Search with both keywords and embedding vectors
* Text Generation with RAG in a single request
* Translate text between hundreds of language pairs
* Summarization to distil complex documents
* Forecasting timeseries data for key metrics with complex metadata
* Fraud and anomaly detection with application data

Our goal is to provide access to Open Source AI for everyone. PostgresML is under continuous development to keep up with the rapidly evolving use cases for ML & AI, and we release non breaking changes with minor version updates in accordance with SemVer. We welcome contributions to our [open source code and documentation](https://github.com/postgresml).&#x20;

We can host your AI database in our cloud, or you can run our Docker image locally with PostgreSQL, pgml, pgvector and NVIDIA drivers included.
