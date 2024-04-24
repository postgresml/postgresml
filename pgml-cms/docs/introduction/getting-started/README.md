---
description: Setup a database and connect your application to PostgresML
---

# Getting Started

A PostgresML deployment consists of multiple components working in concert to provide a complete Machine Learning platform. We provide a fully managed solution in our cloud, and document a self-hosted installation in our docs.

* PostgreSQL database, with `pgml`, `pgvector` and many other extensions installed, including backups, metrics, logs, replicas and high availability
* PgCat pooler to provide secure access and model load balancing across thousands of clients
* A web application to manage deployed models and write experiments in SQL notebooks

<figure><img src="../../.gitbook/assets/architecture.png" alt=""><figcaption></figcaption></figure>

By building PostgresML on top of a mature database, we get reliable backups for model inputs and proven scalability without reinventing the wheel, so that we can focus on providing access to the latest developments in open source machine learning and artificial intelligence.

This guide will help you get started with a generous free account, that includes access to GPU accelerated models and 5GB of storage, or you can skip to our Developer Docs to see how to run PostgresML locally with our Docker image.
