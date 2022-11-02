# Scaling PostgresML to 1 Million XGBoost Predictions per Second

<p class="author">
  <img width="54px" height="54px" src="/images/team/lev.jpg" alt="Author" />
  Lev Kokotov<br/>
  November 7, 2022
</p>

The question "Does it Scale?" has become somewhat of a meme in software engineering. There is a good reason for it though, because most businesses plan for success. If your app, online store, or SaaS takes off, you want to be sure that the system powering it can serve all your customers.

At PostgresML, we are very concerned with scale. Our engineering background took us through scaling OLTP and OLAP Postgres to 100 TB+, so we're certain that Postgres scales, but could we scale machine learning alongside with it?

In this post, we'll discuss all the challenges facing scaling machine learning inference with PostgresML, and how we solved them to achieve a glorious **1 million XGBoost predictions per second** on commodity hardware. 

## An Image Worth Four Thousand Words

Our thesis, and the reason why we chose Postgres as our host for machine learning, is that scaling machine learning inference is very similar to scaling read queries in a typical database cluster.

Inference speed varies based on the model complexity (e.g. `n_estimators` for XGBoost) and the size of the dataset (how many features the model uses), which is analogous to query complexity and table size in the database world. Scaling the latter is mostly a solved problem.

### System Architecture

If you're a Postgres enthusiast (or a database engineer), scaling Postgres may not be a secret to you, and you can jump straight to the [results](#results). For everyone else, here is a diagram showing the final state of our system:

<center>
![Scaling PostgresML](/images/illustrations/scaling-postgresml-1.svg) <br />
_System Architecture_
</center>

