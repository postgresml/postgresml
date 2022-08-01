---
hide:
  - navigation
---

<style>
img.float-right {
  margin: 0 16px !important;
  max-width: 50%  !important;
}
img.center {
  margin: 16px 12.5%;
  max-width: 75%;
}
</style>

Announcing the PostgresML Gym! 🎉
================================

A common problem with data science and machine learning tutorials is the published example data is usually nothing like what you’ll find in industry.

![tabular data](/images/illustrations/table.png){.float-right}

- It’s usually denormalized into a single tabular form, e.g. csv file
- It’s relatively tiny to medium amounts of data, not big data
- It’s static, new rows are never added
- It’s often been pre-treated to clean or simplify the data

As Data Science transitions from academia into industry, their norms influence organizations, applications and deployments. Professional Data Scientists need teams of Data Engineers to move the data from production databases into centralized data warehouses and denormalized schemas that they are more familiar with. Large offline batch jobs are a typical integration point between Data Scientists and their Engineering counterparts who deal with online systems. As the systems grow more complex, additional specialized Machine Learning Engineers are required to optimize performance and scalability bottlenecks between databases, warehouses, models and applications.

This eventually leads to expensive maintenance, and then to terminal complexity where new improvements to the system become exponentially more difficult. Ultimately, previously working models start getting replaced by simpler solutions, so the business can continue to iterate. This is not a new phenomenon, see the fate of the Netflix Prize.

Instead of starting from the academic perspective that data is dead, PostgresML embraces the living and dynamic nature of data inside modern organizations. It's relational.

![relational data](/images/illustrations/uml.png){.center}

- Schemas are normalized for OLTP use cases and real time performance considerations
- New rows are constantly added and updated, which form incomplete features for a prediction
- Denormalized datasets may grow to billions of rows, and terabytes of data
- The data often spans multiple versions of the schema, and bugs can introduce outliers
- Modern applications are interactive and real time in nature, rather than static and precomputable

These are the types of considerations that might make some Data Scientists and Statisticians sigh, and maybe even twitch a little bit, but they are the bread and butter of Software Engineering. It’s hard to teach Machine Learning with these considerations, because these types of datasets are non trivial or even illegal to distribute outside of their typically proprietary applications. 

We think it’s worth attempting to move the learning process in industry beyond the status quo. To that end, we’re building the PostgresML Gym to generate realistic application data in a Postgres database. For now, the gym starts as an empty Postgres database after you sign up, but you can start loading those familiar academic data sets with calls to pgml.load_data() in your own [Dashboard](/users_guides/overview.md).

We'll be publishing a series of blog posts detailing common machine learning applications, that demonstrate the differences in OLTP ML vs conventional data warehouse centric approaches, and the advantages that PostgresML can provide for industrial applications. We’d also love to hear feedback from the broader ML and Engineering community about applications and other real world scenarios to help prioritize our work. 

<p align="center" markdown>
  [Sign up for the Gym](https://gym.postgresml.org/){ .md-button .md-button--primary }
</p>
