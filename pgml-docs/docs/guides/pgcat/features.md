# Features

PgCat has many features currently in various stages of readiness and development. Most of its features are used in production and at scale.

### Query load balancing

PgCat is able to load balance Postgres queries against multiple replicas automatically. Clients connect to a single PgCat instance, which pretends to be a single Postgres database, and can issue as many queries as they need. The queries are then evenly distributed to all available replicas using configurable load balancing strategies.

### High availability

Just like any other modern load balancer, PgCat supports healthchecks and failover. PgCat maintains an internal map of healthy and unhealthy replicas, and routes traffic only to the healthy ones.

All replicas are periodically checked, and if they are responding, placed into the healthy pool. If the healthcheck fails, they are removed from that pool for a configurable amount of time, until they are checked again. This allows PgCat to run independently of any other Postgres management system and make decisions based on its own internal knowledge or configuration.

### Read/write query separation

Postgres is typically deployed in a one primary and many replicas architecture, where write queries go to a single primary, and read queries are distributed to either all machines or just the read replicas. PgCat can inspect incoming queries, parse the SQL to determine if the query intends to read or write, and route the query to either the primary or the replicas, as needed.

This allows for much simpler application configuration and opens up at scale deployments to all application frameworks, which currently require developers to manually route queries (e.g. Rails, Django, and others).

### Multithreading

PgCat is written in Rust using Tokio, which gives it the ability to use as many CPUs as are available. This simplifies deployments in environments with large transactional workloads, by requiring only one instance of PgCat per hardware instance.

This architecture allows to offload more work to the pooler which would otherwise would have to be implemented in the clients, without blocking them from accessing the database. For example, if we wanted to perform some CPU-intensive workload per query, we would be able to do so for multiple connections at a time.

### Sharding

Sharding allows to horizontally scale write queries, something that wasn't possible with typical Postgres deployments. PgCat is able to inspect incoming queries, extract the sharding key, hash it, and route the query to the correct primary, without requiring clients to modify their code.

PgCat also accepts a custom SQL syntax to override its sharding decisions, e.g. when the clients want to talk to a specific shard and, when clients want full control over sharding, a query comment indicating the desired shard for that query.

Since PgCat is a proxy, it makes decisions only based on configuration and its internal knowledge of the architecture. Therefore, it doesn't move data around and reshard Postgres clusters. It works in tandem with other tools that shard Postgres, and supports multiple hashing and routing functions, depending on the sharding tool.

### Standard features

In addition to novel features that PgCat introduces to Postgres deployments, it supports all the standard features expected from a pooler:

* authentication, multiple users and databases
* TLS encryption
* live configuration reloading
* statistics and an admin database for pooler management
* transaction and session mode

