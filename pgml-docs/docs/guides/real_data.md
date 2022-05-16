# Real data

Our previous guides covered how to use PostgresML with toy datasets. This section will cover different ways to use real production data. Depending on the size of your dataset and its update frequency, some methods below could be better than others.

## pg_dump

`pg_dump` is a [standard tool](https://www.postgresql.org/docs/12/app-pgdump.html) to export data from a PostgreSQL database. If your dataset is small (e.g. less than 10GB) and changes infrequently, this could be quickest and simplest way to do it.

!!! example
	```bash
	# Export data from your production DB
	pg_dump \
		postgres://username:password@production-database.example.com/production_db \
		-t table_one \
		-t table_two > dump.sql

	# Import the data into PostgresML
	psql \
		postgres://username:password@postgresml.example.com/postgresml_db \
		-f dump.sql
	```

PostgresML tables and functions are located in the `pgml` schema, so you can safely import your data into PostgresML without conflicts.


## Foreign Data Wrappers

Foreign Data Wrappers, or [FDWs](https://www.postgresql.org/docs/12/postgres-fdw.html) for short, are another good tool for reading or importing data from another PostgreSQL database into PostgresML.

Setting up FDWs is a bit more involved than `pg_dump` but they provide real-time access to your production data and are good for small to medium size datasets (e.g. 10GB to 100GB) that change frequently.

How to setup FDWs is documented in PostgreSQL docs more; below we'll document a basic example.

### Install the extension

PostgresML comes with `postgres_fdw` installed, but the extension needs to be explicitely installed into the database. Connect to PostgresML and run:

```sql
CREATE EXTENSION postgres_fdw;
```

### Create foreign server

A foreign server is a FDW reference to another PostgreSQL database running somewhere else. In this case, that foreign server is your existing production database.

```sql
CREATE SERVER your_production_db
	FOREIGN DATA WRAPPER postgres_fdw
	OPTIONS (
		host 'production-database.example.com',
		port '5432',
		dbname 'production_db'
	);
```

### Create user mapping

A user mapping is a relationship between the user you're connecting with to PostgresML and a user that exists on your production database. The FDW will use
this mapping to reach out to your production database when it wants to read some data.

```sql
CREATE USER MAPPING FOR pgml_user
	SERVER your_production_db
	OPTIONS (
		user 'your_production_db_user',
		password 'your_production_db_user_password'
	);
```

At this point, when you connect to PostgresML using the example `pgml_user` and then query data in your production database using the FDW, it'll use the user `your_production_db_user`
to connect to your production database and fetch the data.

### Import the tables

The final step is map your production database tables into PostgresML using the FDW. This mapping will tell PostgresML which tables are available in your database. The quickest way is to import all of them, like so:

```sql
IMPORT FOREIGN SCHEMA 'public'
FROM SERVER your_production_db
INTO 'public';
```

This will import all tables from your production DB `public` schema into the `public` schema in PostgresML. The tables are now available for querying in PostgresML.

### Usage

PostgresML snapshots the data before training on it, so every time you run `pgml.train`, the data will be fetched from the foreign data wrapper and imported into PostgresML. FDWs are reasonably good at fetching only the data specified by the `VIEW`, so if you place sufficient limits on your dataset in the `CREATE VIEW` statement, e.g. train on the last two weeks of data, or something similar, FDWs will do its best to fetch only the last two weeks of data from your database into PostgresML which should be reasonably quick.


## Logical replication

Logical replication is a database replication tool that's been available since PostgreSQL 10. Logical replication allows to copy entire tables and schemas from any database into PostgresML and keeping them up-to-date fairly cheaply as the data in production changes. This is suitable for medium to large PostgreSQL deployments (e.g. 100GB - 10TB).
