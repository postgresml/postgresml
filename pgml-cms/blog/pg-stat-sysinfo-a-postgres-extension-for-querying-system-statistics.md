---
description: Introducing a Postgres extension which collects system statistics
---

# PG Stat Sysinfo, a Postgres Extension for Querying System Statistics

<div align="left">

<figure><img src=".gitbook/assets/jason.jpg" alt="Author" width="125"><figcaption></figcaption></figure>

</div>

Jason Dusek

May 8, 2023

What if we could query system statistics relationally? Many tools that present system and filesystem information -- tools like `ls`, `ss`, `ps` and `df` -- present it in a tabular format; a natural next step is to consider working on this data with a query language adapted to tabular structures.

Our recently released [`pg_stat_sysinfo`](https://github.com/postgresml/pg\_stat\_sysinfo) provides common system metrics as a Postgres virtual table. This allows us to collect metrics using the Postgres protocol. For dedicated database servers, this is one of the simplest ways to monitor the database server's available disk space, use of RAM and CPU, and load average. For systems running containers, applications and background jobs, using a Postgres as a sort of monitoring agent is not without some benefits, since Postgres itself is low overhead when used with few clients, is quite stable, and offers secure and well-established connection protocols, libraries, and command-line tools with remote capability.

A SQL interface to system data is not a new idea. Facebook's [OSQuery](https://www.osquery.io) is widely used, and the project is now homed under the Linux foundation and has a plugin ecosystem with contributions from a number of companies. The idea seems to work out well in practice as well as in theory.

Our project is very different from OSQuery architecturally, in that the underlying SQL engine is a relational database server, rather than an embedded database. OSQuery is built on SQLite, so connectivity or forwarding and continuous monitoring must both be handled as extensions of the core.

The `pg_stat_sysinfo` extension is built with [PGRX](https://github.com/tcdi/pgrx). It can be used in one of two ways:

* The collector function can be called whenever the user wants system statistics: `SELECT * FROM pg_stat_sysinfo_collect()`
* The collector can be run in the background as a Postgres worker. It will cache about 1MiB of metrics -- about an hour in common cases -- and these can be batch collected by some other process. (Please see "Enable Caching Collector" in the [README](https://github.com/postgresml/pg\_stat\_sysinfo#readme) to learn more about how to do this.)

The way `pg_stat_sysinfo` is meant to be used, is that the caching collector is turned on, and every minute or so, something connects with a standard Postgres connection and collects new statistics, augmenting the metadata with information like the node's ID, region or datacenter, role, and so forth. Since `pg_stat_sysinfo` is just a Postgres extension, it implements caching using standard Postgres facilities -- in this case, a background worker and Postgres shared memory. Because we expect different environments to differ radically in the nature of metadata that they store, all metrics are stored in a uniform way, with metadata pushed into a `dimensions` column. These are both real differences from OSQuery, and are reflective of a different approach to design questions that everyone confronts when putting together a tool for collecting system metrics.

## Data & Dimensions

The `pg_stat_sysinfo` utility stores metrics in a streamlined, generic way. The main query interface, a view called `pg_stat_sysinfo`, has four columns:

!!! generic

!!! code\_block

```
\d pg_stat_sysinfo
```

!!!

!!! results

| Column     | Type                     | Collation | Nullable | Default |
| ---------- | ------------------------ | --------- | -------- | ------- |
| metric     | text                     |           |          |         |
| dimensions | jsonb                    |           |          |         |
| at         | timestamp with time zone |           |          |         |
| value      | double precision         |           |          |         |

!!!

!!!

All system statistics are stored together in this one structure.

!!! generic

!!! code\_block

```postgresql
SELECT * FROM pg_stat_sysinfo
 WHERE metric = 'load_average'
   AND at BETWEEN '2023-04-07 19:20:09.3'
              AND '2023-04-07 19:20:11.4';
```

!!!

!!! results

| metric        | dimensions          | at                            | value         |
| ------------- | ------------------- | ----------------------------- | ------------- |
| load\_average | {"duration": "1m"}  | 2023-04-07 19:20:11.313138+00 | 1.88330078125 |
| load\_average | {"duration": "5m"}  | 2023-04-07 19:20:11.313138+00 | 1.77587890625 |
| load\_average | {"duration": "15m"} | 2023-04-07 19:20:11.313138+00 | 1.65966796875 |
| load\_average | {"duration": "1m"}  | 2023-04-07 19:20:10.312308+00 | 1.88330078125 |
| load\_average | {"duration": "5m"}  | 2023-04-07 19:20:10.312308+00 | 1.77587890625 |
| load\_average | {"duration": "15m"} | 2023-04-07 19:20:10.312308+00 | 1.65966796875 |
| load\_average | {"duration": "1m"}  | 2023-04-07 19:20:09.311474+00 | 1.88330078125 |
| load\_average | {"duration": "5m"}  | 2023-04-07 19:20:09.311474+00 | 1.77587890625 |
| load\_average | {"duration": "15m"} | 2023-04-07 19:20:09.311474+00 | 1.65966796875 |

!!!

!!!

However, there is more than one way to do this.

One question that naturally arises with metrics is what metadata to record about them. One can of course name them -- `fs_bytes_available`, `cpu_usage`, `load_average` -- but what if that's the only metadata that we have? Since there is more than one load average, we might find ourself with many similarly named metrics: `load_average:1m`, `load_average:5m`, `load_average:15m`.

In the case of the load average, we could handle this situation by having a table with columns for each of the similarly named metrics:

!!! code\_block

```postgresql
CREATE TABLE load_average (
    at          timestamptz NOT NULL DEFAULT now(),
    "1m"        float4 NOT NULL,
    "5m"        float4 NOT NULL,
    "15m"       float4 NOT NULL
);
```

!!!

This structure is fine for `load_average` but wouldn't work for CPU, disk, RAM or other metrics. This has at least one disadvantage, in that we need to write queries that are structurally different, for each metric we are working with; but another disadvantage is revealed when we consider consolidating the data for several systems altogether. Each system is generally associated with a node ID (like the instance ID on AWS), a region or data center, maybe a profile or function (bastion host, database master, database replica), and other metadata. Should the consolidated tables have a different structure than the ones used on the nodes? Something like the following?

!!! code\_block

```postgresql
CREATE TABLE load_average (
    at          timestamptz NOT NULL DEFAULT now(),
    "1m"        float4 NOT NULL,
    "5m"        float4 NOT NULL,
    "15m"       float4 NOT NULL,
    node        text NOT NULL,
    -- ...and so on...
    datacenter  text NOT NULL
);
```

!!!

This has the disadvantage of baking in a lot of keys and the overall structure of someone's environment; it makes it harder to reuse the system and makes it tough to work with the data as a system evolves. What if we put the keys into a key-value column type?

!!! generic

!!! code\_block

```postgresql
CREATE TABLE load_average (
    at          timestamptz NOT NULL DEFAULT now(),
    "1m"        float4 NOT NULL,
    "5m"        float4 NOT NULL,
    "15m"       float4 NOT NULL,
    metadata    jsonb NOT NULL DEFAULT '{}'
);
```

!!!

!!! results

| at                            | metadata            | value         |
| ----------------------------- | ------------------- | ------------- |
| 2023-04-07 19:20:11.313138+00 | {"duration": "1m"}  | 1.88330078125 |
| 2023-04-07 19:20:11.313138+00 | {"duration": "5m"}  | 1.77587890625 |
| 2023-04-07 19:20:11.313138+00 | {"duration": "15m"} | 1.65966796875 |
| 2023-04-07 19:20:10.312308+00 | {"duration": "1m"}  | 1.88330078125 |
| 2023-04-07 19:20:10.312308+00 | {"duration": "5m"}  | 1.77587890625 |
| 2023-04-07 19:20:10.312308+00 | {"duration": "15m"} | 1.65966796875 |
| 2023-04-07 19:20:09.311474+00 | {"duration": "1m"}  | 1.88330078125 |
| 2023-04-07 19:20:09.311474+00 | {"duration": "5m"}  | 1.77587890625 |
| 2023-04-07 19:20:09.311474+00 | {"duration": "15m"} | 1.65966796875 |

!!!

!!!

This works pretty well for most metadata. We'd store keys like `"node": "i-22121312"` and `"region": "us-atlantic"` in the metadata column. Postgres can index JSON columns so queries can be reasonably efficient; and the JSON query syntax is not so difficult to work with. What if we moved the `"1m"`, `"5m"`, \&c into the metadata as well? Then we'd end up with three rows for every measurement of the load average:

Now if we had a name column, we could store really any floating point metric in the same table. This is basically what `pg_stat_sysinfo` does, adopting the terminology and method of "dimensions", common to many cloud monitoring solutions.

## Caching Metrics in Shared Memory

Once you can query system statistics, you need to find a way to view them for several systems all at once. One common approach is store and forward -- the system on which metrics are being collected runs the collector at regular intervals, caches them, and periodically pushes them to a central store. Another approache is simply to have the collector gather the metrics and then something comes along to pull the metrics into the store. This latter approach is relatively easy to implement with `pg_stat_sysinfo`, since the data can be collected over a Postgres connection. In order to get this to work right, though, we need a cache somewhere -- and it needs to be somewhere that more than one process can see, since each Postgres connection is a separate process.

The cache can be enabled per the section "Enable Caching Collector" in the [README](https://github.com/postgresml/pg\_stat\_sysinfo#readme). What happens when it's enabled? Postgres starts a [background worker](https://www.postgresql.org/docs/current/bgworker.html) that writes metrics into a shared memory ring buffer. Sharing values between processes -- connections, workers, the Postmaster -- is something Postgres does for other reasons so the server programming interface provides shared memory utilities, which we make use of by way of PGRX.

The [cache](https://github.com/postgresml/pg\_stat\_sysinfo/blob/main/src/shmem\_ring\_buffer.rs) is a large buffer behind a lock. The background worker takes a write lock and adds statistics to the end of the buffer, rotating the buffer if it's getting close to the end. This part of the system wasn't too tricky to write; but it was a little tricky to understand how to do this correctly. An examination of the code reveals that we actually serialize the statistics into the buffer -- why do we do that? Well, if we write a complex structure into the buffer, it may very well contain pointers to something in the heap of our process -- stuff that is in scope for our process but that is not in the shared memory segment. This actually would not be a problem if we were reading data from within the process that wrote it; but these pointers would not resolve to the right thing if read from another process, like one backing a connection, that is trying to read the cache. An alternative would be to have some kind of Postgres-shared-memory allocator.

## The Extension in Practice

There are some open questions around collecting and presenting the full range of system data -- we don't presently store complete process listings, for example, or similarly large listings. Introducing these kinds of "inventory" or "manifest" data types might lead to a new table.

Nevertheless, the present functionality has allowed us to collect fundamental metrics -- disk usage, compute and memory usage -- at fine grain and very low cost.
