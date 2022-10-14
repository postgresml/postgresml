
In this blog post, we'll compare the unified PostgresML v2.0 architecture with the decentralized architecture of Python machine learning microservices. Our goal is to demonstrate that doing inference at the data layer with Rust and Postgres is much faster and simpler than with Python and Redis, as commonly done across the industry today.

## Setting the stage

### Data

We'll be using the [Flight Status Prediction](https://www.kaggle.com/datasets/robikscube/flight-delay-dataset-20182022) dataset from Kaggle which, after some post-processing, ends up being about 1 GB of floating point features. We won't be using all columns because some of them are redundant, e.g. airport name and airport identifier which mean the same thing.

### Algorithm

We'll be training an XGBoost model with default hyperparameters and 25 estimators (also known as boosting rounds).

### Candidate architectures

#### PostgresML

PostgresML architecture will be composed of:

1. PostgreSQL server with PostgresML v2.0
2. [pgbench](https://www.postgresql.org/docs/current/pgbench.html) client


#### Python

Python architecture will be composed of:

1. Flask & Gunicorn server speaking JSON
2. CSV file containing the dataset used for training
3. Redis feature store with the inference dataset, serialized with JSON
4. [ab](https://httpd.apache.org/docs/2.4/programs/ab.html) HTTP client

## Results

### Latency

<center>
	<iframe width="600" height="371" seamless frameborder="0" scrolling="no" src="https://docs.google.com/spreadsheets/d/e/2PACX-1vSLNYEaLD92xfrWhx6c2Q248NJGC6Sh9l1wm055HdTPZbakjQg0PVS9KqyuWrNepYvLeOdVNfbmhCwf/pubchart?oid=1092074944&amp;format=interactive"></iframe>
</center>

PostgresML outperformed Gunicorn and Redis by a **factor of 8** on average.

Most of the inference cost for Python came from fetching data from Redis, deserializing it, and inputting it into XGBoost to get a prediction. PostgresML collocates data and compute, so fetching data from a Postgres table, which already comes in standard floating point format, and passing it to XGBoost through a Rust function, is more efficient and therefore much faster.

An interesting thing happened at 20 clients: PostgresML latency started to quickly increase. This may be surprising to some, but to Postgres enthusiasts it's well known issue: Postgres isn't very good at handling many connections at the same time. To mitigate this, we introduced PgBouncer, a well known Postgres proxy, in front of our database, and the latency decreased and continued to hold as we went to 100 clients.

Meanwhile Python latency continued to increase substantially.

### Throughput

<center>
	<iframe width="600" height="371" seamless frameborder="0" scrolling="no" src="https://docs.google.com/spreadsheets/d/e/2PACX-1vSLNYEaLD92xfrWhx6c2Q248NJGC6Sh9l1wm055HdTPZbakjQg0PVS9KqyuWrNepYvLeOdVNfbmhCwf/pubchart?oid=188372587&amp;format=interactive"></iframe>
</center>

Since most system resources are limited, as is the case with the machine we used to perform this benchmark, latency directly impacts throughput. If active requests take a long time, requests waiting in the queue will take longer to be serviced.

In this benchmark, PostgresML outperformed Python by a **factor of 8**. You'll note the same issue happening at 20 clients and the identical mitigation using PgBouncer to multiplex our requests.

### Memory utilization

<center>
	<iframe width="600" height="371" seamless frameborder="0" scrolling="no" src="https://docs.google.com/spreadsheets/d/e/2PACX-1vSLNYEaLD92xfrWhx6c2Q248NJGC6Sh9l1wm055HdTPZbakjQg0PVS9KqyuWrNepYvLeOdVNfbmhCwf/pubchart?oid=1410199200&amp;format=interactive"></iframe>
</center>

Python is known for using more memory than more optimized languages, and in this case it did use **7 times** more than PostgresML.

PostgresML is a Postgres extension and it shares RAM with the database server. Postgres is very efficient at fetching and allocating only the memory it needs; in this case, Postgres reuses `shared_buffers` and OS page cache to store rows used for inference, and requires very little to no memory allocation to serve queries.

Meanwhile Python must allocate memory for each feature it receives from Redis, at the very least doubling memory requirements during inference. This benchmark did not measure Redis memory usage, which is an additional cost for running traditional machine learning microservices.

## Quick note on training

<center>
	<iframe width="600" height="371" seamless frameborder="0" scrolling="no" src="https://docs.google.com/spreadsheets/d/e/2PACX-1vSLNYEaLD92xfrWhx6c2Q248NJGC6Sh9l1wm055HdTPZbakjQg0PVS9KqyuWrNepYvLeOdVNfbmhCwf/pubchart?oid=294879553&amp;format=interactive"></iframe>
</center>

We spent the majority of our time measuring inference, but it's worth to do a quick aside on training as well. Since Python uses Pandas to load the training dataset, it was notably more memory hungry. Before even loading data into XGBoost, it was already at 8GB RSS (resident set size), and during actual fitting, memory utilization went to almost 12GB.

PostresML meanwhile enjoyed sharing RAM with the Postgres server and only allocated the memory needed by XGBoost to train the model. The overhead was still significant, but we managed to train the same model using only 5GB of RAM.

## Methodology

### Hardware

Both the client and the server were located on the same machine. Redis was running locally as well. The machine is an 8 core, 16 threads AMD Ryzen 7 5800X with 32GB RAM and 1TB NVMe SSD. Both training and inference only used the CPU.

### Configuration

Gunicorn was running with 5 workers and 2 threads per worker. Postgres was given up to 100 connections, as is standard, but no more than 20 were used. XGBoost was set to use 2 threads during inference and all CPU cores (16 threads) during training. PgBouncer was given a `default_pool_size` of 10, so only 10 Postgres connections were used.

Both `ab` and `pgbench` used all available resources, but are both very lightweight and the requests were a single JSON object and a single query respectively. Both of the clients use persistent connections, `ab` by using HTTP Keep-Alives and `pgbench` by keeping the Postgres connection open for the duration of the benchmark.

### ML

Data used for training and inference with PostgresML is available [here](https://static.postgresml.org/benchmarks/flights.csv). Data stored in the Redis feature store is available [here](https://static.postgresml.org/benchmarks/flights_sub.csv). It's only a subset because it was taking hours to load the entire dataset into Redis using a single Python process (28 million rows). Postgres `COPY` only took about a minute.

PostgresML model was trained with:

```postgresql
SELECT * FROM pgml.train(
	project_name => 'benchmark',
	algorithm => 'xgboost',
	hyperparams => '{ "n_estimators": 25 }'
);
```

It had terrible accuracy (in Python as well), most likely because we were missing any kind of weather information, which is likely what causes delays at airports.

### Source code

Benchmark source code can be found on [Github](https://github.com/postgresml/postgresml) in `pgml-docs/docs/blog/benchmarks/python_microservices_vs_postgresml/`

## Feedback

Many thanks and ❤️ to all those who are supporting this endeavor. We’d love to hear feedback from the broader ML and Engineering community about applications and other real world scenarios to help prioritize our work. You can show your support by starring us on our [Github](https://github.com/postgresml/postgresml).
