<style>
img.float-right {
  margin: 0 16px !important;
  max-width: 50%  !important;
  float: right;
}
img.float-left {
  margin: 0 16px !important;
  max-width: 60%  !important;
  float: left;
}
img.center {
  margin: 16px 12.5%;
  max-width: 75%;
}
</style>

Data is Living and Relational
================================

<div class="author">
  <img width="54px" height="54px" src="/images/team/montana.jpg" />
  <p>Montana Low</p>
  <p class="date">August 25, 2022</p>
</div>


A common problem with data science and machine learning tutorials is the published and studied data sets are often nothing like what you’ll find in industry.

<center markdown>

  | width | height | area  |
  | ----- | ------ | ----- |
  | 1 | 1 | 1 |
  | 2 | 1 | 2 |
  | 2 | 2 | 4 |

</center>

- It’s usually denormalized into a single tabular form, e.g. csv file
- It’s often relatively tiny to medium amounts of data, not big data
- It’s always static, new rows are never added
- It’s sometimes been pre-treated to clean or simplify the data

As Data Science transitions from academia into industry, those norms influence organizations and applications. Professional Data Scientists now need teams of Data Engineers to move the data from production databases into centralized data warehouses and denormalized schemas that are more familiar, and ideally easier to work with. Large offline batch jobs are a typical integration point between Data Scientists and their Engineering counterparts who deal with online systems. As the systems grow more complex, additional specialized Machine Learning Engineers are required to optimize performance and scalability bottlenecks between databases, warehouses, models and applications.

This eventually leads to expensive maintenance and then to terminal complexity where new improvements to the system become exponentially more difficult. Ultimately, previously working models start getting replaced by simpler solutions, so the business can continue to iterate. This is not a new phenomenon, see the fate of the Netflix Prize.

Announcing the PostgresML Gym 🎉
-------------------------------

Instead of starting from the academic perspective that data is dead, PostgresML embraces the living and dynamic nature of data inside modern organizations. It's relational and growing in multiple dimensions.

![relational data](/images/illustrations/uml.png)

- Schemas are normalized for real time performance and correctness considerations
- New rows are constantly added and updated, which form the incomplete features for a prediction
- Denormalized datasets may grow to billions of rows, and terabytes of data
- The data often spans multiple iterations of the schema, and software bugs can introduce outlier data

We think it’s worth attempting to move the machine learning process and modern data architectures beyond the status quo. To that end, we’re building the PostgresML Gym to provide a test bed for real world ML experimentation in a Postgres database. Your personal gym will include the PostgresML dashboard and several tutorial notebooks to get you started.

<center>
  <video autoplay loop muted width="90%" style="box-shadow: 0 0 8px #000;">
    <source src="https://static.postgresml.org/postgresml-org-static/gym_demo.webm" type="video/webm">
    <source src="https://static.postgresml.org/postgresml-org-static/gym_demo.mp4" type="video/mp4">
    <img src="/images/demos/gym_demo.png" alt="PostgresML in practice" loading="lazy">
  </video>
</center>

<center markdown>
  [Try the PostgresML Gym](https://gym.postgresml.org/){ .md-button .md-button--primary }
</center>

We’d love to hear feedback from the broader ML and Engineering community about applications and other real world scenarios to help prioritize our work. 
