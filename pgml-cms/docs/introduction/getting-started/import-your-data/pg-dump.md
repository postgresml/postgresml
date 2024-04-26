---
description: Migrate your PostgreSQL database to PostgresML using pg_dump.
---

# Migrate with pg_dump

_pg_dump_ is a command-line PostgreSQL tool that can move data between PostgreSQL databases. If you're planning a migration from your database to PostgresML, _pg_dump_ is a good tool to get you going quickly.

## Getting started

If your database is reasonably small (10 GB or less), you can just run _pg_dump_ in one command:

{% tabs %}
{% tab title="pg_dump" %}

```bash
pg_dump \
	--no-owner \
	--clean \
	--no-privileges \
  postgres://user:password@your-production-database.amazonaws.com/production_db | \
psql postgres://user:password@sql.cloud.postgresml.org:6432/your_pgml_db
```

{% endtab %}
{% endtabs %}

This will take a few minutes, and once the command completes, all your data, including indexes, will be in your PostgresML database.

## Migrating one table at a time

If your database is larger, you can split the migration into multiple steps, migrating one or more tables at a time.

{% tabs %}
{% tab title="pg_dump" %}

```bash
pg_dump \
	--no-owner \
	--clean \
	--no-privileges \
	-t users \
	-t orders \
  postgres://user:password@your-production-database.amazonaws.com/production_db | \
psql postgres://user:password@sql.cloud.postgresml.org:6432/your_pgml_db
```

{% endtab %}
{% endtabs %}
