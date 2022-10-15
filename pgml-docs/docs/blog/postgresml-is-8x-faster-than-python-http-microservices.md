---
author: Lev Kokotov
description: PostgresML v2.0 outperforms Python HTTP microservices by a factor of 8 in throughput, latency, and memory utilization.
image: http://localhost:8001/images/logos/logo-small.png
image_alt: We're going really fast now.
---

# PostgresML is 8-40x faster than Python HTTP microservices

<p class="author">
  <img width="54px" height="54px" src="/images/team/lev.jpg" alt="Author" />
  Lev Kokotov<br/>
  October 17, 2022
</p>

Machine learning architectures can be some of the most complex, expensive and _difficult_ arenas in modern systems. The number of technologies and the amount of hardware required compete for tightening headcount, hosting and latency budgets. Unfortunately, the trend in industry is only getting worse along these lines using state-of-the-art architectures that center around data warehouses, microservices and NoSQL databases.

PostgresML is a simpler alternative to that ever-growing complexity. In this post we explore some additional performance benefits a more elegant architecture offers.

## Candidate architectures

To consider Python microservices with every possible advantage, we first benchmark with Python and Redis running inside the same container. Our goal being to avoid any additional network latency, and put it on a more even footing with PostgresML. Later, testing a networked microservice is relatively devastating. The full source code for both benchmarking setups is available [online](https://github.com/postgresml/postgresml/tree/master/pgml-docs/docs/blog/benchmarks/python_microservices_vs_postgresml).

### PostgresML

PostgresML architecture is composed of:

1. A PostgreSQL server with PostgresML v2.0
2. [pgbench](https://www.postgresql.org/docs/current/pgbench.html) SQL client


### Python

Python architecture is composed of:

1. A Flask/Gunicorn server accepting and returning JSON
2. CSV file with the training data
3. Redis feature store with the inference dataset, serialized with JSON
4. [ab](https://httpd.apache.org/docs/2.4/programs/ab.html) HTTP client

### ML

Both architectures host the same XGBoost model, running predictions against the same dataset.

## Results

### Throughput

<center>
	<iframe width="600" height="371" seamless frameborder="0" scrolling="no" src="https://docs.google.com/spreadsheets/d/e/2PACX-1vSLNYEaLD92xfrWhx6c2Q248NJGC6Sh9l1wm055HdTPZbakjQg0PVS9KqyuWrNepYvLeOdVNfbmhCwf/pubchart?oid=188372587&amp;format=interactive"></iframe>
</center>

Throughput is defined as the number of XGBoost predictions we can serve per second. PostgresML outperforms Python and Redis collocated in the same container by a **factor of 8**.

In Python, most of the cost comes from fetching and deserializing Redis data, passing it through Python into XGBoost, and returning the prediction serialized as JSON in the HTTP response. This is pretty much the bare minimum amount of work you can do for an inference microservice.

PostgresML, on the other hand, collocates data and compute, so fetching data from a Postgres table, which already comes in a standard floating point format, and passing it to XGBoost as a pointer through Rust, is much more efficient and therefore much faster.

An interesting thing happens at 20 clients vs 16 hardware threads available: PostgresML throughput starts to quickly decrease. This may be surprising to some, but to Postgres enthusiasts it's a known issue: Postgres isn't very good at handling many concurrent active connections. To mitigate this, we introduce PgBouncer (a Postgres proxy) in front of the database, and the throughput increases back up and continues to hold as we go to 100 clients.

### Latency

<center>
	<iframe width="600" height="371" seamless frameborder="0" scrolling="no" src="https://docs.google.com/spreadsheets/d/e/2PACX-1vSLNYEaLD92xfrWhx6c2Q248NJGC6Sh9l1wm055HdTPZbakjQg0PVS9KqyuWrNepYvLeOdVNfbmhCwf/pubchart?oid=1092074944&amp;format=interactive"></iframe>
</center>

Latency is defined as the time it takes to return a single XGBoost prediction. Since most systems have limited resources, throughput directly impacts latency (and vice versa). If there are many active requests, clients waiting in the queue take longer to be serviced, and overall system latency increases.

In this benchmark, PostgresML outperforms Python by a **factor of 8** as well. You'll note the same issue happens at 20 clients, and the same mitigation using PgBouncer reduces its impact.

Meanwhile, Python latency continues to increase substantially.


### Memory utilization

<center>
	<iframe width="600" height="371" seamless frameborder="0" scrolling="no" src="https://docs.google.com/spreadsheets/d/e/2PACX-1vSLNYEaLD92xfrWhx6c2Q248NJGC6Sh9l1wm055HdTPZbakjQg0PVS9KqyuWrNepYvLeOdVNfbmhCwf/pubchart?oid=1410199200&amp;format=interactive"></iframe>
</center>

Python is known for using more memory than more optimized languages and, in this case, it uses **7 times** more than PostgresML.

PostgresML is a Postgres extension, and it shares RAM with the database server. Postgres is very efficient at fetching and allocating only the memory it needs. It reuses `shared_buffers` and OS page cache to store rows for inference, and requires very little to no memory allocation to serve queries.

Meanwhile, Python must allocate memory for each feature it receives from Redis, at the very least doubling memory requirements. This benchmark did not measure Redis memory utilization, which is an additional and often substantial cost of running traditional machine learning microservices.

## Quick note on training

<center>
	<iframe width="600" height="371" seamless frameborder="0" scrolling="no" src="https://docs.google.com/spreadsheets/d/e/2PACX-1vSLNYEaLD92xfrWhx6c2Q248NJGC6Sh9l1wm055HdTPZbakjQg0PVS9KqyuWrNepYvLeOdVNfbmhCwf/pubchart?oid=294879553&amp;format=interactive"></iframe>
</center>

We spent the majority of our time measuring inference, but it's worth to do a quick aside on training. Since Python often uses Pandas to load and preprocess data, it is notably more memory hungry. Before even passing the data into XGBoost, we were already at 8GB RSS (resident set size); during actual fitting, memory utilization went to almost 12GB.

Meanwhile, PostresML enjoys sharing RAM with the Postgres server and only allocates the memory needed by XGBoost. The overhead was still significant, but we managed to train the same model using only 5GB of RAM. PostgresML therefore allows training models on datasets twice as large as Python, while using identical hardware.

## What about UltraJSON/MessagePack/Serializer X?

JSON is the most user-friendly format, but it's certainly not the fastest. MessagePack and Ultra JSON, for example, are faster and more efficient at reading and storing binary information. So would using them instead of Python's built-in `json` module in this benchmark be better? The answer is not really.

<center>
	<iframe width="600" height="371" seamless frameborder="0" scrolling="no" src="https://docs.google.com/spreadsheets/d/e/2PACX-1vSLNYEaLD92xfrWhx6c2Q248NJGC6Sh9l1wm055HdTPZbakjQg0PVS9KqyuWrNepYvLeOdVNfbmhCwf/pubchart?oid=1855533349&amp;format=interactive"></iframe>
</center>

<center>
	<iframe width="600" height="371" seamless frameborder="0" scrolling="no" src="https://docs.google.com/spreadsheets/d/e/2PACX-1vSLNYEaLD92xfrWhx6c2Q248NJGC6Sh9l1wm055HdTPZbakjQg0PVS9KqyuWrNepYvLeOdVNfbmhCwf/pubchart?oid=309145169&amp;format=interactive"></iframe>
</center>

Time to (de)serialize is important, but needing (de)serialization in the first place is the bottleneck. Taking data out of a remote system (e.g. a feature store like Redis), sending it over a network socket, parsing it into a Python object (which requires memory allocation), only to convert it again to a binary type for XGBoost, is causing unnecessary delays in the system.

PostgresML does **one in-memory copy** of features from Postgres. No network, no (de)serialization, no unnecessary latency.


## What about the real world?

Testing over localhost is convenient, but not that realistic. In production deployments, the client and the server are on different machines, and in the case of the Python + Redis architecture, the feature store is yet another network hop away. To demonstrate this, we spun up 3 EC2 instances and ran the benchmark again.

### Throughput

<center>
	<iframe width="600" height="371" seamless frameborder="0" scrolling="no" src="https://docs.google.com/spreadsheets/d/e/2PACX-1vSLNYEaLD92xfrWhx6c2Q248NJGC6Sh9l1wm055HdTPZbakjQg0PVS9KqyuWrNepYvLeOdVNfbmhCwf/pubchart?oid=179138052&amp;format=interactive"></iframe>
</center>

Network gap between Redis and Gunicorn made things worse...a lot worse. The same concurrency issue hit Postgres at 20 connections, and we used PgBouncer again to save the day.

### Latency

<center>
	<iframe width="600" height="371" seamless frameborder="0" scrolling="no" src="https://docs.google.com/spreadsheets/d/e/2PACX-1vSLNYEaLD92xfrWhx6c2Q248NJGC6Sh9l1wm055HdTPZbakjQg0PVS9KqyuWrNepYvLeOdVNfbmhCwf/pubchart?oid=10610340&amp;format=interactive"></iframe>
</center>

Just like with our local benchmark, Python starts to deteriorate at 20 clients quite significantly and explodes at 100 clients. Latency has a compound effect: the more slow clients are querying the system, the slower it gets for everyone waiting in line to get a prediction.

PostgresML performs well in both cases. Data collocation removes any network latency to the feature store, and Rust, together with single-copy semantics of the inference layer, did the rest.

## Methodology

### Hardware

Both the client and the server are located on the same machine. Redis is running locally as well. One can expect additional latency from Redis servers running over the network. The machine is an 8 core, 16 threads AMD Ryzen 7 5800X with 32GB RAM, 1TB NVMe SSD and Ubuntu 22.04.

AWS EC2 benchmarks are done with one `c5.4xlarge` instance hosting Gunicorn and PostgresML, and two `c5.large` instances hosting the client and Redis, respectively.

### Configuration

Gunicorn runs with 5 workers and 2 threads per worker. Postgres was given up to 100 connections, but no more than 20 were used. XGBoost is set to use 2 threads during inference and all available CPU cores (16 threads) during training. PgBouncer is given a `default_pool_size` of 10, so only 10 Postgres connections are used.

Both `ab` and `pgbench` use all available resources, but are very lightweight; the requests were a single JSON object and a single query respectively. Both of the clients use persistent connections, `ab` by using HTTP Keep-Alives, and `pgbench` by keeping the Postgres connection open for the duration of the benchmark.

## ML


### Data

We use the [Flight Status Prediction](https://www.kaggle.com/datasets/robikscube/flight-delay-dataset-20182022) dataset from Kaggle. After some post-processing, it ends up being about 2 GB of floating point features. We don't use all columns because some of them are redundant, e.g. airport name and airport identifier, which refer to the same thing.

### Model

We train an XGBoost model with default hyperparameters and 25 estimators (also known as boosting rounds).

Data used for training and inference is available [here](https://static.postgresml.org/benchmarks/flights.csv). Data stored in the Redis feature store is available [here](https://static.postgresml.org/benchmarks/flights_sub.csv). It's only a subset because it was taking hours to load the entire dataset into Redis with a single Python process (28 million rows). Postgres `COPY` only took about a minute.

PostgresML model is trained with:

```postgresql
SELECT * FROM pgml.train(
	project_name => 'r2',
	algorithm => 'xgboost',
	hyperparams => '{ "n_estimators": 25 }'
);
```

It had terrible accuracy (as did the Python version), probably because we were missing any kind of weather information, the latter most likely causing delays at airports.

### Source code

Benchmark source code can be found on [Github](https://github.com/postgresml/postgresml/tree/master/pgml-docs/docs/blog/benchmarks/python_microservices_vs_postgresml/).

## Feedback

Many thanks and ❤️ to all those who are supporting this endeavor. We’d love to hear feedback from the broader ML and Engineering community about applications and other real world scenarios to help prioritize our work. You can show your support by starring us on our [Github](https://github.com/postgresml/postgresml).
