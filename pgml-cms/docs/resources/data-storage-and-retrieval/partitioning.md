# Partitioning

Partitioning is splitting Posgres tables into multiple smaller tables, with the intention of querying each smaller table independently. This is useful and sometimes necessary when tables get so large that accessing them becomes too slow. Partitioning requires detailed knowledge of the dataset and uses that knowledge to help Postgres execute queries faster.

### Partitioning schemes

Postgres supports three (3) kinds of partition schemes: by range, by hash, and by list. Each scheme is appropriate for different use cases, and choosing the right scheme is important to get the best performance out of your data.

### Partition by range

Partition by range operates on numerical values. Dates, numbers and vectors can be used as range partition keys because their range of values can be split into non-overlapping parts.

For example, if we have a table with a date column (`TIMESTAMPTZ`, a date and time with timezone information), we could create three (3) partitions with the following bounds:

* partition 1 will contain all dates prior to January 1, 2000,
* partition 2 will contain all dates between January 1, 2000 and January 1, 2021,
* partition 3 will contain all dates after January 1, 2021.

While these ranges are not even, we chose them because of some knowledge we have about our dataset. In our hypothetical example, we know that these date ranges will split our dataset into roughly three (3) evenly sized tables.

#### Building partitions

Let's build some real partitions with a dataset from Kaggle: [Hourly Energy Consumption](https://www.kaggle.com/datasets/robikscube/hourly-energy-consumption).

In Postgres, you can create a partition by range with just a few queries. Partitioning requires creating two types of tables: the parent table which defines the partitioning scheme, and the child tables which define the ranges and store the actual data.

Let's start with the parent table:

```postgresql
CREATE TABLE energy_consumption (
    "Datetime" TIMESTAMPTZ,
    "AEP_MW" REAL
) PARTITION BY RANGE("Datetime");
```

Now, let's add a couple child tables:

```postgresql
CREATE TABLE energy_consumption_2004_2011
PARTITION OF energy_consumption
FOR VALUES FROM ('2004-01-01') TO ('2011-12-31');

CREATE TABLE energy_consumption_2012_2018 
PARTITION OF energy_consumption 
FOR VALUES FROM ('2011-12-31') TO ('2018-12-31');
```

Postgres partition bounds are defined as `[start, end)`, which means the start of the range is included and the end of the range is excluded.

Let's ingest the dataset into our partitioned table and see what we get:

```
postgresml=# \copy energy_consumption FROM 'AEP_hourly.csv' CSV HEADER;
COPY 121273
```

We have a grand total of 121,273 rows. If we partitioned the dataset correctly, the two child tables should have roughly the same number of rows:

```
postgresml=# SELECT count(*) FROM energy_consumption_2004_2011;
 count 
-------
 63511
 
 postgresml=# SELECT count(*) FROM energy_consumption_2012_2018;
 count 
-------
 57762
```

Nicely done. The two table counts are pretty close, which creates a roughly even distribution of data in our partitioning scheme.

Postgres allows to query each partition individually, which is nice if we know what the range specification is. While this works in this example, in a living dataset, we would need continue adding partitions to include more values. If we wanted to store dates for the years 2019 through 2023, for example, we would need to make at least one more child table.

To make reading this data user-friendly, Postgres allows us to query the parent table instead. As long as we specify the partition key, we are guaranteed to get the most efficient query plan possible:

```postgresql
SELECT
    avg("AEP_MW")
FROM energy_consumption
WHERE "Datetime" BETWEEN '2004-01-01' AND '2005-01-01';
```

```
        avg         
--------------------
 15175.689170820118
```

If we look at the query plan, we'll see that Postgres only queries one of the child tables we created:

```
postgresml=# EXPLAIN SELECT
    avg("AEP_MW")
FROM energy_consumption
WHERE "Datetime" BETWEEN '2004-01-01' AND '2005-01-01';

                                QUERY PLAN                                                                          
----------------------------------------------------------------------------
 Aggregate  (cost=10000001302.18..10000001302.19 rows=1 width=8)
   ->  Seq Scan on energy_consumption_2004_2011 energy_consumption  (...)
         Filter: [...]
```

