---
description: >-
  Integrating PostgresML with other managed PostgreSQL services like AWS RDS.
---

# Using PostgresML with AWS RSD

<div align="left">

<figure><img src=".gitbook/assets/lev.jpg" alt="Author" width="125"><figcaption></figcaption></figure>

</div>

Lev Kokotov

April 19, 2024

## Introduction

PostgresML is an Postgres extension that enables you to run machine learning models and LLMs inside your database. Our managed cloud provides PostgresML deployments on GPUs, but other cloud providers like AWS RDS do not. In an effort to make PostgresML available to everyone,
we're adding support for using PostgresML with AWS RDS and other cloud providers.

## Getting started

Make sure to create an account on our cloud and create a serverless AI engine.
