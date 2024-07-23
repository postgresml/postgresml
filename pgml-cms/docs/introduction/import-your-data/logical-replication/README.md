---
description: Stream data from your primary database to PostgresML in real time using logical replication.
---

# Logical replication

Logical replication allows your PostgresML database to copy data from your primary database to PostgresML in real time. As soon as your customers make changes to their data on your website, those changes will become available in PostgresML.

<figure class="my-3 py-3"><img src="../../../.gitbook/assets/Getting-Started_Logical-Replication-Diagram.svg" alt="Logical replication" width="80%"><figcaption></figcaption></figure>

## Getting started

Setting up & maintaining logical replication requires a few steps, but once you're done, you'll be able to generate embeddings, train models & generate text using LLMs directly using your production data.

### Configure your primary database

First things first, make sure your primary database is configured to support logical replication. To do so, make sure the following settings are set:

| Setting                 | Value          |
|-------------------------|----------------|
| `wal_level`             | `logical`      |
| `wal_senders`           | Greater than 0 |
| `max_replication_slots` | Greater than 0 |
| `rds.logical_replication` (only on AWS RDS) | `1` |

Make sure to **restart your database** after changing any of these settings.

### Check connectivity

All PostgresML databases are allowed to connect to any other database through the Internet by default. You can test connectivity to your database from PostgresML by using the `dblink` extension:

```postgresql
SELECT
	dblink(
		'postgres://user:password@your-production-db.amazonaws.com:5432/production_db',
		'SELECT 1 AS one'
) AS t1(one integer);

```

### Start replicating

Logical replication works like a pub/sub system: your primary database decides which tables it would like to publish, and PostgresML subscribes to those changes and downloads them as they are made.

#### Create a publication

A publication is a set of tables that your primary database would like to share with your PostgresML database. To create a publication, connect to your primary database as a superuser and create the publication for your tables of choice:

```postgresql
CREATE PUBLICATION postgresml
FOR TABLE your_list_of_tables;
```

where `your_list_of_tables` are the tables you'd like to replicate. For example, if you have two tables, _users_ and _blog_posts_, you can create a publication for those two tables using this command:

{% tabs %}
{% tab title="SQL" %}

```postgresql
CREATE PUBLICATION postgresml_users
FOR TABLE users, blog_posts;
```

{% endtab %}

{% tab title="Output" %}

```
CREATE PUBLICATION
```

{% endtab %}
{% endtabs %}

#### Subscribe to changes

Now that we have a list of tables we want to replicate, we need to make sure those tables exist in your PostgresML database. Logical replication only sends over the data, without knowing anything else about your databases. Therefore, we need to make sure both the tables in your primary database and in your PostgresML databases match.

You can get the schema for your tables either by using a PostgreSQL client like pgAdmin or, more easily, by using _pg_dump_ and then importing it into PostgresML using _psql_:

{% tabs %}
{% tab title="Export the schema" %}

```bash
pg_dump \
	postgres://user:password@yyour-production-db.amazonaws.com:5432/prodution_db \
	--schema-only \
	--no-owner \
	--no-privileges \
	-t users \
	-t blog_posts \
> schema.sql
```

{% endtab %}
{% tab title="Import the schema" %}

```bash
psql \
	postgres://user:password@db.cloud.postgresml.org:6432/your_postgresml_database \
	-f schema.sql
```

{% endtab %}
{% endtabs %}

Once you have the tables created, we can start replicating data:

{% tabs %}
{% tab title="SQL" %}

```postgresql
CREATE SUBSCRIPTION postgresml
CONNECTION 'postgres://user:password@your-production-db.amazonaws.com:5432/prodution_db'
PUBLICATION postgresml;
```

{% endtab %}
{% tab title="Output" %}

```
CREATE SUBSCRIPTION
```

{% endtab %}
{% endtabs %}

As soon you run this command, the PostgresML database will create a connection to your production database and copy the data from your tables into your PostgresML tables. Once that's done, the replication will start in real time and individual changes will be sent one row at a time.
