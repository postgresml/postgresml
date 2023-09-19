---
description: PostgresML Frequently Asked Questions
---

# FAQs

## What is PostgresML?

PostgresML is an open-source database extension that turns Postgres into an end-to-end machine learning platform. It allows you to build, train, and deploy ML models directly within your Postgres database without moving data between systems.&#x20;

## What is a DB extension?&#x20;

A database extension is software that extends the capabilities of a database. Postgres allows extensions to add new data types, functions, operators, indexes, etc. PostgresML uses extensions to bring machine learning capabilities natively into Postgres.

## How does it work?&#x20;

PostgresML installs as extensions in Postgres. It provides SQL API functions for each step of the ML workflow like importing data, transforming features, training models, making predictions, etc. Models are stored back into Postgres tables. This unified approach eliminates complexity.

## What are the benefits?

Benefits include faster development cycles, reduced latency, tighter integration between ML and applications, leveraging Postgres' reliability and ACID transactions, and horizontal scaling.&#x20;

## What are the cons?

PostgresML requires using Postgres as the database. If your data currently resides in a different database, there would be some upfront effort required to migrate the data into Postgres in order to utilize PostgresML's capabilities.

## What is hosted PostgresML?

Hosted PostgresML is a fully managed cloud service that provides all the capabilities of open source PostgresML without the need to run your own database infrastructure.

With hosted PostgresML, you get:

* Flexible compute resources - Choose CPU, RAM or GPU machines tailored to your workload
* Horizontally scalable inference with read-only replicas&#x20;
* High availability for production applications with multi-region deployments&#x20;
* Support for multiple users and databases&#x20;
* Automated backups and point-in-time restore&#x20;
* Monitoring dashboard with metrics and logs&#x20;

In summary, hosted PostgresML removes the operational burden so you can focus on developing machine learning applications, while still getting the benefits of the unified PostgresML architecture.
