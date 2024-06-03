---
description: Move data into PostgresML from data files using COPY and CSV.
---

# Move data with COPY

Data that changes infrequently can be easily imported into PostgresML (and any other Postgres database) using `COPY`. All you have to do is export your data as a file, create a table in Postgres to store it, and import it using the command line (or your IDE of choice).

## Getting started

We'll be using CSV as our data format of choice. CSV is a supported mechanism for data transport in pretty much every database and system in existence, so you won't have any trouble finding the CSV export functionality in your current data store.

Let's use a simple CSV file with 3 columns as an example:

| Column           | Data type | Example data |
| ---------------- | --------- | ------- |
| name             | text      | John    |
| age              | integer   | 30      |
| is\_paying\_user | boolean   | true    |

### Export data

If you're using a Postgres database already, you can export any table as CSV with just one command:

```bash
psql \
  postgres://user:password@your-production-db.amazonaws.com \
  -c "\copy (SELECT * FROM users) TO '~/users.csv' CSV HEADER"
```

If you're using another data store, it will almost always provide a CSV export functionality.

### Create table in PostgresML

Create a table in PostgresML with the correct schema:

{% tabs %}
{% tab title="SQL" %}

```postgresql
CREATE TABLE users(
  name TEXT,
  age INTEGER,
  is_paying_user BOOLEAN
);
```

{% endtab %}
{% tab title="Output" %}

```
CREATE TABLE
```

{% endtab %}
{% endtabs %}

Data types should roughly match to what you have in your CSV file. If the data type is not known, you can always use `TEXT` and figure out what it is later with a few queries. Postgres also supports converting data types, as long as they are formatted correctly.

### Import data

Once you have a table and your data exported as CSV, importing it can also be done with just one command:

```bash
psql \
 postgres://user:password@sql.cloud.postgresml.org/your_pgml_database \
 -c "\copy your_table FROM '~/your_table.csv' CSV HEADER"
```

We took our export command and changed `TO` to `FROM`, and that's it. Make sure you're connecting to your PostgresML database when importing data.

## Refresh data

If your data changed, repeat this process again. To avoid duplicate entries in your table, you can truncate (or delete) all rows beforehand:

```postgresql
TRUNCATE your_table;
```
