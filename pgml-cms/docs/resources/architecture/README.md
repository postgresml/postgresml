# PostgresML architecture

PostgresML is an extension for the PostgreSQL database server. It operates inside the database, using the same hardware to perform machine learning tasks.

## PostgreSQL foundation

PostgreSQL is a process-based database server. It handles multiple connections by forking the main process, which creates OS-level isolation between clients.

<figure>
	<img src="/docs/.gitbook/assets/architecture_1.png" alt="PostgreSQL architecture" width="100%">
	<figcaption class="mt-4"><i>PostgreSQL architecture</i></figcaption>
</figure>

The main process allocates a block of shared memory, and grants all client processes direct access. Shared memory is used to store data retrieved from disk, so different clients can re-use the same data for different queries.

Data access is controlled with lightweight locking and transaction-based multi-version concurrency control (MVCC). Each client gets its own version of the entire database, which remains consistent for the duration of the transaction.

This architecture is perfect for machine learning.

## PostgresML open-source extension

A process-based architecture is perfect for multi-tenant machine learning applications. Each client connection loads its own libraries and models, serves them to the client, and removes all traces of them when the connection is closed.

<figure>
	<img src="/docs/.gitbook/assets/architecture_2.png" alt="PostgresML models" width="60%">
	<figcaption class="mt-4"><i>PostgresML models</i></figcaption>
</figure>

Since PostgreSQL shares data between clients, the expensive part of retrieving data is optimized, while the relatively inexpensive part of loading models into memory is automated and isolated. MVCC ensures that models trained in the database are consistent: no new data is added or removed during training.

### Optimizations

Most classical machine learning models are small: an average XGBoost model could be only a few megabytes, which is easy to load into memory for each connection process. LLMs like Mistral and Llama can range anywhere between a few gigabytes to hundreds of gigabytes, and most machines can only afford to load one instance at a time.

To share models between multiple clients, PostgresML, just like PostgreSQL, takes advantage of a connection pooler. We've built our own, called [PgCat](/docs/product/pgcat/), which supports load balancing, sharding, and many more enterprise-grade features.

<figure>
	<img src="/docs/.gitbook/assets/architecture_3.png" alt="Connection pooling" width="80%">
	<figcaption class="mt-4"><i>Connection pooling</i></figcaption>
</figure>

Pooling connections allows thousands of clients to reuse one PostgreSQL server connection. That server connection loads one instance of a LLM and shares it with all clients, one transaction at a time.

If the machine has enough RAM and GPU memory, more instances of the model can be loaded by allowing more than one server connection. PgCat will route client queries at random and evenly load balance the queries across all available LLM instances.
