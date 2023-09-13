---
description: Nextgen PostgreSQL Pooler
---

# PgCat

PgCat is a PostgreSQL pooler and proxy (like PgBouncer) with support for sharding, load balancing, failover and mirroring.

## Features

| **Feature**                           | **Status**       | **Comments**                                                                                                                                                   |
| ------------------------------------- | ---------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Transaction pooling                   | **Stable**       | Identical to PgBouncer with notable improvements for handling bad clients and abandoned transactions.                                                          |
| Session pooling                       | **Stable**       | Identical to PgBouncer.                                                                                                                                        |
| Multi-threaded runtime                | **Stable**       | Using Tokio asynchronous runtime, the pooler takes advantage of multicore machines.                                                                            |
| Load balancing of read queries        | **Stable**       | Queries are automatically load balanced between replicas and the primary.                                                                                      |
| Failover                              | **Stable**       | Queries are automatically rerouted around broken replicas, validated by regular health checks.                                                                 |
| Admin database statistics             | **Stable**       | Pooler statistics and administration via the `pgbouncer` and `pgcat` databases.                                                                                |
| Prometheus statistics                 | **Stable**       | Statistics are reported via a HTTP endpoint for Prometheus.                                                                                                    |
| SSL/TLS                               | **Stable**       | Clients can connect to the pooler using TLS. Pooler can connect to Postgres servers using TLS.                                                                 |
| Client/Server authentication          | **Stable**       | Clients can connect using MD5 authentication, supported by `libpq` and all Postgres client drivers. PgCat can connect to Postgres using MD5 and SCRAM-SHA-256. |
| Live configuration reloading          | **Stable**       | Identical to PgBouncer; all settings can be reloaded dynamically (except `host` and `port`).                                                                   |
| Auth passthrough                      | **Stable**       | MD5 password authentication can be configured to use an `auth_query` so no cleartext passwords are needed in the config file.                                  |
| Sharding using extended SQL syntax    | **Experimental** | Clients can dynamically configure the pooler to route queries to specific shards.                                                                              |
| Sharding using comments parsing/Regex | **Experimental** | Clients can include shard information (sharding key, shard ID) in the query comments.                                                                          |
| Automatic sharding                    | **Experimental** | PgCat can parse queries, detect sharding keys automatically, and route queries to the correct shard.                                                           |
| Mirroring                             | **Experimental** | Mirror queries between multiple databases in order to test servers with realistic production traffic.                                                          |

## Status

PgCat is stable and used in production to serve hundreds of thousands of queries per second.

