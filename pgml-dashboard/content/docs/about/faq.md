# FAQ

*How far can this scale?*

Petabyte-sized Postgres deployments are [documented](https://www.computerworld.com/article/2535825/size-matters--yahoo-claims-2-petabyte-database-is-world-s-biggest--busiest.html) in production since at least 2008, and [recent patches](https://www.2ndquadrant.com/en/blog/postgresql-maximum-table-size/) have enabled working beyond exabyte and up to the yotabyte scale. Machine learning models can be horizontally scaled using standard Postgres replicas.

*How reliable can this be?*

Postgres is widely considered mission critical, and some of the most [reliable](https://www.postgresql.org/docs/current/wal-reliability.html) technology in any modern stack. PostgresML allows an infrastructure organization to leverage pre-existing best practices to deploy machine learning into production with less risk and effort than other systems. For example, model backup and recovery happens automatically alongside normal Postgres data backup.

*How good are the models?*

Model quality is often a trade-off between compute resources and incremental quality improvements. Sometimes a few thousands training examples and an off the shelf algorithm can deliver significant business value after a few seconds of training. PostgresML allows stakeholders to choose several [different algorithms](/docs/guides/training/algorithm_selection/) to get the most bang for the buck, or invest in more computationally intensive techniques as necessary. In addition, PostgresML can automatically apply best practices for [data cleaning](/docs/guides/training/preprocessing/)) like imputing missing values by default and normalizing features to prevent common problems in production. 

PostgresML doesn't help with reformulating a business problem into a machine learning problem. Like most things in life, the ultimate in quality will be a concerted effort of experts working over time. PostgresML is intended to establish successful patterns for those experts to collaborate around while leveraging the expertise of open source and research communities.

*Is PostgresML fast?*

Collocating the compute with the data inside the database removes one of the most common latency bottlenecks in the ML stack, which is the (de)serialization of data between stores and services across the wire. Modern versions of Postgres also support automatic query parrellization across multiple workers to further minimize latency in large batch workloads. Finally, PostgresML will utilize GPU compute if both the algorithm and hardware support it, although it is currently rare in practice for production databases to have GPUs. We're working on [benchmarks](https://github.com/postgresml/postgresml/blob/master/pgml-extension/sql/benchmarks.sql).
