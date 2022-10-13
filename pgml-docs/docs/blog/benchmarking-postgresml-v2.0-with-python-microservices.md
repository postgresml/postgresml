
In this blog post, we'll compare the unified PostgresML v2.0 architecture with the decentralized architecture of Python machine learning microservices. Our goal is to demonstrate that doing inference at the data layer with Rust is much faster and simpler than with standard architectures used across the industry today.

## Setting the stage

### Data

We'll be using the [Flight Status Prediction](https://www.kaggle.com/datasets/robikscube/flight-delay-dataset-20182022) dataset from Kaggle which ends up being about 1 GB of floating point features. We won't be using all columns because some of them are redundant, e.g. airport name and airport identifier mean the same thing.

### Algorithm

We'll be training an XGBoost model with default hyperparameters and 25 estimators (also known as boosting rounds).

### Architectures

#### PostgresML

PostgresML architecture will be a single PostgreSQL server with the PostgresML 2.0 extension installed, as documented in our [Installation](/user_guides/setup/v2/installation/) instructions. The client will be [pgbench](https://www.postgresql.org/docs/current/pgbench.html).


#### Python

Python architecture will be composed of:

1. Flask & Gunicorn HTTP/1.1 server speaking JSON
2. CSV file containing the dataset used for training
3. Redis feature store containing the dataset used for inference
4. [ab](https://httpd.apache.org/docs/2.4/programs/ab.html) HTTP client.

## Results

<center>
	<iframe width="600" height="371" seamless frameborder="0" scrolling="no" src="https://docs.google.com/spreadsheets/d/e/2PACX-1vSLNYEaLD92xfrWhx6c2Q248NJGC6Sh9l1wm055HdTPZbakjQg0PVS9KqyuWrNepYvLeOdVNfbmhCwf/pubchart?oid=663872992&amp;format=interactive"></iframe>
</center>

<center>
	<iframe width="600" height="371" seamless frameborder="0" scrolling="no" src="https://docs.google.com/spreadsheets/d/e/2PACX-1vSLNYEaLD92xfrWhx6c2Q248NJGC6Sh9l1wm055HdTPZbakjQg0PVS9KqyuWrNepYvLeOdVNfbmhCwf/pubchart?oid=294879553&amp;format=interactive"></iframe>
</center>
Python training RAM: 11.6GB
PostgresML training RAM: 5.2GB

## Methodology

### PostgresML

```
pgbench -f pgbench.sql -p 28813 -h 127.0.0.1 pgml -t 10000 -c 1 --protocol extended
pgbench (14.5 (Ubuntu 14.5-0ubuntu0.22.04.1), server 13.8)
transaction type: pgbench.sql
scaling factor: 1
query mode: extended
number of clients: 1
number of threads: 1
number of transactions per client: 10000
number of transactions actually processed: 10000/10000
latency average = 0.169 ms
initial connection time = 0.995 ms
tps = 5926.537018 (without initial connection time)
```

### Python

```
ab -n 10000 -c 1 -T application/json -k -p ab.txt http://localhost:8000/
This is ApacheBench, Version 2.3 <$Revision: 1879490 $>
Copyright 1996 Adam Twiss, Zeus Technology Ltd, http://www.zeustech.net/
Licensed to The Apache Software Foundation, http://www.apache.org/

Benchmarking localhost (be patient)
Completed 1000 requests
Completed 2000 requests
Completed 3000 requests
Completed 4000 requests
Completed 5000 requests
Completed 6000 requests
Completed 7000 requests
Completed 8000 requests
Completed 9000 requests
Completed 10000 requests
Finished 10000 requests


Server Software:        gunicorn
Server Hostname:        localhost
Server Port:            8000

Document Path:          /
Document Length:        13 bytes

Concurrency Level:      1
Time taken for tests:   11.640 seconds
Complete requests:      10000
Failed requests:        0
Keep-Alive requests:    0
Total transferred:      1580000 bytes
Total body sent:        1770000
HTML transferred:       130000 bytes
Requests per second:    859.09 [#/sec] (mean)
Time per request:       1.164 [ms] (mean)
Time per request:       1.164 [ms] (mean, across all concurrent requests)
Transfer rate:          132.55 [Kbytes/sec] received
                        148.49 kb/s sent
                        281.05 kb/s total

Connection Times (ms)
              min  mean[+/-sd] median   max
Connect:        0    0   0.0      0       0
Processing:     1    1   0.4      1      19
Waiting:        0    1   0.4      1      19
Total:          1    1   0.4      1      19

Percentage of the requests served within a certain time (ms)
  50%      1
  66%      1
  75%      1
  80%      1
  90%      1
  95%      1
  98%      1
  99%      2
 100%     19 (longest request)
```
