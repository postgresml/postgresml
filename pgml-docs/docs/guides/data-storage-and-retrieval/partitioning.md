# Partitioning

Partitioning is the act of splitting Posgres tables into multiple smaller tables, which allows to query each smaller table independently. This is useful and sometimes necessary when tables get so large that querying a single table becomes too slow. Partitioning requires detailed knowledge of the dataset and uses that knowledge to help Postgres execute faster queries.

### Partitioning schemes

Postgres supports three (3) kinds of partitioning schemes: by range, by list, and by hash. Each scheme is appropriate for different use cases, and choosing the right scheme is important to get the best performance out of your data.

### Partition by range

Partition by range operates on numerical values. Dates, numbers and vectors can be used as range partition keys because their range of values can be split into non-overlapping parts.

For example, if we have a table with a date column (`TIMESTAMPTZ`, a date and time with timezone information), we can create three (3) partitions with the following bounds:

* partition 1 will contain all dates prior to January 1, 2000,
* partition 2 will contain all dates between January 1, 2000 and December 31, 2020,
* partition 3 will contain all dates after January 1, 2021.

While these ranges are not even, we chose them because of some knowledge we have about our dataset. In our hypothetical example, we know that these date ranges will split our dataset into roughly three (3) evenly sized tables.

#### Building partitions

Let's build some real partitions with a dataset from Kaggle: [Hourly Energy Consumption](https://www.kaggle.com/datasets/robikscube/hourly-energy-consumption).

You can create a partition by range in Postgres with just a few queries. Partitioning requires two types of tables: the parent table which defines the partitioning scheme, and the child tables which define the ranges and store the actual data.

Let's start with the parent table:

```sql
CREATE TABLE energy_consumption (
    "Datetime" TIMESTAMPTZ,
    "AEP_MW" REAL
) PARTITION BY RANGE("Datetime");
```

Now, let's add a couple child tables:

```sql
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

Nicely done. The two tables are pretty close to each other which creates a roughly even distribution of data in our partitioning scheme.

Postgres allows to query each partition individually, which is nice if we know what the range specification is. While this works in this example, in a living dataset, we could continue to add partitions to include more values. If we wanted to store dates for the years 2019 through 2023, for example, we would need to make at least one more child table.

To make this user friendly, Postgres allows us to query the parent table instead. As long as we specify the partition key, we are guaranteed to get the most efficient query plan possible:

```sql
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

If we look at the query plan, we'll see that Postgres only queries the first child table we created:

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

which reduces the number of rows it has to scan by half. By adding more smaller partitions, we can significantly reduce the amount of data Postgres needs to scan to execute a query. That being said, scanning multiple tables could be more expensive than scanning just one table, so adding too many partitions quickly reduces its benefits if the queries need to scan more than a few child tables.

### Partition by hash

Partitioning by hash, unlike by range, can be applied to any data type, including text. A hash function is applied to the partition key to create a reasonably unique number, and that number is then divided by the number of partitions to find the right child table for the row.
