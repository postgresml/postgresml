# Importing data

PostgresML can easily ingest data from your existing data stores. Depending on how frequently your data changes, different methodologies are preferable.

### Static data

Data that changes infrequently can be easily imported into PostgresML using `COPY`. All you have to do is export your data as a CSV file, create a table in Postgres to store it, and import it using the command line.

Let's use a simple CSV file with 3 columns as an example:

| Column           | Data type | Example |
| ---------------- | --------- | ------- |
| name             | text      | John    |
| age              | integer   | 30      |
| is\_paying\_user | boolean   | true    |

#### Export data as CSV

If you're using a Postgres database already, you can export any table as CSV with just one command:

```bash
psql -c "\copy your_table TO '~/Desktop/your_table.csv' CSV HEADER"
```

If you're using another  data store, it should almost always provide a CSV export functionality, since CSV is the most commonly used data format in machine learning.

#### Create table in Postgres

Creating a table in Postgres with the correct schema is as easy as:

```
CREATE TABLE your_table (
  name TEXT,
  age INTEGER,
  is_paying_user BOOLEAN
);
```

#### Import data using the command line

Once you have a table and your data exported as CSV, importing it can also be done with just one command:

```bash
psql -c "\copy your_table FROM '~/Desktop/your_table.csv' CSV HEADER"
```

We took our export command and changed `TO` to `FROM`, and that's it. Make sure you're connecting to your PostgresML database when importing data.

#### Refreshing data

If your data changed, repeat this process again. To avoid duplicate entries in your table, you can truncate (or delete) all rows beforehand:

```
TRUNCATE your_table;
```

### Live data

Importing data from online databases can be done with foreign data wrappers. All PostgresML databases come with both `postgres_fdw` and `dblink` extensions pre-installed, so you can import data from any of your existing Postgres databases, and export machine learning artifacts from PostgresML using just a few lines of SQL.

#### Setting up

Before you get started with foreign data wrappers, log into your current database hosting provider and grab the following connection details:

* Host
* Port (typically `5432`)
* Database name
* Postgres user
* Postgres password

Once you have them, we can setup our live foreign data wrapper connection. All following commands should be executed on your PostgesML database. You don't need to perform any additional steps on your production database.

#### Connecting

To connect to your database from PostgresML, first create a corresponding `SERVER`:

```
CREATE SERVER live_db
FOREIGN DATA WRAPPER postgres_fdw
OPTIONS (
  host 'Host'
  port 'Port'
  dbname 'Database name'
);
```

Replace `Host`, `Port` and `Database name` with details you've collected in the previous step.

Once you have a `SERVER`, let's authenticate to your database:

```
CREATE USER MAPPING
FOR CURRENT_USER
SERVER live_db
OPTIONS (
  user 'Postgres user'
  password 'Postgres password'
);
```

Replace `Postgres user` and `Postgres password` with details collected in the previous step. If everything went well, we'll be able to validate that everything is working with just one query:

```
SELECT * FROM dblink(
  'live_db',
  'SELECT 1 AS one'
) AS t1(one INTEGER);
```

You can now execute any query you want on your live database from inside your PostgresML database.

#### Working with your tables

Instead of creating temporary tables for each query, you can import your entire schema into PostgresML using foreign data wrappers:

```
CREATE SCHEMA live_db_tables;

IMPORT FOREIGN SCHEMA public
FROM SERVER live_db
INTO live_db_tables;
```

All your tables from your `public` schema are now available in the `live_db_tables` schema. You can read and write to those tables as if they were hosted in PostgresML. For example, if you have a table called `users`, you could access it with:

```
SELECT * FROM live_db_tables.users LIMIT 1;
```

That's it, your PostgresML database is directly connected to your production database and you can start your machine learning journey.

#### Accelerating bulk access

To speed up access to your data, you can cache it in PostgresML by copying it from a foreign table into a regular table. Taking the example of the `users` table:

```
CREATE TABLE public.users (LIKE live_db_tables.users);
INSERT INTO public.users SELECT * FROM live_db_tables.users;
```

This will copy all rows from your `users` table into PostgresML. You'll be able to access them much quicker if you need to perform a batch job like generating embeddings or training a supervised model.

#### Exporting ML artifacts

If you want to export some artifacts you've created with PostresML to your live database, you can do so with foreign data wrappers as well. Simply copy them using the same mechanism as above, except instead of copying data from the foreign schema, copy data into the foreign schema from the regular table.
