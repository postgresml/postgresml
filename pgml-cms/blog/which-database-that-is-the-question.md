---
description: >-
  Choosing a database for your product sounds like a hard problem. These days,
  we engineers have an abundance of choice, which makes this decision harder
  than it should be. Let's look at a few options.
---

# Which Database, That is the Question

<div align="left">

<figure><img src=".gitbook/assets/lev.jpg" alt="Author" width="100"><figcaption></figcaption></figure>

</div>

Lev Kokotov

September 1, 2022

Choosing a database for your product sounds like a hard problem. These days, we engineers have an abundance of choice, which makes this decision harder than it should be. Let's look at a few options.

## Redis

Redis is not really a database. It's a key-value store that keeps your data in memory. If Redis accidentally restarts, due to power failure for example, you'll lose some or all of your keys, depending on configuration. Don't get me wrong, I love Redis; it's fast, it has cool data structures like sets and HyperLogLog, and it can even horizontally scale most of its features in cluster mode.

For this and many of its other properties, it is the key-value store of choice for high throughput systems like ML feature stores, job queues, Twitter and Twitch[^1]. None of those systems however expect your data to be safe. In fact, if it's gone, your product should be able to go on like nothing really happened. For those deployments, machine learning and other features it powers, are treated as just a nice to have.

## ScyllaDB (and friends)

Scylla is the new kid on the block, at least as far as databases go. It's been around for 6 years, but it's making headlines with large deployments like Discord[^2] and Expedia[^3]. It takes the idea that key-value stores can be fast, and if you have a power outage, your data remains safe and replicated across availability zones of your favorite cloud. To top it all off, it uses Cassandra's SQL syntax and client/server protocol, so you might think that it can actually power your business-critical systems.

At its heart though Scylla is still a key-value store. We can put things in, but getting them back out in a way that makes sense will still prove to be a challenge. It does have secondary indexes, so if you want to find your users by email instead of by primary key one day, you still might be able to, it'll just be slower.

Ultimately though, with no join support or foreign keys, Scylla tables, much like Redis keys, are isolated from each other. So finding out how many of your customers in San Francisco have ordered your best selling shoes will require an expensive data warehouse instead of a `GROUP BY city ORDER BY COUNT(*)`.

You might think DynamoDB, MongoDB, and all other SQL look-alikes[^4] are better, but they are all forgetting one important fact.

## Denormalized Data is DOA

Relationships are the foundation of everything, ranging from personal well-being to having a successful business. Most problems we'll run into involve understanding how entities work together. Which users logged in today? That's a relationship between users, logins and time. How many users bought our top selling product? How much did that product cost to deliver? Those are relationships between prices, products, date ranges, users, and orders.

If we denormalize this data, by either flattening it into a key-value store or just storing it in independent tables in different databases, we lose the ability to query it in interesting ways, and if we lose that, we stop understanding our business.

## PostgreSQL

<figure><img src=".gitbook/assets/image (52).png" alt=""><figcaption></figcaption></figure>

Okay, that was a bit of a spoiler.

When looking at our options, one has to wonder, why can't we have our cake and eat it too? That's a bad analogy though, because we're not asking for that much and we certainly can have it.

When it comes to reliability, there is no better option. PostgreSQL does not lose data. In fact, it has several layers of failure checks[^5] to ensure that bytes in equals bytes out. When installed on modern SSDs, PostgreSQL can serve 100k+ write transactions per second without breaking a sweat, and push 1GB/second write throughput. When it comes to reads, it can serve datasets going into petabytes and is horizontally scalable into millions of reads per second. That's better than web scale[^6].

Most importantly though, Postgres allows you to understand your data and your business. With just a few joins, you can connect users to orders to chargebacks and to your website visits. You don't need a data warehouse, Spark, Cassandra, large pipelines to make them all work together or data validation scripts. You can read, write and understand straight from the source.

## In Comes Machine Learning

Understanding your business is good, but what if you could improve it too? Most are tempted to throw spaghetti against the wall (and that's okay), but machine learning allows for a more scientific approach. Traditionally, ML has been tough to use with modern data architectures: using key-value databases makes data virtually inaccessible in bulk. With PostgresML though, you can train an XGBoost model directly on your orders table with a single SQL query:

```postgresql
SELECT pgml.train(
	'Orders Likely To Be Returned', -- name of your model
	'regression', -- objective (regression or classification)
	'public.orders', -- table
	'refunded', -- label (what are we predicting)
	'xgboost' -- algorithm
);

SELECT
	pgml.predict(
		'Orders Likely To Be Returned',
		ARRAY[orders.*]) AS refund_likelihood,
		orders.*
FROM orders
ORDER BY refund_likelyhood DESC
LIMIT 100;
```

Checkmate.

Check out our [free PostgresML tutorials](https://cloud.postgresml.org) if you haven't already, and become a machine learning engineer with just a few lines of SQL.

[^1]: [Enterprise Redis Twitch Case Study](https://twitter.com/Redisinc/status/962856298088992768)

[^2]: [Discord Chooses ScyllaDB as Its Core Storage Layer](https://www.scylladb.com/press-release/discord-chooses-scylla-core-storage-layer/)

[^3]: [Expedia Group: Our Migration Journey to ScyllaDB](https://www.scylladb.com/2021/02/18/expedia-group-our-migration-journey-to-scylla/)

[^4]: [SQL to MongoDB Mapping Chart](https://www.mongodb.com/docs/manual/reference/sql-comparison/)

[^5]: [PostgreSQL WAL](https://www.postgresql.org/docs/14/wal.html)

[^6]: [Web scale](https://www.youtube.com/watch?v=b2F-DItXtZs)