This reduces the number of rows Postgres has to scan by half. By adding more partitions, we can significantly reduce the amount of data Postgres needs to scan to perform a query.

### Partition by hash

Partitioning by hash, unlike by range, can be applied to any data type, including text. A hash function is executed on the partition key to create a reasonably unique number, and that number is then divided by the number of partitions to find the right child table for the row.

To create a table partitioned by hash, the syntax is similar to partition by range. Let's use the USA House Prices dataset we used in [Vectors](../../cloud/vector-database.md) and [Tabular data](README.md), and split that table into two (2) roughly equal parts. Since we already have the `usa_house_prices` table, let's create a new one with the same columns, except this one will be partitioned:

```postgresql
CREATE TABLE usa_house_prices_partitioned (
  "Avg. Area Income" REAL NOT NULL,
  "Avg. Area House Age" REAL NOT NULL,
  "Avg. Area Number of Rooms" REAL NOT NULL,
  "Avg. Area Number of Bedrooms" REAL NOT NULL,
  "Area Population" REAL NOT NULL,
  "Price" REAL NOT NULL,
  "Address" VARCHAR NOT NULL
) PARTITION BY HASH("Address");
```

Let's add two (2) partitions by hash. Hashing uses modulo arithmetic; when creating a child data table with these scheme, you need to specify the denominator and the remainder:

```postgresql
CREATE TABLE usa_house_prices_partitioned_1
PARTITION OF usa_house_prices_partitioned
FOR VALUES WITH (modulus 2, remainder 0);

CREATE TABLE usa_house_prices_partitioned_1
PARTITION OF usa_house_prices_partitioned
FOR VALUES WITH (modulus 2, remainder 1);
```

Importing data into the new table can be done with just one query:

```postgresql
INSERT INTO usa_house_prices_partitioned
SELECT * FROM usa_houses_prices;
```

```
INSERT 0 5000
```

Let's validate that our partitioning scheme worked:

```
postgresml=# SELECT count(*) FROM usa_house_prices_partitioned_1;
 count 
-------
  2528
(1 row)

postgresml=# SELECT count(*) FROM usa_house_prices_partitioned_2;
 count 
-------
  2472
(1 row)
```

Great! As expected, hashing split our dataset into roughly equal parts. To take advantage of this when reading data, you need to specify the partition key "Address" in every query. Postgres will hash the key using the same hashing function and query the child table that can contain the row with the "Address" value:

```
postgresml=# EXPLAIN SELECT
    "Avg. Area House Age",
    "Address"
FROM usa_house_prices_partitioned
WHERE "Address" = '1 Infinite Loop, Cupertino, California';
                                                 QUERY PLAN                                                  
-------------------------------------------------------------------------------------------------------------
 Seq Scan on usa_house_prices_partitioned_1 usa_house_prices_partitioned  (cost=0.00..63.60 rows=1 width=51)
   Filter: (("Address")::text = '1 Infinite Loop, Cupertino, California'::text)
```

### Partitioning vectors

When discussing [Vectors](broken-reference), we mentioned that HNSW indexes slow down table inserts as the table grows over time. Partitioning is a great tool to help us scale vector indexes used for ANN search.

For this example, we'll be using a section of the [Amazon Reviews](https://cseweb.ucsd.edu/\~jmcauley/datasets.html#amazon\_reviews) dataset that we've embedded using the `intloat/e5-large` embeddings model. Our subset of the data contains 250,000 rows and two columns:

| Column                      | Data type      | Example                                      |
| --------------------------- | -------------- | -------------------------------------------- |
| `review_body`               | `VARCHAR`      | `It was great`                               |
| `review_embedding_e5_large` | `VECTOR(1024)` | `[-0.11999297,-1.5099727,-0.102814615, ...]` |

