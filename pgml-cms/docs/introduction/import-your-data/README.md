---
description: Import your data into PostgresML using one of many supported methods.
---

# Import your data

AI needs data, whether it's generating text with LLMs, creating embeddings, or training regression or classification models on customer data.

Just like any PostgreSQL database, PostgresML can be configured as the primary application database, a logical replica of your primary database, or with foreign data wrappers to query your primary database on demand. Depending on how frequently your data changes and your latency requirements, one approach is better than the other.

## Primary database

If your intention is to use PostgresML as your primary database, your job here is done. You can use the connection credentials provided and start building your application on top of in-database AI right away.

## [Logical replication](logical-replication/)

If your primary database is hosted elsewhere, for example AWS RDS, or Azure Postgres, you can get your data replicated to PostgresML in real time using logical replication. 

<figure class="my-3 py-3"><img src="../../../.gitbook/assets/Getting-Started_Logical-Replication-Diagram.svg" alt="Logical replication" width="80%"><figcaption></figcaption></figure>

Having access to your data immediately is very useful to
accelerate your machine learning use cases and removes the need for moving data multiple times between microservices. Latency-sensitive applications should consider using this approach.

## [Foreign data wrappers](foreign-data-wrappers)

Foreign data wrappers are a set of PostgreSQL extensions that allow making direct connections from inside the database directly to other databases, even if they aren't running on Postgres. For example, Postgres has foreign data wrappers for MySQL, S3, Snowflake and many others.

<figure class="my-3 py-3"><img src="../../../.gitbook/assets/Getting-Started_FDW-Diagram.svg" alt="Foreign data wrappers" width="80%"><figcaption></figcaption></figure>

FDWs are useful when data access is infrequent and not latency-sensitive. For many use cases, like offline batch workloads and not very busy websites, this approach is suitable and easy to get started with.

## [Move data with COPY](copy)

`COPY` is a powerful PostgreSQL command to import data from a file format like CSV. Most data stores out there support exporting data using the CSV format, so moving data from your data source to PostgresML can almost always be done this way.

## [Migrate with pg_dump](pg-dump)

_pg_dump_ is a command-line PostgreSQL utility to migrate databases from one server to another. Databases of almost any size can be migrated with _pg_dump_ quickly and safely.
