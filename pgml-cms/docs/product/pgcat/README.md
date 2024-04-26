---
description: PgCat, the PostgreSQL connection pooler and proxy with support for sharding, load balancing, failover, and many more features.
---

# PgCat pooler

<div class="row">
	<div class="col-12 col-md-4">
		<figure class="my-4">
			<img class="mb-3" src="../../.gitbook/assets/pgcat_1.svg" height="auto" width="185" alt="PgCat logo">
			<figcaption></figcaption>
		</figure>
	</div>
	<div class="col-12 col-md-8">
		<p>PgCat is PostgreSQL connection pooler and proxy which scales PostgreSQL (and PostgresML) databases beyond a single instance.</p>
		<p>
			It supports replicas, load balancing, sharding, failover, and many more features expected out of high availability enterprise-grade PostgreSQL deployment.
		</p>
		<p>
			Written in Rust using Tokio, it takes advantage of multiple CPUs and the safety and performance guarantees of the Rust language.
		</p>
	</div>
</div>

PgCat, like PostgresML, is free and open source, distributed under the MIT license. It's currently running in our [cloud](https://postgresml.org/signup), powering both Serverless and Dedicated databases.

## [Features](features)

PgCat implements the PostgreSQL wire protocol and can understand and optimally route queries & transactions based on their characteristics. For example, if your database deployment consists of a primary and replica, PgCat can send all `SELECT` queries to the replica, and all other queries to the primary, creating a read/write traffic separation.

<figure>
	<img class="mb-3" src="../../.gitbook/assets/pgcat_4.png" alt="PgCat architecture" width="95%" height="auto">
	<figcaption><i>PgCat deployment at scale</i></figcaption>
</figure>

<br>

If you have more than one primary, sharded with either the Postgres hashing algorithm or a custom sharding function, PgCat can parse queries, extract the sharding key, and route the query to the correct shard without requiring any modifications on the client side.

PgCat has many more features which are  more thoroughly described in the [PgCat features](features) section.

## [Installation](installation)

PgCat is open source and available from our [GitHub repository](https://github.com/postgresml/pgcat) and, if you're running Ubuntu 22.04, from our Aptitude repository. You can read more about how to install PgCat in the [installation](installation) section.

## [Configuration](configuration)

PgCat, like many other PostgreSQL poolers, has its own configuration file format (it's written in Rust, so of course we use TOML). The settings and their meaning are documented in the [configuration](configuration) section.
