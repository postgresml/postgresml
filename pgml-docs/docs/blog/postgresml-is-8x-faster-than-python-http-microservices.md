---
author: Lev Kokotov
description: PostgresML v2.0 outperforms Python HTTP microservices by a factor of 8 in throughput, latency, and memory utilization.
image: http://localhost:8001/images/logos/logo-small.png
image_alt: We're going really fast now.
---

# PostgresML is 8x faster than Python HTTP microservices

<p class="author">
  <img width="54px" height="54px" src="/images/team/lev.jpg" alt="Author" />
  Lev Kokotov<br/>
  October 17, 2022
</p>


In this post, we'll benchmark the unified PostgresML v2.0 architecture and compare it to decentralized Python machine learning microservices. Our goal is to demonstrate that inference at the data layer with Rust and Postgres is much faster than at the service layer with Python and Redis, the latter being the current industry standard.

## Setting the stage

### Data

We'll be using the [Flight Status Prediction](https://www.kaggle.com/datasets/robikscube/flight-delay-dataset-20182022) dataset from Kaggle which, after some post-processing, ends up being about 2 GB of floating point features. We won't be using all columns because some of them are redundant, e.g. airport name and airport identifier, which refer to the same thing.

### Model

We'll be training an XGBoost model with default hyperparameters and 25 estimators (also known as boosting rounds).

### Candidate architectures

#### PostgresML

PostgresML architecture will be composed of:

1. A PostgreSQL server with PostgresML v2.0
2. [pgbench](https://www.postgresql.org/docs/current/pgbench.html) SQL client


#### Python

Python architecture will be composed of:

1. A Flask/Gunicorn server accepting and returning JSON
2. CSV file with the training data
3. Redis feature store with the inference dataset, serialized with JSON
4. [ab](https://httpd.apache.org/docs/2.4/programs/ab.html) HTTP client

## Results

### Throughput

<center>
	<iframe width="600" height="371" seamless frameborder="0" scrolling="no" src="https://docs.google.com/spreadsheets/d/e/2PACX-1vSLNYEaLD92xfrWhx6c2Q248NJGC6Sh9l1wm055HdTPZbakjQg0PVS9KqyuWrNepYvLeOdVNfbmhCwf/pubchart?oid=188372587&amp;format=interactive"></iframe>
</center>

Throughput is defined as the number of XGBoost predictions we can serve per second. PostgresML outperformed Python and Redis by a **factor of 8**.

In Python, most of the cost came from fetching and deserializing Redis data, passing it through Python into XGBoost, and returning the prediction serialized as JSON in the HTTP response.

PostgresML, on the other hand, collocates data and compute, so fetching data from a Postgres table, which already comes in a standard floating point format, and passing it to XGBoost through a Rust function, is much more efficient and therefore much faster.

An interesting thing happened at 20 clients: PostgresML throughput started to quickly decrease. This may be surprising to some, but to Postgres enthusiasts it's a known issue: Postgres isn't very good at handling many concurrent active connections. To mitigate this, we introduced PgBouncer (a Postgres proxy) in front of our database, and the throughput increased back up and continued to hold as we went to 100 clients.

### Latency

<center>
	<iframe width="600" height="371" seamless frameborder="0" scrolling="no" src="https://docs.google.com/spreadsheets/d/e/2PACX-1vSLNYEaLD92xfrWhx6c2Q248NJGC6Sh9l1wm055HdTPZbakjQg0PVS9KqyuWrNepYvLeOdVNfbmhCwf/pubchart?oid=1092074944&amp;format=interactive"></iframe>
</center>

Latency is defined as the time it takes to return a single XGBoost prediction. Since most systems have limited resources, throughput directly impacts latency (and vice versa). If there are many active requests, clients waiting in the queue will take longer to be serviced, and overall system latency will increase.

In this benchmark, PostgresML outperformed Python by a **factor of 8** as well. You'll note the same issue happened at 20 clients, and the same mitigation using PgBouncer reduced its impact.

Meanwhile, Python latency continued to increase substantially.


### Memory utilization

<center>
	<iframe width="600" height="371" seamless frameborder="0" scrolling="no" src="https://docs.google.com/spreadsheets/d/e/2PACX-1vSLNYEaLD92xfrWhx6c2Q248NJGC6Sh9l1wm055HdTPZbakjQg0PVS9KqyuWrNepYvLeOdVNfbmhCwf/pubchart?oid=1410199200&amp;format=interactive"></iframe>
</center>

Python is known for using more memory than more optimized languages and, in this case, it used **7 times** more than PostgresML.

PostgresML is a Postgres extension and it shares RAM with the database server. Postgres is very efficient at fetching and allocating only the memory it needs. It reuses `shared_buffers` and OS page cache to store rows for inference, and requires very little to no memory allocation to serve queries.

Meanwhile Python must allocate memory for each feature it receives from Redis, at the very least doubling memory requirements. This benchmark did not measure Redis memory utilization, which is an additional and often substantial cost of running traditional machine learning microservices.

## Quick note on training

<center>
	<iframe width="600" height="371" seamless frameborder="0" scrolling="no" src="https://docs.google.com/spreadsheets/d/e/2PACX-1vSLNYEaLD92xfrWhx6c2Q248NJGC6Sh9l1wm055HdTPZbakjQg0PVS9KqyuWrNepYvLeOdVNfbmhCwf/pubchart?oid=294879553&amp;format=interactive"></iframe>
</center>

We spent the majority of our time measuring inference, but it's worth to do a quick aside on training. Since Python uses Pandas to load data, it was notably more memory hungry. Before even passing the data into XGBoost, we were already at 8GB RSS (resident set size); during actual fitting, memory utilization went to almost 12GB.

PostresML meanwhile enjoyed sharing RAM with the Postgres server and only allocated the memory needed by XGBoost. The overhead was still significant, but we managed to train the same model using only 5GB of RAM. PostgresML therefore allows to train models on datasets twice as large as Python, while using identical hardware.

## Methodology

### Hardware

Both the client and the server were located on the same machine. Redis was running locally as well. One can expect additional latency from Redis servers running over the network. The machine used was an 8 core, 16 threads AMD Ryzen 7 5800X with 32GB RAM, 1TB NVMe SSD and Ubuntu 22.04. Only the CPU was used for both training and inference.

### Configuration

Gunicorn was running with 5 workers and 2 threads per worker. Postgres was given up to 100 connections, but no more than 20 were used. XGBoost was set to use 2 threads during inference and all available CPU cores (16 threads) during training. PgBouncer was given a `default_pool_size` of 10, so only 10 Postgres connections were used.

Both `ab` and `pgbench` used all available resources, but are very lightweight; the requests were a single JSON object and a single query respectively. Both of the clients use persistent connections, `ab` by using HTTP Keep-Alives, and `pgbench` by keeping the Postgres connection open for the duration of the benchmark.

### ML

Data used for training and inference is available [here](https://static.postgresml.org/benchmarks/flights.csv). Data stored in the Redis feature store is available [here](https://static.postgresml.org/benchmarks/flights_sub.csv). It's only a subset because it was taking hours to load the entire dataset into Redis with a single Python process (28 million rows). Postgres `COPY` only took about a minute.

PostgresML model was trained with:

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
