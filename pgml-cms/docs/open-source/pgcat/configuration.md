---
description: PgCat configuration settings & recommended default values.
---

# PgCat configuration

PgCat offers many features out of the box, and comes with good default values for most of its configuration options, but some minimal configuration is required before PgCat can start serving PostgreSQL traffic.

### General settings

General settings configure basic behavior of the pooler, e.g. what port it should run on, TLS configuration, admin database access, and various network timeouts.

If you're self-hosting PgCat, these need to be declared in the `[general]` section of `pgcat.toml` TOML configuration file.

#### host

The IP address of the interface the pooler should bind onto when starting up. If you'd like to allow connections only from `localhost`, set this to `127.0.0.1`.

Default value: `0.0.0.0`, allowing connections from everywhere.

#### port

The network port the pooler should bind onto when starting up. If running on the same host as Postgres with default settings, avoid using `5432` because it will cause a conflict.

Default value: `5432`

#### worker\_threads

The number of Tokio worker threads to launch. This should match the number of CPU cores available.

Default: `5`

#### connect\_timeout

Number of milliseconds to wait for a successful connection to a Postgres server. If this timeout expires, next candidate in the replica pool will be attempted until one succeeds or all replica candidates fail.

Default value: `1000`(1 second)

#### idle\_timeout

Number of milliseconds to keep an idle Postgres connection open and available in the connection pool. When this timeout expires, the connection will be closed.

Default value: `600000` (10 minutes)

#### server\_lifetime

Number of milliseconds a Postgres server connection is kept available in the connection pool. Once this expires, the connection is closed. This setting helps keeping Postgres connections fresh and avoids long-living processes on the Postgres server (if that's something that's desired).

Default value: `3600000` (1 hour)

#### idle\_client\_in\_transaction\_timeout

Number of milliseconds to allow a Postgres server connection to be idle, while in the middle of a transaction.

Default: `0`(disabled)

#### healthcheck\_timeout

Number of milliseconds to wait for a healthcheck query to return with a result. If this expires without an answer from the Postgres server, PgCat will mark the replica as unhealthy and stop sending it traffic.

Default: `1000` (1 second)

#### healthcheck\_delay

Number of milliseconds between mandatory healthchecks of a Postgres replica. No healthcheck will be issued for this duration after a successful healthcheck is completed.

Default: `30000`(30 seconds)

#### shutdown\_timeout

Number of milliseconds to wait for all clients to disconnect from the pooler when executing a graceful shutdown. When the timeout expires, the pooler will shutdown and disconnect all remaining clients.

Default: `60000` (60 seconds)

#### ban\_time

Number of seconds a replica will be removed from the pool for when marked as unhealthy by a healthcheck. Once this timeout expires, the replica will be placed back into the pool and healthchecked again.

Default: `60` (seconds)

#### tcp\_keepalives\_idle

Number of seconds to wait for an idle TCP connection to be reused until sending a Keep-Alive packet. This ensures that connections are still healthy and not terminated by the network.

Default: `5` (seconds)

#### tcp\_keepalives\_count

Number of unacknowledged TCP Keep-Alive packets that are allowed before forcibly terminating a TCP connection. This ensures that broken TCP connections due to network failure are recognized and closed in the pooler.

Default: `5`

#### tcp\_keepalives\_interval

Number of seconds to wait between sending Keep-Alive packets on an idle TCP connection. Multiplied with `tcp_keepalives_count`, this produces the total amount of time to wait before forcibly closing an idle and unresponsive TCP connection.

Default: `5`

#### prepared\_statements

Enables/disables support for prepared statements in transaction and session mode. Prepared statements are cached SQL queries that can be reused with different parameters and allow for dramatic increase in performance of `SELECT` queries in production.

Default: `false`(disabled)

#### prepared\_statements\_cache\_size

Number of prepared statements to keep in the pooler cache for reuse by the same client. The higher this setting, the higher the opportunity for a cache hit and not having to prepare the same SQL statement again. This is configurable and not infinite because keeping prepared statements in memory consumes PgCat memory and Postgres server resources.

