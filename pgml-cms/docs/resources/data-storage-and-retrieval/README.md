# Tabular data

Tabular data is data stored in tables. A table is a format that defines rows and columns, and is the most common type of data organization. Examples of tabular data are spreadsheets, database tables, CSV files, and Pandas dataframes.

Storing and accessing tabular data in an efficient manner is a subject of multiple decade-long studies, and is the core purpose of most database systems. PostgreSQL has been leading the charge on optimal tabular storage for a long time, and remains one of the most popular and effective ways to store, organize and retrieve tabular data today.

### Creating tables

Postgres makes it easy to create and use tables. If you're looking to use PostgresML for a supervised learning project, creating a table will be very similar to a Pandas dataframe, except it will be durable and accessible for as long as the database exists.

For the rest of this guide, we'll use the [USA House Prices](https://www.kaggle.com/code/fatmakursun/supervised-unsupervised-learning-examples/) dataset from Kaggle, store it in a Postgres table and run some basic queries. The dataset has seven (7) columns and 5,000 rows:

| Column                       | Data type | Postgres data type |
| ---------------------------- | --------- | ------------------ |
| Avg. Area Income             | Float     | REAL               |
| Avg. Area House Age          | Float     | REAL               |
| Avg. Area Number of Rooms    | Float     | REAL               |
| Avg. Area Number of Bedrooms | Float     | REAL               |
| Area Population              | Float     | REAL               |
| Price                        | Float     | REAL               |
| Address                      | String    | VARCHAR            |

Once we know the column names and data types, the Postgres table definition is pretty straight forward:

```plsql
CREATE TABLE usa_house_prices (
  "Avg. Area Income" REAL NOT NULL,
  "Avg. Area House Age" REAL NOT NULL,
  "Avg. Area Number of Rooms" REAL NOT NULL,
  "Avg. Area Number of Bedrooms" REAL NOT NULL,
  "Area Population" REAL NOT NULL,
  "Price" REAL NOT NULL,
  "Address" VARCHAR NOT NULL
);
```

The column names are double quoted because they contain special characters like `.` and space, which can be interpreted to be part of the SQL syntax. Generally speaking, it's good practice to double quote all entity names when using them in a query, although most of the time it's not needed.

If you run this using `psql`, you'll get something like this:

```
postgresml=# CREATE TABLE usa_house_prices (
  "Avg. Area Income" REAL NOT NULL,
  "Avg. Area House Age" REAL NOT NULL,
  "Avg. Area Number of Rooms" REAL NOT NULL,
  "Avg. Area Number of Bedrooms" REAL NOT NULL,
  "Area Population" REAL NOT NULL,
  "Price" REAL NOT NULL,
  "Address" VARCHAR NOT NULL
);
CREATE TABLE
postgresml=#
```

### Ingesting data

When created for the first time, the table is empty. Let's import our example data using one of the fastest way to do so in Postgres: with `COPY`.

If you're like me and prefer to use the terminal, you can open up `psql` and ingest the data like this:

```
postgresml=# \copy usa_house_prices FROM 'USA_Housing.csv' CSV HEADER;
COPY 5000
```

As expected, Postgres copied all 5,000 rows into the `usa_house_prices` table. `COPY` accepts CSV, text, and Postgres binary formats, but CSV is definitely the most common.

You may have noticed that we used the `\copy` command in the terminal, not `COPY`. The `COPY` command actually comes in two forms: `\copy` which is a `psql` command that copies data from system files to remote databases, while `COPY` is more commonly used in applications to send data from other sources, like standard input, files, other databases and streams.

If you're writing your own application to ingest large amounts of data into Postgres, you should use `COPY` for maximum throughput.

### Querying data

Querying data stored in tables is what makes PostgresML so powerful. Postgres has one of the most comprehensive querying languages of all databases we've worked with so, for our example, we won't have any trouble calculating some statistics:

```postgresql
SELECT
    count(*),
    avg("Avg. Area Income"),
    max("Avg. Area Income"),
    min("Avg. Area Income"),
    percentile_cont(0.75)
        WITHIN GROUP (ORDER BY "Avg. Area Income") AS percentile_75,
    stddev("Avg. Area Income")
FROM usa_house_prices;
```

```
 count |        avg        |    max    |   min    | percentile_75  |      stddev
-------+-------------------+-----------+----------+----------------+-------------------
  5000 | 68583.10897773437 | 107701.75 | 17796.63 | 75783.33984375 | 10657.99120344229
```

The SQL language is expressive and allows to select, filter and aggregate any number of columns with a single query.

### Adding more data

Because databases store data permanently, adding more data to Postgres can be done in many ways. The simplest and most common way is to just insert it into a table you already have. Using the same example dataset, we can add a new row with just one query:

```postgresql
INSERT INTO usa_house_prices (
  "Avg. Area Income",
  "Avg. Area House Age",
  "Avg. Area Number of Rooms",
  "Avg. Area Number of Bedrooms",
  "Area Population",
  "Price",
  "Address"
) VALUES (
  199778.0,
  43.0,
  3.0,
  2.0,
  57856.0,
  5000000000.0,
  '1 Infinite Loop, Cupertino, California'
);
```

