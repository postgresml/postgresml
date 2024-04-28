# PostgresML architecture

PostgresML is an extension for the PostgreSQL database server. It operates inside the database, using the same hardware to perform machine learning tasks.

## PostgreSQL

PostgreSQL is a process-based database server. It handles multiple connections by forking the main process, which creates OS-level isolation between clients.

<figure>
	<img src="/docs/.gitbook/assets/architecture_1.png" alt="Architecture" width="100%">
	<figcaption class="mt-4"><i>PostgreSQL architecture</i></figcaption>
</figure>

The main process allocates a block of shared memory, and grants all client processes direct access to it. The shared memory is used to store data retrieved from disk, so clients can re-use the same data for different queries.

This architecture is perfect for machine learning.

## PostgresML extension

A process-based architecture is perfect for multi-tenant machine learning applications. Each client connection loads its own libraries and models, serves them to the client, and removes all traces of them when the connection is closed.

<figure>
	<img src="/docs/.gitbook/assets/architecture_2.png" alt="Architecture" width="60%">
	<figcaption class="mt-4"><i>Per-connection models</i></figcaption>
</figure>

Since PostgreSQL shares data between clients, the expensive part of retrieving data is optimized, while the relatively inexpensive part of loading models into memory is automated and isolated.

## Conclusion

By running on the same machine and relying on PostgreSQL for scaling, stability, and performance, PostgresML eliminates network latency and brittleness of a service-based architecture.
