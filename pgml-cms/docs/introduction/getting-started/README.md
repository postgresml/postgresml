---
description: Getting starting with PostgresML, a GPU powered machine learning database.
---

# Getting started

This guide will walk you through the steps of getting started with PostgresML by following these easy steps:

1. [Create a free cloud account with PostgresML](/docs/introduction/getting-started/create-your-database#sign-up-for-an-account). This also creates a PostgresML database and includes access to GPU-accelerated models and 5 GB of storage.
2. [Select a plan](create-your-database#select-a-plan).
3. [Connect your PostgreSQL client to your PostgresML database](connect-your-app).

If you would prefer to run PostgresML locally, you can skip to our [Developer Docs](/docs/resources/developer-docs/quick-start-with-docker).

## How PostgresML works

A PostgresML deployment consists of multiple components working in concert to provide a complete Machine Learning platform:

* PostgreSQL database, with [_pgml_](/docs/api/sql-extension/), _pgvector_ and many other extensions that add features useful in day-to-day and machine learning use cases
* [PgCat pooler](/docs/product/pgcat/) to load balance thousands of concurrenct client requests across several database instances
* A web application to manage deployed models and share experiments analysis with SQL notebooks

We provide a fully managed solution in [our cloud](create-your-database), and document a self-hosted installation in the [Developer Docs](/docs/resources/developer-docs/quick-start-with-docker).

<figure class="my-4"><img src="../../.gitbook/assets/architecture.png" alt="PostgresML architecture"><figcaption></figcaption></figure>

By building PostgresML on top of a mature database, we get reliable backups for model inputs and proven scalability without reinventing the wheel, so that we can focus on providing access to the latest developments in open source machine learning and artificial intelligence.