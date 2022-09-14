---
hide:
  - navigation
  - toc
  - comments
title: End-to-end Machine Learning for Everyone
---

<style>
.md-typeset h1 {
  font-size: 3em;
  font-weight: 700;
  margin-bottom: -1rem;
  max-width: 80em; 
}

.md-typeset p.subtitle {
  font-weight: 100;
  margin: 2em;
  max-width: 80em; 
}

.md-typeset img {
  margin: 0;
  border-radius: 10px;
}

.md-grid {
  max-width: 100em; 
}

.md-content video, .md-content img {
  max-width: 90%;
  margin: 2em 5%;
}

article.md-content__inner.md-typeset a.md-content__button.md-icon {
  display: none;
}
</style>

<h1 align="center">End-to-end<br/>machine learning platform <br/>for everyone</h1>

<p align="center" class="subtitle">
    Train and deploy models to make online predictions using only SQL, with an open source extension for Postgres. Manage your projects and visualize datasets using the built-in dashboard.
</p>

<p align="center" markdown>
  [Try PostgresML Free!](https://gym.postgresml.org/signup/){ .md-button .md-button--primary }
  [Docs](/user_guides/setup/quick_start_with_docker/){ .md-button }
</p>

<center>
  <video controls autoplay loop muted width="90%" style="box-shadow: 0 0 8px #000;">
    <source src="https://static.postgresml.org/postgresml-org-static/gym_demo.webm" type="video/webm">
    <source src="https://static.postgresml.org/postgresml-org-static/gym_demo.mp4" type="video/mp4">
    <img src="/images/demos/gym_demo.png" alt="PostgresML in practice" loading="lazy">
  </video>
</center>

## Pure SQL Solution

<div class="grid bare" markdown>
  <div class="card" markdown>
```sql linenums="1" title="train.sql"
SELECT pgml.train(
  'My project name', 
  task => 'regression',
  relation_name => 'my_table_with_data',
  y_column_name => 'my_column_with_labels',
  algorithm => 'xgboost' 
);
```
  <p align="center" markdown>
:material-arrow-right: Learn more about [Training](/user_guides/training/overview/)
  </p>
  </div>


  <div class="card" markdown>
![models](/images/dashboard/labels.png)
  </div>

  <div class="card" markdown>
![models](/images/dashboard/models.png)
  </div>

  <div class="card" markdown>
```sql linenums="1" title="deploy.sql"
SELECT pgml.deploy(
  'My project name', 
  strategy => 'most_recent',
  algorithm => 'xgboost'
);
```
  <p align="center" markdown>
:material-arrow-right: Learn more about [Deployments](/user_guides/predictions/deployments/)
  </p>
  </div>

  <div class="card" markdown>
```sql linenums="1" title="predict.sql"
SELECT *, pgml.predict(
  'My project name', 
  ARRAY[...] -- same features used in training
) AS prediction
FROM my_new_unlabeled_table
ORDER BY prediction DESC;
```
  <p align="center" markdown>
:material-arrow-right: Learn more about [Predictions](/user_guides/predictions/overview/)
  </p>
  </div>

  <div class="card" markdown>
![models](/images/dashboard/features.png)
  </div>
</div>

<p align="center" markdown>
  [Get Started](https://gym.postgresml.org/signup/){ .md-button .md-button--primary }
</p>
## What's in the box

<div class="grid" markdown>
  <div class="card" markdown>
:material-lightbulb-group:
__All your favorite algorithms__

Whether you need a simple linear regression, or extreme gradient boosting, we've included support for all classification and regression algorithms in [Scikit Learn](https://scikit-learn.org/), [XGBoost](https://xgboost.readthedocs.io/), [LightGBM](https://lightgbm.readthedocs.io/) and pre-trained deep learning models via [Hugging Face](https://huggingface.co/models).

[:material-arrow-right: Algorithms](/user_guides/training/algorithm_selection/)
  </div>
  <div class="card" markdown>
:fontawesome-solid-arrow-trend-up:
__Instant visualizations__

Run standard analysis on your datasets to detect outliers, bimodal distributions, feature correlation, and other common data visualizations on your datasets. Everything is cataloged in the dashboard for easy reference.

[:material-arrow-right: Dashboard](/user_guides/dashboard/overview/)
  </div>
  <div class="card" markdown>
:material-graph-outline:
__Hyperparameter search__

Use either grid or random searches with cross validation on your training set to discover the most important knobs to tweak on your favorite algorithm.

[:material-arrow-right: Hyperparameter Search](/user_guides/training/hyperparameter_search/)
  </div>
  <div class="card" markdown>
:material-cloud-outline:
__Online and offline support__

Predictions are served via a standard Postgres connection to ensure that your core apps can always access both your data and your models in real time. Pure SQL workflows also enable batch predictions to cache results in native Postgres tables for lookup.

[:material-arrow-right: Predictions](/user_guides/predictions/overview/)
  </div>
  <div class="card" markdown>
:material-arrow-top-right-thin:
__SQL native vector operations__

Vector operations make working with learned emebeddings a snap, for things like nearest neighbor searches or other similarity comparisons.

[:material-arrow-right: Vector Operations](/user_guides/vector_operations/overview/)
  </div>
  <div class="card" markdown>
:material-clipboard-check:
__Managed model deployments__

Models can be periodically retrained and automatically promoted to production depending on their key metric. Rollback capability is provided to ensure that you're always able to serve the highest quality predictions, along with historical logs of all deployments for long term study.
  
[:material-arrow-right: Deployments](/user_guides/predictions/deployments/)
  </div>
  <div class="card" markdown>
:fontawesome-solid-link:
__The performance of Postgres__

Since your data never leaves the database, you retain the speed, reliability and security you expect in your foundational stateful services. Leverage your existing infrastructure and the data distribution strategies native to PostgreSQL to deliver new capabilities.

[:material-arrow-right: Distributed Training](/user_guides/setup/distributed_training/)
  </div>
  <div class="card" markdown>
:fontawesome-solid-envelope-open-text:
__Open source__

We're building on the shoulders of giants. These machine learning libraries and Postgres have received extensive academic and industry use, and we'll continue their tradition to build with the community.

[:material-arrow-right: MIT License](/about/license/)
  </div>
</div>