If you have more CSV files you'd like to ingest, you can run `COPY` for each one. Many ETL pipelines from Snowflake or Redshift chunk their output into multiple CSVs, which can be individually imported into Postgres using `COPY`:

{% tabs %}
{% tab title="Python" %}
```python
import psycopg
from glob import glob

with psycopg.connect("postgres:///postgresml") as conn:
    cur = conn.cursor()

    with cur.copy("COPY usa_house_prices FROM STDIN CSV") as copy:
        for csv_file in glob("*.csv"):
            with open(csv_file) as f:
                next(f) # Skip header
                for line in f:
                    copy.write(line)
```
{% endtab %}

{% tab title="Bash" %}
```bash
#!/bin/bash

for f in $(ls *.csv); do
    psql postgres:///postgresml \
        -c "\copy usa_house_prices FROM '$f' CSV HEADER"
done
```
{% endtab %}
{% endtabs %}

Now that our dataset is changing, we should explore some tools to protect it against bad values.

### Data integrity

Databases store important data so they were built with many safety features in mind to protect from common errors. In machine learning, one of the most common errors is data duplication, i.e. having the same row appear in the a table twice. Postgres can protect us against this with unique indexes.

Looking at the USA House Prices dataset, we can find its natural key pretty easily. Since most columns are aggregates, the only column that seems like it should contain unique values is the "Address", i.e there should never be more than one house for sale at a single address.

To ensure that our table reflects this, let's add a unique index:

```postgresql
CREATE UNIQUE INDEX ON usa_house_prices USING btree("Address");
```

When creating a unique index, Postgres scans the whole table, checks to ensure there are no duplicates in the indexed column, and writes the column into an index using the B-Tree algorithm.

If we attempt to insert the same row again, we'll get an error:

```
ERROR:  duplicate key value violates unique constraint "usa_house_prices_Address_idx"
DETAIL:  Key ("Address")=(1 Infinite Loop, Cupertino, California) already exists.
```

Postgres supports many more indexing algorithms, e.g. GiST, BRIN, GIN, and Hash. Many extensions, e.g. `pgvector`, implement their own index types like HNSW and IVFFlat, which help efficiently search and retrieve vector values. We explore those in our guide about [Vectors](broken-reference).

### Accelerating recall

Once the dataset gets large enough, and we're talking millions of rows, it's no longer practical to query the table directly. The amount of data Postgres has to scan becomes large and queries become slow. At that point, tables should have indexes that order and organize commonly read columns. Searching an index can be done in _O(log n)_ time, which is orders of magnitude faster than the _O(n)_ full table scan.

#### Querying an index

Postgres automatically uses indexes when possible and optimal to do so. From our example, if we filter the dataset by the "Address" column, Postgres will use the index we created and return a result quickly:

```postgresql
SELECT
    "Avg. Area House Age",
    "Address"
FROM usa_house_prices
WHERE "Address" = '1 Infinite Loop, Cupertino, California';
```

```
 Avg. Area House Age |                Address
---------------------+----------------------------------------
                  43 | 1 Infinite Loop, Cupertino, California
(1 row)
```

Since we have a unique index on the table, we expect to see only one row with that address.

#### Query plan

To double check that Postgres is using an index, we can take a look at the query execution plan. A query plan is a list of steps that Postgres will take to get the result of the query. To see the query plan, prepend the keyword `EXPLAIN` to the query you'd like to run:

```
postgresml=# EXPLAIN (FORMAT JSON) SELECT
    "Avg. Area House Age",
    "Address"
FROM usa_house_prices
WHERE "Address" = '1 Infinite Loop, Cupertino, California';

                                          QUERY PLAN
----------------------------------------------------------------------------------------------
 [                                                                                           +
   {                                                                                         +
     "Plan": {                                                                               +
       "Node Type": "Index Scan",                                                            +
       "Parallel Aware": false,                                                              +
       "Async Capable": false,                                                               +
       "Scan Direction": "Forward",                                                          +
       "Index Name": "usa_house_prices_Address_idx",                                         +
       "Relation Name": "usa_house_prices",                                                  +
       "Alias": "usa_house_prices",                                                          +
       "Startup Cost": 0.28,                                                                 +
       "Total Cost": 8.30,                                                                   +
       "Plan Rows": 1,                                                                       +
       "Plan Width": 51,                                                                     +
       "Index Cond": "((\"Address\")::text = '1 Infinite Loop, Cupertino, California'::text)"+
     }                                                                                       +
   }                                                                                         +
 ]
```

The plan indicates that it will use an "Index Scan" on `usa_house_prices_Address_index` which is what we're expecting. Using `EXPLAIN` doesn't actually run the query, so it's safe to use on production systems.

The ability to create indexes on datasets of any size, and to efficiently query that data using them, is what separates Postgres from most ad-hoc tools like Pandas and Arrow. Postgres can store and query data that would never fit in memory, and it can do that quicker and more efficiently than most other databases used in the industry.

#### Maintaining an index

Postgres indexes require no special maintenance. They are automatically updated when data is added and removed. Postgres also ensures that indexes are efficiently organized and are ACID compliant: the database guarantees that the data is always consistent, no matter how many concurrent changes are made.