|                                                                                              |                                                                                                 |           |
| -------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- | --------- |
| [Instacart](https://tech.instacart.com/adopting-pgcat-a-nextgen-postgres-proxy-3cf284e68c2f) | [PostgresML](https://postgresml.org/blog/scaling-postgresml-to-one-million-requests-per-second) | OneSignal |

Some features remain experimental and are being actively developed. They are optional and can be enabled through configuration.

## Deployment

See `Dockerfile` for example deployment using Docker. The pooler is configured to spawn 4 workers so 4 CPUs are recommended for optimal performance. That setting can be adjusted to spawn as many (or as little) workers as needed.

A Docker image is available from `docker pull ghcr.io/postgresml/pgcat:latest`. See our [Github packages repository](https://github.com/postgresml/pgcat/pkgs/container/pgcat).

For quick local example, use the Docker Compose environment provided:

```bash
docker-compose up

# In a new terminal:
PGPASSWORD=postgres psql -h 127.0.0.1 -p 6432 -U postgres -c 'SELECT 1'
```

### Config

See [**Configuration**](https://github.com/levkk/pgcat/blob/main/CONFIG.md).

## Contributing

The project is being actively developed and looking for additional contributors and production deployments.

### Local development

1. Install Rust (latest stable will work great).
2. `cargo build --release` (to get better benchmarks).
3. Change the config in `pgcat.toml` to fit your setup (optional given next step).
4. Install Postgres and run `psql -f tests/sharding/query_routing_setup.sql` (user/password may be required depending on your setup)
5. `RUST_LOG=info cargo run --release` You're ready to go!

### Tests

When making substantial modifications to the protocol implementation, make sure to test them with pgbench:

```
pgbench -i -h 127.0.0.1 -p 6432 && \
pgbench -t 1000 -p 6432 -h 127.0.0.1 --protocol simple && \
pgbench -t 1000 -p 6432 -h 127.0.0.1 --protocol extended
```

See sharding README for sharding logic testing.

Additionally, all features are tested with Ruby, Python, and Rust unit and integration tests.

Run `cargo test` to run Rust unit tests.

Run the following commands to run Ruby and Python integration tests:

```
cd tests/docker/
docker compose up --exit-code-from main # This will also produce coverage report under ./cov/
```

### Docker-based local development

You can open a Docker development environment where you can debug tests easier. Run the following command to spin it up:

```
./dev/script/console
```

This will open a terminal in an environment similar to that used in tests. In there, you can compile the pooler, run tests, do some debugging with the test environment, etc. Objects compiled inside the container (and bundled gems) will be placed in `dev/cache` so they don't interfere with what you have on your machine.

## Usage

### Session mode

In session mode, a client talks to one server for the duration of the connection. Prepared statements, `SET`, and advisory locks are supported. In terms of supported features, there is very little if any difference between session mode and talking directly to the server.

To use session mode, change `pool_mode = "session"`.

### Transaction mode

In transaction mode, a client talks to one server for the duration of a single transaction; once it's over, the server is returned to the pool. Prepared statements, `SET`, and advisory locks are not supported; alternatives are to use `SET LOCAL` and `pg_advisory_xact_lock` which are scoped to the transaction.

This mode is enabled by default.

### Load balancing of read queries

All queries are load balanced against the configured servers using either the random or least open connections algorithms. The most straightforward configuration example would be to put this pooler in front of several replicas and let it load balance all queries.

If the configuration includes a primary and replicas, the queries can be separated with the built-in query parser. The query parser, implemented with the `sqlparser` crate, will interpret the query and route all `SELECT` queries to a replica, while all other queries including explicit transactions will be routed to the primary.

#### **Query parser**

The query parser will do its best to determine where the query should go, but sometimes that's not possible. In that case, the client can select which server it wants using this custom SQL syntax:

```sql
-- To talk to the primary for the duration of the next transaction:
SET SERVER ROLE TO 'primary';

-- To talk to the replica for the duration of the next transaction:
SET SERVER ROLE TO 'replica';

-- Let the query parser decide
SET SERVER ROLE TO 'auto';

-- Pick any server at random
SET SERVER ROLE TO 'any';

-- Reset to default configured settings
SET SERVER ROLE TO 'default';
```

The setting will persist until it's changed again or the client disconnects.

By default, all queries are routed to the first available server; `default_role` setting controls this behavior.

#### Failover

All servers are checked with a `;` (very fast) query before being given to a client. Additionally, the server health is monitored with every client query that it processes. If the server is not reachable, it will be banned and cannot serve any more transactions for the duration of the ban. The queries are routed to the remaining servers. If all servers become banned, the ban list is cleared: this is a safety precaution against false positives. The primary can never be banned.

The ban time can be changed with `ban_time`. The default is 60 seconds.

#### Sharding

We use the `PARTITION BY HASH` hashing function, the same as used by Postgres for declarative partitioning. This allows to shard the database using Postgres partitions and place the partitions on different servers (shards). Both read and write queries can be routed to the shards using this pooler.

**Extended syntax**

To route queries to a particular shard, we use this custom SQL syntax:

```sql
-- To talk to a shard explicitly
SET SHARD TO '1';

-- To let the pooler choose based on a value
SET SHARDING KEY TO '1234';
```

The active shard will last until it's changed again or the client disconnects. By default, the queries are routed to shard 0.

For hash function implementation, see `src/sharding.rs` and `tests/sharding/partition_hash_test_setup.sql`.

**ActiveRecord/Rails**

```ruby
class User < ActiveRecord::Base
end

# Metadata will be fetched from shard 0
ActiveRecord::Base.establish_connection

# Grab a bunch of users from shard 1
User.connection.execute "SET SHARD TO '1'"
User.take(10)

# Using id as the sharding key
User.connection.execute "SET SHARDING KEY TO '1234'"
User.find_by_id(1234)

# Using geographical sharding
User.connection.execute "SET SERVER ROLE TO 'primary'"
User.connection.execute "SET SHARDING KEY TO '85'"
User.create(name: "test user", email: "test@example.com", zone_id: 85)

# Let the query parser figure out where the query should go.
# We are still on shard = hash(85) % shards.
User.connection.execute "SET SERVER ROLE TO 'auto'"
User.find_by_email("test@example.com")
```

**Raw SQL**

```sql
-- Grab a bunch of users from shard 1
SET SHARD TO '1';
SELECT * FROM users LIMT 10;

-- Find by id
SET SHARDING KEY TO '1234';
SELECT * FROM USERS WHERE id = 1234;

-- Writing in a primary/replicas configuration.
SET SHARDING ROLE TO 'primary';
SET SHARDING KEY TO '85';
INSERT INTO users (name, email, zome_id) VALUES ('test user', 'test@example.com', 85);

SET SERVER ROLE TO 'auto'; -- let the query router figure out where the query should go
SELECT * FROM users WHERE email = 'test@example.com'; -- shard setting lasts until set again; we are reading from the primary
```

**With comments**

Issuing queries to the pooler can cause additional latency. To reduce its impact, it's possible to include sharding information inside SQL comments sent via the query. This is reasonably easy to implement with ORMs like [ActiveRecord](https://api.rubyonrails.org/classes/ActiveRecord/QueryMethods.html#method-i-annotate) and [SQLAlchemy](https://docs.sqlalchemy.org/en/20/core/events.html#sql-execution-and-connection-events).

```
/* shard_id: 5 */ SELECT * FROM foo WHERE id = 1234;

/* sharding_key: 1234 */ SELECT * FROM foo WHERE id = 1234;
```

**Automatic query parsing**

PgCat can use the `sqlparser` crate to parse SQL queries and extract the sharding key. This is configurable with the `automatic_sharding_key` setting. This feature is still experimental, but it's the ideal implementation for sharding, requiring no client modifications.

#### Statistics reporting

The stats are very similar to what PgBouncer reports and the names are kept to be comparable. They are accessible by querying the admin database `pgcat`, and `pgbouncer` for compatibility.

```
psql -h 127.0.0.1 -p 6432 -d pgbouncer -c 'SHOW DATABASES'
```

Additionally, Prometheus statistics are available at `/metrics` via HTTP.

#### Live configuration reloading

The config can be reloaded by sending a `kill -s SIGHUP` to the process or by querying `RELOAD` to the admin database. All settings except the `host` and `port` can be reloaded without restarting the pooler, including sharding and replicas configurations.

#### Mirroring

Mirroring allows to route queries to multiple databases at the same time. This is useful for prewarning replicas before placing them into the active configuration, or for testing different versions of Postgres with live traffic.
