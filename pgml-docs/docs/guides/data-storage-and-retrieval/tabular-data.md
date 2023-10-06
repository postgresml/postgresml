# Tabular data

Tabular data is data stored in tables. While that's a bit of a recursive definition, tabular data is any kind for format that defines rows and columns, and is the most common type of data storage mechanism. Examples of tabular data include things like spreadsheets, database tables, CSV files, and Pandas dataframes.

Storing and accessing tabular data is a subject of decades of studies, and is the core purpose of many database systems. PostgreSQL has been leading the charge on optimal tabular storage for a while and remains today one of the most popular and effective ways to store, organize and retrieve this kind of data.

### Creating tables

Postgres makes it really easy to create and use tables. If you're looking to use PostgresML for a supervised learning project, creating a table will be very similar to a Pandas dataframe, except it will be durable and easily accessible for as long as the database exists.

For the rest of this guide, we'll take the [USA House Prices](https://www.kaggle.com/code/fatmakursun/supervised-unsupervised-learning-examples/) dataset from Kaggle, store it in Postgres and query it for basic statistics. The dataset has seven (7) columns and 5,000 rows:



| Column                       | Data type | Postgres data type |
| ---------------------------- | --------- | ------------------ |
| Avg. Area Income             | Float     | REAL               |
| Avg. Area House Age          | Float     | REAL               |
| Avg. Area Number of Rooms    | Float     | REAL               |
| Avg. Area Number of Bedrooms | Float     | REAL               |
| Area Population              | Float     | REAL               |
| Price                        | Float     | REAL               |
| Address                      | String    | VARCHAR            |

Once we know the column names and data types, the Postgres table definition almost writes itself:

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

The column names are double quoted because they contain special characters like `.` and space, which can be interpreted to be part of the SQL syntax. Generally speaking, it's good practice to double quote all entity names when using them in a PostgreSQL query, although most of the time it's not needed.

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

Right now the table is empty and that's a bit boring. Let's import the USA House Prices dataset into it using one of the easiest and fastest way to do so in Postgres: using `COPY`.

If you're like me and prefer to use the terminal, you can open up `psql` and ingest the dataset like this:

```
postgresml=# \copy usa_house_prices FROM 'USA_Housing.csv' CSV HEADER;
COPY 5000
```

As expected, Postgres copied all 5,000 rows into the `usa_house_prices` table. `COPY` accepts CSV, text, and Postgres binary formats, but CSV is definitely the most common.

You may have noticed that we used the `\copy` command in the terminal, not `COPY`. The `COPY` command actually comes in two forms: `\copy` which is a `psql` command that performs a local system to remote database server copy, and `COPY` which is more commonly used in applications. If you're writing your own application to ingest data into Postgres, you'll be using `COPY`.

### Querying data

Querying data stored in tables is what this is all about. After all, just storing data isn't particularly interesting or useful. Postgres has one of the most comprehensive and powerful querying languages of all data storage systems we've worked with so, for our example, we won't have any trouble calculating some statistics to understand our data better.

Let's compute some basic statistics on the "Avg. Area Income" column using SQL:

```sql
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

which produces exactly what we want:

```
 count |        avg        |    max    |   min    | percentile_75  |      stddev
-------+-------------------+-----------+----------+----------------+-------------------
  5000 | 68583.10897773437 | 107701.75 | 17796.63 | 75783.33984375 | 10657.99120344229
```

The SQL language is very expressive and allows to select, filter and aggregate any number of columns from any number of tables with a single query.

### Adding more data

Because databases store data in perpetuity, adding more data to Postgres can take several forms. The simplest and most commonly used way to add data is to just insert it into a table that we already have. Using the USA House Prices example, we can add a new row into the table with just one query:

```sql
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

Another way to add more data to a table is to run `COPY` again with a different CSV as the source. Many ETL pipelines from places like Snowflake or Redshift split their output into multiple CSVs, which can be individually imported into Postgres using multiple `COPY` statements.

Adding rows is pretty simple, but now that our dataset is changing, we should explore some tools to help us protect it against bad values.

### Data integrity

Databases store very important data and they were built with many safety features to protect that data from common errors. In machine learning, one of the most common errors is data duplication, i.e. having the same row appear in the a table twice. Postgres can easily protect us against this with unique indexes.

Looking at the USA House Price dataset, we can find its natural key pretty easily. Since most columns are aggregates, the only column that seems unique is the "Address". After all, there should never be more than one house at a single address, not for sale anyway.

To ensure that our dataset reflects this, let's add a unique index to our table. To do so, we can use this SQL query:

```sql
CREATE UNIQUE INDEX ON usa_house_prices USING btree("Address");
```

Postgres scans the whole table, ensures there are no duplicates in the "Address" column and creates an index on that column using the B-Tree algorithm.

If we now attempt to insert the same row again, we'll get an error:

```
ERROR:  duplicate key value violates unique constraint "usa_house_prices_Address_idx"
DETAIL:  Key ("Address")=(1 Infinite Loop, Cupertino, California) already exists.
```

Postgres supports many more indexing algorithms, namely GiST, BRIN, GIN, and Hash. Many extensions, for example `pgvector`, implement their own index types like HNSW and IVFFlat, to efficiently search and retrieve specialized values. We explore those in our guide about [Vectors](vectors.md).

### Accelerating recall

Once the dataset gets large enough, and we're talking millions of rows, it's no longer practical to query the table directly. The amount of data Postgres has to scan to return a result becomes quite large and queries become slow. To help with that, tables should have indexes that order and organize commonly accessed columns. Scanning a B-Tree index can be done in _O(log n)_ time, which is orders of magnitude faster than the _O(n)_ full table search.

#### Querying an index

Postgres automatically uses indexes when possible in order to accelerate recall. Using our example above, we can query data using the "Address" column and we can do so very quickly by using the unique index we created.

```sql
SELECT
    "Avg. Area House Age",
    "Address"
FROM usa_house_prices
WHERE "Address" = '1 Infinite Loop, Cupertino, California';
```

which produces

```
 Avg. Area House Age |                Address
---------------------+----------------------------------------
                  43 | 1 Infinite Loop, Cupertino, California
(1 row)
```

which is exactly what we expected. Since we have a unique index on the table, we should only be getting one row back with that address.

To ensure that Postgres is using an index when querying a table, we can ask it to produce the query execution plan that it's going to use before executing that query. A query plan is a list of steps that Postgres will take in order to get the query result we requested.

To get the query plan for any query, prepend the keyword `EXPLAIN` to any query you're planning on running:

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

The query plan indicates that it will be running an "Index Scan" using the index `usa_house_prices_Address_index` which is exactly what we want.

The ability to create indexes on datasets of any size and to then efficiently query that data is what separates Postgres from most ad-hoc tools like Pandas and Arrow. Postgres can store and query datasets that would never be able to fit into memory and can do so quicker and more efficiently than most database systems currently used across the industry.

#### Maintaining an index

Indexes are automatically updated when new data is added and old data is removed. Postgres automatically ensures that indexes are efficiently organized and are ACID compliant. When using Postgres tables, the system guarantees that the data will always be consistent, no matter how many concurrent changes are made to the tables.