You can [download](https://static.postgresml.org/datasets/amazon\_reviews\_with\_embeddings.csv.gz) this dataset in CSV format from our CDN. To unzip it, install `pigz` and run:

```bash
unpigz amazon_reviews_with_embeddings.csv.gz
```

#### Creating partitions

Let's get started by creating a partitioned table with three (3) child partitions. We'll be using hash partitioning on the `review_body` column which should produce three (3) roughly equally sized tables.

```postgresql
CREATE TABLE amazon_reviews_with_embedding (
    review_body TEXT,
    review_embedding_e5_large VECTOR(1024)
) PARTITION BY HASH(review_body);

CREATE TABLE amazon_reviews_with_embedding_1
PARTITION OF amazon_reviews_with_embedding
FOR VALUES WITH (modulus 3, remainder 0);

CREATE TABLE amazon_reviews_with_embedding_2
PARTITION OF amazon_reviews_with_embedding
FOR VALUES WITH (modulus 3, remainder 1);

CREATE TABLE amazon_reviews_with_embedding_3
PARTITION OF amazon_reviews_with_embedding
FOR VALUES WITH (modulus 3, remainder 2);
```

This creates a total of four (4) tables: one parent table defining the schema and three (3) child tables that will contain the review text and the embeddings vectors. To import data into the tables, you can use `COPY`:

```
postgresml=# \copy 
    amazon_reviews_with_embedding FROM 'amazon_reviews_with_embeddings.csv'
    CSV HEADER;
COPY 250000
```

#### Indexing vectors

Now that we've split our 250,000 vectors into three (3) tables, we can create an HNSW index on each partition independently. This allows us to shard the index into three (3) equally sized parts and because Postgres is a database server, we can do so in parallel.

If you're doing this with `psql`, open up three (3) terminal tabs, connect to your PostgresML database and create an index on each partition separately:

{% tabs %}
{% tab title="Tab 1" %}
```postgresql
SET maintenance_work_mem TO '2GB';

CREATE INDEX ON
    amazon_reviews_with_embedding_1
USING hnsw(review_embedding_e5_large vector_cosine_ops);
```
{% endtab %}

{% tab title="Tab 2" %}
```postgresql
SET maintenance_work_mem TO '2GB';

CREATE INDEX ON
    amazon_reviews_with_embedding_2
USING hnsw(review_embedding_e5_large vector_cosine_ops);
```
{% endtab %}

{% tab title="Tab 3" %}
```postgresql
SET maintenance_work_mem TO '2GB';

CREATE INDEX ON
    amazon_reviews_with_embedding_3
USING hnsw(review_embedding_e5_large vector_cosine_ops);
```
{% endtab %}
{% endtabs %}

This is an example of scaling vector search using partitions. We are increasing our indexing speed 3x because we can create HNSW indexes on separate tables in parallel. Since we have separate indexes for each partition, we are also reducing the size of the HNSW index by 3x, making sure that `INSERT` queries against the data remain sufficiently quick.

#### Partitioned vector search

To perform an ANN search using the indexes we created, we don't have to do anything special. Postgres will automatically scan all three (3) indexes for the closest matches and combine them into one result:

```postgresql
SELECT
    review_body,
    review_embedding_e5_large <=> pgml.embed(
        'Alibaba-NLP/gte-base-en-v1.5',
        'this chair was amazing'
    )::vector(1024) AS cosine_distance
FROM amazon_reviews_with_embedding
ORDER BY cosine_distance
LIMIT 9;
```

```
      review_body       |   cosine_distance   
------------------------+---------------------
 It was great.          |  0.1514577011633712
 It was great.          |  0.1514577011633712
 It was great.          |  0.1514577011633712
 It was great.          |  0.1514577011633712
 It was great.          |  0.1514577011633712
 It was great.          |  0.1514577011633712
 amazing                | 0.17130070002153353
 Amazing                | 0.17130070002153353
 Absolutely phenomenal. |  0.1742546608547857
```

Since scanning HNSW indexes is very quick, we are okay with having to scan all indexes we created for every query. As of this writing, `pgvector` doesn't support partitioning its indexes because this requires splitting the graph in distinct sections. Work on this front will continue and we'll add support for sharding HNSW indexes in the future.

To validate that Postgres is using indexes, prepend `EXPLAIN` to the query. You should see three (3) index scans, one for each partition table.