| Component | Description |
|-----------|-------------|
| Clients | Regular Postgres clients |
| ELB | [Elastic Network Load Balancer](https://aws.amazon.com/elasticloadbalancing/) |
| PgCat | A Postgres [pooler](https://github.com/levkk/pgcat/) with built-in load balancing, failover, and sharding |
| Replica | Regular Postgres [replicas](https://www.postgresql.org/docs/current/high-availability.html) |
| Primary | Regular Postgres primary |


Our architecture has four components that may need to scale up or down based on load:

1. Clients
2. Load balancer
3. [PgCat](https://github.com/levkk/pgcat/) pooler
4. Postgres replicas

We intentionally don't discuss scaling the primary in this post, because sharding, which is the most effective way to do so, is a fascinating subject that deserves its own series of posts. Spoiler alert: we sharded Postgres without any problems.

#### Clients

Clients are regular Postgres connections coming from web apps, job queues, or pretty much anywhere that needs data. They can be long-living or ephemeral and they typically grow in numbers as the application scales.

Most modern deployments use containers which are added as load on the app increases, and removed as the load decreases. This is called dynamic horizontal scaling, and it's an effective way to adapt to changing traffic patterns of a regular application.

This is the most important rule of any system: the number of clients can change at any time, and the system must adapt.

#### Load Balancer

The load balancer is a way to spread traffic across horizontally scalable components, by routing new connections to targets in a round robin (or random) fashion. It's typically a very large box (or a fast router), but even those need to be scaled if traffic suddenly increases. Thankfully, since we're running our system on AWS, this is already taken care of, for a reasonably small fee.

#### PgCat

<center>
	<img src="https://raw.githubusercontent.com/levkk/pgcat/main/pgcat3.png" alt="PgCat" height="300" width="auto" /> <br />
	_Meow. All your Postgres belong to me._
</center>

If you've used Postgres in the past, you know that it can't handle many concurrent connections. For large deployments, it's necessary to run something we call a pooler. A pooler routes thousands of clients to only a few dozen server connections by time-sharing when a client can use a server. Because most queries are very quick, this is a very effective way to run Postgres at scale.

There are many poolers available presently, the most notable being PgBouncer, which has been around for a very long time, and is trusted by many large organizations. Unfortunately, it hasn't evolved much with the growing needs of highly available Postgres deployments, so we wrote [our own](https://github.com/levkk/pgcat/) which added important functionality we needed:

- Load balancing of read queries
- Failover in case a read replica is broken
- Automatic sharding (this feature is still in progress)

In this benchmark, we used its load balancing feature to evenly distribute our XGBoost predictions across our Postgres replicas.


#### Postgres Replicas

Scaling Postgres reads is a solved problem. If more read queries are coming in, add a replica to serve the increased load. If the load is decreasing, remove a replica to save money. The data is replicated from the primary, so all replicas are identical, and all of them can serve any query, or in our case, an XGBoost prediction.


## Results

We ran over a 100 different benchmarks, by changing the number of clients, poolers, replicas, and XGBoost predictions we requested. Our raw data is available [below](#methodology).

### Summary

<center>
	<iframe width="600" height="371" seamless frameborder="0" scrolling="no" src="https://docs.google.com/spreadsheets/d/e/2PACX-1vRm4aEylX8xMNmO-HFFxr67gbZDQ8rh_vss1HvX0tWAUD_zxkwYYNhiBObT1LVe8m6ELZ0seOzmH0ZL/pubchart?oid=2028066210&amp;format=interactive"></iframe>
</center>

<center>
	<iframe width="600" height="371" seamless frameborder="0" scrolling="no" src="https://docs.google.com/spreadsheets/d/e/2PACX-1vRm4aEylX8xMNmO-HFFxr67gbZDQ8rh_vss1HvX0tWAUD_zxkwYYNhiBObT1LVe8m6ELZ0seOzmH0ZL/pubchart?oid=1442564288&amp;format=interactive"></iframe>
</center>

Both latency and throughput, the standard measurements of system performance, scale mostly linearly with the number of replicas. Linear scaling is the north star of all horizontally scalable systems, and most are not able to achieve it because of increasing complexity that comes with synchronization.

Our architecture shares nothing and requires no synchronization. The replicas don't talk to each other and the poolers don't either. Every component has the knowledge it needs (through configuration) to do its job, and they do it well.

### Scaling to Clients

<center>
	<iframe width="600" height="371" seamless frameborder="0" scrolling="no" src="https://docs.google.com/spreadsheets/d/e/2PACX-1vRm4aEylX8xMNmO-HFFxr67gbZDQ8rh_vss1HvX0tWAUD_zxkwYYNhiBObT1LVe8m6ELZ0seOzmH0ZL/pubchart?oid=1396205706&amp;format=interactive"></iframe>
</center>

<center>
	<iframe width="600" height="371" seamless frameborder="0" scrolling="no" src="https://docs.google.com/spreadsheets/d/e/2PACX-1vRm4aEylX8xMNmO-HFFxr67gbZDQ8rh_vss1HvX0tWAUD_zxkwYYNhiBObT1LVe8m6ELZ0seOzmH0ZL/pubchart?oid=1300503904&amp;format=interactive"></iframe>
</center>



### Throughput

Thoughput is measured as the number of sucessful queries per second. In our case, a query is a single XGBoost prediction.



### Latency

Latency is a measurement of delay between sending a query to the database and getting back a response. In our case, that's how long it took to infer a novel datapoint with XGBoost and PostgresML.






### Methodology

This time we used an XGBoost model with 100 trees:

```postgresql
SELECT * FROM pgml.train(
	'flights',
	task => 'regression',
	relation_name => 'flights_mat_3',
	y_column_name => 'depdelayminutes',
	algorithm => 'xgboost',
	hyperparams => '{"n_estimators": 100 }',
	runtime => 'rust'
);
```

and fetched our predictions the usual way:

```postgresql
SELECT pgml.predict(
	'flights',
	ARRAY[
		year,
		quarter,
		month,
		distance,
		dayofweek,
		dayofmonth,
		flight_number_operating_airline,
		originairportid,
		destairportid,
		flight_number_marketing_airline,
		departure
	]
) AS prediction
FROM flights_mat_3 LIMIT 1;
```
