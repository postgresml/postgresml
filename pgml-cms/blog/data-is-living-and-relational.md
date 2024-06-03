---
description: >-
  A common problem with data science and machine learning tutorials is the
  published and studied datasets are often nothing like what you’ll find in
  industry.
featured: false
tags: [engineering]
---

# Data is Living and Relational

<div align="left">

<figure><img src=".gitbook/assets/montana.jpg" alt="Author" width="125"><figcaption></figcaption></figure>

</div>

Montana Low

August 25, 2022

A common problem with data science and machine learning tutorials is the published and studied datasets are often nothing like what you’ll find in industry.

| width | height | area |
| ----- | ------ | ---- |
| 1     | 1      | 1    |
| 2     | 1      | 2    |
| 2     | 2      | 4    |

They are:

* usually denormalized into a single tabular form, e.g. a CSV file
* often relatively tiny to medium amounts of data, not big data
* always static, with new rows never added
* sometimes pretreated to clean or simplify the data

As Data Science transitions from academia into industry, these norms influence organizations and applications. Professional Data Scientists need teams of Data Engineers to move data from production databases into data warehouses and denormalized schemas, which are more familiar and ideally easier to work with. Large offline batch jobs are a typical integration point between Data Scientists and their Engineering counterparts, who primarily deal with online systems. As the systems grow more complex, additional specialized Machine Learning Engineers are required to optimize performance and scalability bottlenecks between databases, warehouses, models and applications.

This eventually leads to expensive maintenance and terminal complexity: new improvements to the system become exponentially more difficult. Ultimately, previously working models start getting replaced by simpler solutions, so the business can continue to iterate. This is not a new phenomenon, see the fate of the Netflix Prize.

## Announcing the PostgresML Gym 🎉

Instead of starting from the academic perspective that data is dead, PostgresML embraces the living and dynamic nature of data produced by modern organizations. It's relational and growing in multiple dimensions.

<figure><img src=".gitbook/assets/image (54).png" alt=""><figcaption></figcaption></figure>

Relational data:

* is normalized for real time performance and correctness considerations
* has new rows added and updated constantly, which form incomplete features for a prediction

Meanwhile, denormalized datasets:

* may grow to billions of rows, where single updates multiply into mass rewrites
* often span multiple iterations of the schema, with software bugs leaving behind outliers

We think it’s worth attempting to move the machine learning process and modern data architectures beyond the status quo. To that end, we’re building the PostgresML Gym, a free offering, to provide a test bed for real world ML experimentation, in a Postgres database. Your personal Gym will include the PostgresML dashboard, several tutorial notebooks to get you started, and access to your own personal PostgreSQL database, supercharged with our machine learning extension.

Many thanks and ❤️ to all those who are supporting this endeavor. We’d love to hear feedback from the broader ML and Engineering community about applications and other real world scenarios to help prioritize our work.
