---
description: >-
  With PostgresML, developers can prototype and deploy AI applications quickly
  and at scale in a matter of minutes — a task that would otherwise have taken
  weeks.
---

# PostgresML raises $4.7M to launch serverless AI application databases based on Postgres

<div align="left">

<figure><img src=".gitbook/assets/montana.jpg" alt="Author" width="125"><figcaption></figcaption></figure>

</div>

Montana Low

May 10, 2023

Developing AI-powered applications requires a range of APIs for carrying out tasks such as text generation, sentence embeddings, classification, regression, ranking, as well as a stateful database to store the features. The recent explosion in AI power has only driven the costs and complexity for application developers higher. PostgresML’s extension for Postgres brings AI tasks to the database, reducing complexity for app developers, and yielding a host of additional performance, cost and quality advantages.

With PostgresML, developers can prototype and deploy AI applications quickly and at scale in a matter of minutes — a task that would otherwise have taken weeks. By streamlining the infrastructure requirements, PostgresML allows developers to concentrate on creating intelligent and engaging applications.

<figure><img src=".gitbook/assets/image (20).png" alt=""><figcaption></figcaption></figure>

## Our Serverless AI Cloud

Building on the success of our open source database extension to Postgres, we’ve created a cloud with our own custom Postgres load balancer. PgCat is tailored for our machine learning workflows at scale and enables us to pool multiple machines and connections, creating a mesh of Postgres clusters that appear as independent Postgres databases. We can scale single tenant workloads across a large fleet of physical machines, beyond traditional replication, enabling efficient multi GPU inference workloads.

Creating a new database in this cluster takes a few milliseconds. That database will have massive burst capacity, up to a full sized shard with 128 concurrent workers. Our scaling is so fast and efficient we are offering free databases with up to 5GB of data, and only charge if you’d like us to cache your custom models, data, and indexes, for maximum performance.

Even though PgCat is barely a year old, there are already production workloads handling hundreds of thousands of queries per second at companies like Instacart and OneSignal. Our own deployment is already managing hundreds of independent databases, and launching many new ones every day.

<figure><img src=".gitbook/assets/image (21).png" alt=""><figcaption></figcaption></figure>

## Open Source is the Way Forward

Our team moves quickly by working collaboratively within the larger open source community. Our technologies, both [PostgresML](https://github.com/postgresml/postgresml) and [PgCat](https://github.com/postgresml/pgcat), are MIT-licensed because we believe the opportunity size and efforts required to succeed safely are long term and global in scale.

PostgresML is an extension for Postgres that brings models and algorithms into the database engine. You can load pretrained state-of-the-art LLMs and datasets directly from HuggingFace. Additionally, the Postgres community has created a treasure trove of extensions like pgvector. For example, combining the vector database, open source models, and input text in a single process is up to 40 times faster than alternative architectures for semantic search. The quality of those open source embeddings are also at the top of the leaderboards, which include proprietary models.

By integrating all the leading machine learning libraries like Torch, Tensorflow, XGBoost, LightGBM, and Scikit Learn, you can go beyond a simple vector database, to training your own models for better ranking and recall using your application data and real user interactions, e.g personalizing vector search results by taking into account user behavior or fine-tuning open source LLMs using AB test results.

Many amazing open and collaborative communities are shaping the future of our industry, and we will continue to innovate and contribute alongside them. If you’d like to see more of the things you can do with an AI application database, check out the latest series of articles.

<figure><img src=".gitbook/assets/image (22).png" alt=""><figcaption></figcaption></figure>

## Thanks to Our Community

We see a long term benefit to our community by building a company on top of our software that will push the boundaries of scale and edges of practicality that smaller independent teams running their own Postgres databases and AI workloads may not approach.

Toward that end, we’ve raised $4.7M in seed funding led by Amplify Partners. Angels participating in the round include Max Mullen and Brandon Leonardo (Co-founders of Instacart), Jack Altman (Co-founder of Lattice), Rafael Corrales (Founding Investor at Vercel), Greg Rosen (Box Group), Jeremy Stanley (Co-founder of Anomalo) and James Yu (Co-founder of Parse).

Our sincere thanks also goes out to all of the friends, family, colleagues and open source contributors who continue to support us on this journey. We’d love to have you join us as well, because the next decade in this sector is going to be a wild ride.

## We’re Hiring

If this sounds as interesting to you as it does to us, join us! We’re hiring experienced engineers familiar with Rust, Machine Learning, Databases and managing Infrastructure as a Service. The best way to introduce yourself is by submitting a pull request or reporting an issue on our open source projects [PostgresML](https://github.com/postgresml/postgresml), [PgCat](https://github.com/postgresml/pgcat) & [pg\_stat\_sysinfo](https://github.com/postgresml/pg\_stat\_sysinfo), or emailing us at team@postgresml.org.
