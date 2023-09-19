# Overview

PostgresML supercharges your Postgres database into an end-to-end MLOps platform, seamlessly integrating the key components of the machine learning workflow. Without moving data outside your database, PostgresML allows Postgres to function as a feature store, model store, training engine, and inference service all in one place. This consolidation streamlines building and deploying performant, real-time AI applications for developers.

\
With PostgresML, your database becomes a full-fledged ML workbench. It supports supervised and unsupervised algorithms like regression, clustering, deep neural networks, and more. You can build models using SQL on data inside Postgres. Models are stored back into Postgres for low-latency inferences later.

\
PostgresML also unlocked the power of large language models like GPT-3 for your database. With just a few lines of SQL, you can leverage state-of-the-art NLP to build semantic search, analyze text, extract insights, summarize documents, translate text, and more. The possibilities are endless.

\
PostgresML is open source but also offered as a fully-managed cloud service. In addition to the SQL API, it provides Javascript, Python, and Rust SDKs to quickly build vector search, chatbots, and other ML apps in just a few lines of code.

\
To scale horizontally, PostgresML utilizes PgCat, an advanced PostgreSQL proxy and load balancer. PgCat enables sharding, load balancing, failover, and mirroring to achieve extremely high throughput and low latency. By keeping the entire machine learning workflow within Postgres, PostgresML avoids expensive network calls between disparate systems. This allows PostgresML to handle millions of requests per second at up to 40x the speed of other platforms. PgCat and Postgres replication deliver seamless scaling while retaining transactional integrity.

\
