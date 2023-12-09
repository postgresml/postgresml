# APIs

## Introduction

PostgresML adds extensions to the PostgreSQL database, as well as providing separate Client SDKs in JavaScript and Python that leverage the database to implement common ML & AI use cases.&#x20;

The extensions provide all of the ML & AI functionality via SQL APIs, like training and inference. They are designed to be used directly for all ML practitioners who implement dozens of different use cases on their own machine learning models.&#x20;

We also provide Client SDKs that implement the best practices on top of the SQL APIs, to ease adoption and implement common application use cases in applications, like chatbots or search engines.

## SQL Extensions

Postgres is designed to be _**extensible**_. This has created a rich open-source ecosystem of additional functionality built around the core project. Some [extensions](https://www.postgresql.org/docs/current/contrib.html) are include in the base Postgres distribution, but others are also available via the [PostgreSQL Extension Network](https://pgxn.org/). \
\
There are 2 foundational extensions included in a PostgresML deployment that provide functionality inside the database through SQL APIs.

* **pgml** - provides Machine Learning and Artificial Intelligence APIs with access to more than 50 ML algorithms to train classification, clustering and regression models on your own data, or you can perform dozens of tasks with thousands of models downloaded from HuggingFace.
* **pgvector** - provides indexing and search functionality on vectors, in addition to the traditional application database storage, including JSON and plain text, provided by PostgreSQL.

Learn more about developing with the [sql-extensions](sql-extensions/ "mention")

## Client SDKs

PostgresML provides client SDKs that streamline ML & AI use cases in both JavaScript and Python. With these SDKs, you can seamlessly manage various database tables related to documents, text chunks, text splitters, LLM (Language Model) models, and embeddings. By leveraging the SDK's capabilities, you can efficiently index LLM embeddings using pgvector with HNSW for fast and accurate queries.

These SDKs delegate all work to the extensions running in the database, which minimizes software and hardware dependencies that need to be maintained at the application layer, as well as securing data and models inside the data center. Our SDKs minimize data transfer to maximize performance, efficiency, security and reliability.

Learn more about developing with the [client-sdks](client-sdks/ "mention")





##
