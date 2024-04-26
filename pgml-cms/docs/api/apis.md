---
description: Overview of the PostgresML SQL API and SDK.
---

# API overview

PostgresML is a PostgreSQL extension which adds SQL functions to the database where it's installed. The functions work with modern machine learning algorithms and latest open source LLMs while maintaining a stable API signature. They can be used by any application that connects to the database.

In addition to the SQL API, we built and maintain a client SDK for JavaScript, Python and Rust. The SDK uses the same extension functionality to implement common ML & AI use cases, like retrieval-augmented generation (RAG), chatbots, and semantic & hybrid search engines.

Using the SDK is optional, and you can implement the same functionality with standard SQL queries. If you feel more comfortable using a programming language, the SDK can help you to get started quickly.

## [SQL extension](sql-extension/)

The PostgreSQL extension provides all of the ML & AI functionality, like training models and inference, via SQL functions. The functions are designed for ML practitioners to use dozens of ML algorithms to train models, and run real time inference, on live application data. Additionally, the extension provides access to the latest Hugging Face transformers for a wide range of NLP tasks.

### Functions 

The following functions are implemented and maintained by the PostgresML extension:

| Function name    | Description                                                                                                                                                                                        |
|------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| [pgml.embed()](sql-extension/pgml.embed)     | Generate embeddings inside the database using open source embedding models from Hugging Face.                                                                                                |
| [pgml.transform()](sql-extension/pgml.transform/) | Download and run latest Hugging Face transformer models, like Llama, Mixtral, and many more to perform various NLP tasks like text generation, summarization, sentiment analysis and more. |
| [pgml.train()](sql-extension/pgml.train/)     | Train a machine learning model on data from a Postgres table or view. Supports XGBoost, LightGBM, Catboost and all Scikit-learn algorithms.       |
| [pgml.deploy()](sql-extension/pgml.deploy)    | Deploy a version of the model created with pgml.train(). |
| [pgml.predict()](sql-extension/pgml.predict/) | Perform real time inference using a model trained with pgml.train() on live application data. |
| [pgml.tune()](sql-extension/pgml.tune) | Run LoRA fine tuning on an open source model from Hugging Face using data from a Postgres table or view. |

Together with standard database functionality provided by PostgreSQL, these functions allow to create and manage the entire life cycle of a machine learning application.

## [Client SDK](client-sdk/)

The client SDK implements best practices and common use cases, using the PostgresML SQL functions and standard PostgreSQL features to do it. The SDK core is written in Rust, which manages creating and running queries, connection pooling, and error handling.

For each additional language we support (current JavaScript and Python), we create and publish language-native bindings. This architecture ensures all programming languages we support have identical APIs and similar performance when interacting with PostgresML.

### Use cases

The SDK currently implements the following use cases:

| Use case | Description |
|----------|---------|
| [Collections](client-sdk/collections) | Manage documents, embeddings, full text and vector search indexes, and more, using one simple interface. |
| [Pipelines](client-sdk/pipelines) | Easily build complex queries to interact with collections using a programmable interface. |
| [Vector search](client-sdk/search) | Implement semantic search using in-database generated embeddings and ANN vector indexes. |
| [Document search](client-sdk/document-search) | Implement hybrid full text search using in-database generated embeddings and PostgreSQL tsvector indexes. |