Default: `500`

#### admin\_username

The username of the administrative user allowed to connect to the special admin database for managing PgCat.

Default: None (required)

#### admin\_password

The password of the administrative user allowed to connect to the admin database.

Default: None (required)

#### server\_tls

Enable TLS connections from PgCat to Postgres servers. Postgres has to be configured to support TLS, which is typical to be the case for Postgres distributed via package managers.

Default: false

#### verify\_server\_certificate

If `server_tls` is enabled, validate that the server certificate is valid. This disallows connections for self-signed certificates which haven't been added to the root store on the machines running PgCat.

Default: false (don't verify server certificates)

#### autoreload

Sets the interval in milliseconds at which PgCat will check its configuration file and if it changed, reload the configuration file automatically.

Default: disabled

#### dns\_cache\_enabled

If enabled, PgCat will resolve and cache DNS of Postgres servers, overriding default TTL provided by system DNS servers. This is useful when using DNS for configuring traffic routing to Postgres servers: if the IP resolved by the DNS query changed from its previously cached value, the connection pool will be automatically recreated with connections to the new Postgres server.

Default: `false`

#### dns\_max\_ttl

Maximum number of seconds to keep cached DNS values. Once this timeout expires, a DNS refresh is performed against all targets in the cache.

Default: `30` (seconds)

### Pools

PgCat is first and foremost a Postgres connection pooler. It supports proxying multiple users and databases, which are separated into their own independent connection pools for easier configuration and management.

To add a new connection pool to PgCat, you need to add it to the `[pools]` section using the TOML syntax for tables. The name of the pool is the name of the table in TOML, e.g. `[pools.name_of_the_pool]`.

The name of the pool is the name of the Postgres database seen by clients connecting to PgCat.

Each connection pool additionally can be configured with additional settings.

#### pool\_mode

Setting controlling Postgres server connection sharing behavior. `session` mode guarantees that a single server connection is used for each client connecting to PgCat. `transaction` mode shares server connections between multiple PgCat clients, allowing for higher concurrency and sharing of resources.

Default: `transaction`

#### load\_balancing\_mode

The algorithm used for load balancing traffic across read replicas. Currently, two algorithms are supported: `random` which chooses replicas at random using a standard random number generator, and `loc` or least outstanding connections, which selects the replica with the least number of clients waiting for a connection.

Default: `random`

#### query\_parser\_enabled

PgCat comes with a query parser that interprets all incoming SQL queries using the `sqlparser` Rust library. This allows the pooler to determine what the query intends to do, e.g. a read or a write, or to extract the sharding key. Since this feature requires additional compute, it's optional.

Default: `false`

#### query\_parser\_read\_write\_splitting

If enabled, together with `query_parser`, this will separate read queries (e.g. `SELECT`) from write queries (e.g. `INSERT`/`UPDATE`/`DELETE`, etc.), and route read queries to replicas and write queries to the primary.

Default: `false`

#### primary\_reads\_enabled

If enabled, together with `query_parser` and `query_parser_read_write_splitting`, this will allow the primary database to serve read queries, together with the replicas. This is beneficial in situations where read/write traffic separation is not necessary, e.g. when read queries outnumber write queries significantly, and the primary is not under significant load.

Default: `false`

#### sharding\_function

The sharding function used by the pooler to route queries to multiple primaries in a sharded configuration. Currently, two sharding functions are supported and included with PgCat: `PARTITION BY hash(bigint)` i.e. `pg_bigint_hash`, used by Postgres partitions, and a custom sharding function based on SHA1. More sharding functions can be added, but require a contribution to PgCat and aren't currently modular.

Default: `pg_bigint_hash`

#### automatic\_sharding\_key

Column name or fully-qualified table and column name expected to contain the sharding key. If specified, PgCat will attempt to extract it from every query that is processed by the pooler. If found, the value will be hashed and used to compute the correct shard. The query will then be routed to that shard automatically.

Example: `users.id` or `id`

Default: None (disabled)

#### idle\_timeout

Override of the `idle_timeout` configurable in the General settings.

#### connect\_timeout

Override of the `connect_timeout` configurable in the General settings.

### Users

PgCat supports multiple users in connection pools. Each user/pool pair translates internally to a separate connection pool and indepedent client and server connections.

User configuration allows for user-specific settings and additional overrides of general and pool settings.

#### username

The name of the user. This name is expected to be provided by all connecting clients and will match the client to the correct connection pool in PgCat.

Default: None (required)

#### password

The password for the user. Currently, PgCat only supports MD5 authentication, so the client should provide the password accordingly. All modern and legacy Postgres client libraries implement this authentication mechanism.

Default: None (optional, if not set, auth passthrough will be attempted)

#### server\_username

Username used by PgCat to connect to Postgres. Not required, and `username` will be used, if not configured. This allows for separation of client and server credentials, which is often needed when rotating users and passwords.

Default: None (using `username` setting)

#### server\_password

The password used to authenticate with the Postgres server. Not required, and `password` willbe used, if not configured. See `server_username` for use case description.

Default: None (using `password` setting)

#### pool\_size

Maximum number of Postgres connections allowed to be created to serve connections for clients connected to this pool. Lowering this number may increase queueing time for clients needing a Postgres connection to run a query. Increasing this number may increase Postgres server load and decrease overall performance of the system due to context switching.

Default: None (required)

#### min\_pool\_size

Minimum number of Postgres connections to keep open in the connection pool. This ensures that at least this many connections are available to serve clients and minimizes cold start times for new clients connecting to PgCat. Increasing this number may increase the number of unnecessarily open Postgres connection, wasting server resources and blocking other connection pools from using them. Decreasing this number could increase latency during burst traffic events.

Default: `0`

#### `statement_timeout`

Maximum number of milliseconds to wait for a server to answer a client's query. Not typically used in production, since Postgres implements this feature on the server. Use only if the connection between PgCat and Postgres is unreliable or the Postgres installation is known to be unstable.

Default: `0` (disabled)

#### pool\_mode

Override of the `pool_mode` setting for the connection pool defined in the `[pools.pool_name]` section.

#### server\_lifetime

Override of the `server_lifetime` configurable in the General settings.

### Shards

PgCat is built with sharding as a first class feature. All connection pools are built to support multiple shards and the default configuration format reflects that. The most common configuration includes only one shard, which is effectively an unsharded database.

#### database

The name of the Postgres database to connect to.

Default: None (required)

#### servers

The map of Postgres servers that power the shard, i.e. primary and replicas. This is an array of arrays. Each top level array is a server. Each server array contains three (3) values: the host/IP address of the server, the port, and the role (primary or replica).

For example:

```
servers = [
    ["10.0.0.1", 5432, "primary"],
    ["replica-1.internal-dns.net", 5432, "replica"],
]
```

### Minimal configuration example

This is an example of a minimal PgCat configuration for a simple primary-only unsharded Postgres database.

```
[general]
port = 6432
admin_username = "pgcat"
admin_password = "my-pony-likes-to-dance-tango"

[pools.my_database]

[pools.my_database.users.0]
pool_size = 5
username = "developer"
password = "very-secure-password"

[pools.my_database.shards.0]
database = "postgresml"
servers = [
    ["127.0.0.1", 5432, "primary"],
]
```

This configuration assumes the following:

* the pooler is running on the same machine as Postgres,
* a database called `postgresml` exists,
* a user called `developer` with the password `very-security-password` exists and has the `CONNECT` privilege on the `postgresml` database.

Using `psql`, you can connect to PgCat with the following command:

```
psql postgres://developer:very-secure-password@127.0.0.1:6432/my_database
```

Note that the database name used in the connection string is the pool name, not the actual name of the database in Postgres.

