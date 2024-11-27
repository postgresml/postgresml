# Distributed Training

Depending on the size of your dataset and its change frequency, you may want to offload training (or inference) to secondary PostgreSQL servers to avoid excessive load on your primary. We've outlined three of the built-in mechanisms to help distribute the load.

## pg_dump (< 10GB)

`pg_dump` is a [standard tool](https://www.postgresql.org/docs/12/app-pgdump.html) used to export data from a PostgreSQL database. If your dataset is small (e.g. less than 10GB) and changes infrequently, this could be quickest and simplest way to do it.

!!! example

```
# Export data from your production DB
pg_dump \
    postgres://username:password@production-database.example.com/production_db \
    --no-owner \
    -t table_one \
    -t table_two > dump.sql

# Import the data into PostgresML
psql \
    postgres://username:password@postgresml.example.com/postgresml_db \
    -f dump.sql
```

If you're using our <a href="/docs/guides/setup/quick_start_with_docker">Docker</a> stack, you can import the data there:</p>

```
psql \
    postgres://postgres@localhost:5433/pgml_development \
    -f dump.sql
```

!!!

PostgresML tables and functions are located in the `pgml` schema, so you can safely import your data into PostgresML without conflicts. You can also use `pg_dump` to copy the `pgml` schema to other servers which will make the trained models available in a distributed fashion.


## Foreign Data Wrappers (10GB - 100GB)

Foreign Data Wrappers, or [FDWs](https://www.postgresql.org/docs/12/postgres-fdw.html) for short, are another good tool for reading or importing data from another PostgreSQL database into PostgresML.

Setting up FDWs is a bit more involved than `pg_dump` but they provide real time access to your production data and are good for small to medium size datasets (e.g. 10GB to 100GB) that change frequently.

Official PostgreSQL [docs](https://www.postgresql.org/docs/12/postgres-fdw.html) explain FDWs with more detail; we'll document a basic example below.

### Install the extension

PostgreSQL comes with `postgres_fdw` already available, but the extension needs to be explicitly installed into the database. Connect to your PostgresML database as a superuser and run:

```postgresql
CREATE EXTENSION postgres_fdw;
```

### Create foreign server

A foreign server is a FDW reference to another PostgreSQL database running somewhere else. In this case, that foreign server is your production database.

```postgresql
CREATE SERVER your_production_db
    FOREIGN DATA WRAPPER postgres_fdw
    OPTIONS (
        host 'production-database.example.com',
        port '5432',
        dbname 'production_db'
    );
```

### Create user mapping

A user mapping is a relationship between the user you're connecting with to PostgresML and a user that exists on your production database. FDW will use
this mapping to talk to your database when it wants to read some data.

```postgresql
CREATE USER MAPPING FOR pgml_user
    SERVER your_production_db
    OPTIONS (
        user 'your_production_db_user',
        password 'your_production_db_user_password'
    );
```

At this point, when you connect to PostgresML using the example `pgml_user` and then query data in your production database using FDW, it'll use the user `your_production_db_user`
to connect to your DB and fetch the data. Make sure that `your_production_db_user` has `SELECT` permissions on the tables you want to query and the `USAGE` permissions on the schema.

### Import the tables

The final step is import your production database tables into PostgresML by creating a foreign schema mapping. This mapping will tell PostgresML which tables are available in your database. The quickest way is to import all of them, like so:

```postgresql
IMPORT FOREIGN SCHEMA public
FROM SERVER your_production_db
INTO public;
```

This will import all tables from your production DB `public` schema into the `public` schema in PostgresML. The tables are now available for querying in PostgresML.

### Usage

PostgresML snapshots the data before training on it, so every time you run `pgml.train` with a `relation_name` argument, the data will be fetched from the foreign data wrapper and imported into PostgresML.

FDWs are reasonably good at fetching only the data specified by the `VIEW`, so if you place sufficient limits on your dataset in the `CREATE VIEW` statement, e.g. train on the last two weeks of data, or something similar, FDWs will do its best to fetch only the last two weeks of data in an efficient manner, leaving the rest behind on the primary.


## Logical replication (100GB - 10TB)

Logical replication is a [replication mechanism](https://www.postgresql.org/docs/12/logical-replication.html) that's been available since PostgreSQL 10. It allows to copy entire tables and schemas from any database into PostgresML and keeping them up-to-date in real time fairly cheaply as the data in production changes. This is suitable for medium to large PostgreSQL deployments (e.g. 100GB - 10TB).

Logical replication is designed as a pub/sub system, where your production database is the publisher and PostgresML is the subscriber. As data in your database changes, it is streamed into PostgresML in milliseconds, which is very similar to how Postgres streaming replication works as well.

The setup is slightly more involved than Foreign Data Wrappers, and is documented below. All queries must be run as a superuser.

### WAL

First, make sure that your production DB has logical replication enabled. For this, it has to be on PostgreSQL 10 or above and also have `wal_level` configuration set to `logical`.

```
pgml# SHOW wal_level;
 wal_level 
-----------
 logical
(1 row) 
```

If this is not the case, you'll need to change it and restart the server.

### Publication

The [publication](https://www.postgresql.org/docs/12/sql-createpublication.html) is created on your production DB and configures which tables are replicated using logical replication. To replicate all tables in your `public` schema, you can run this:

```postgresql
CREATE PUBLICATION all_tables
FOR ALL TABLES;
```

### Schema

Logical replication does not copy the schema, so it needs to be copied manually in advance; `pg_dump` is great for this:

```bash
# Dump the schema from your production DB
pg_dump \
    postgres://username:password@production-db.example.com/production_db \
    --schema-only \
    --no-owner > schema.sql

# Import the schema in PostgresML
psql \
    postgres://username:password@postgresml.example.com/postgresml_db \
    -f schema.sql
```


### Subscription

The [subscription](https://www.postgresql.org/docs/12/sql-createsubscription.html) is created in your PostgresML database. To replicate all the tables we marked in the previous step, run:

```postgresql
CREATE SUBSCRIPTION all_tables
CONNECTION 'postgres://superuser:password@production-database.example.com/production_db'
PUBLICATION all_tables;
```

As soon as you run this, logical replication will begin. It will start by copying all the data from your production database into PostgresML. That will take a while, depending on database size, network connection and hardware performance. Each table will be copied individually and the process is parallelized.

Once the copy is complete, logical replication will synchronize and will replicate the data from your production database into PostgresML in real-time.

### Schema changes

Logical replication has one notable limitation: it does not replicate schema (table) changes. If you change a table in your production DB in an incompatible way, e.g. by adding a column, the replication will break.

To remediate this, when you're performing the schema change, make the change first in PostgresML and then in your production database.


## Native installation (10TB and beyond)

For databases that are very large, e.g. 10TB+, we recommend you install the extension directly into your database.

This option is available for databases of all sizes, but we recognize that many small to medium databases run on managed services, e.g. RDS, which don't allow this mechanism.
