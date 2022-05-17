# Dashboard

PostgresML comes with an app to provide visibility into models and datasets in your database. If you're running the standard docker container, you can view it running on [http://localhost:8000/](http://localhost:8000/). Since your `pgml` schema starts empty, there isn't much to see. If you'd like to generate some examples, you can run the test suite against your database. 

## Generate example data

The test suite for PostgresML is composed by running the sql files in the [examples directory](https://github.com/postgresml/postgresml/tree/master/pgml-extension/examples). You can use these examples to populate your local installation with some seed data. The test suite only operates on the `pgml` schema, and is otherwise isolated from the rest of the Postgres cluster.

```bash
$ psql -f pgml-extension/sql/test.sql -P pager postgres://postgres@127.0.0.1:5433/pgml_development
```

## Overview
Now there should be something to see in your local dashboard.

### Projects
Projects organize Models that are all striving toward the same objective. They aren't much more than a name to group a collection of models. You can see the currently deployed model for each project indicated by :material-star:.

![Project](../images/project.png)

### Models
Models are the result of training an algorithm on a Snapshot of a dataset. They record `metrics` depending on their projects objective, and are scored accordingly. Some models are the result of a hyperparameter search, and include additional analysis on the range of hyperparameters they are tested against.

![Model](../images/model.png)

### Snapshots
A Snapshot is created during training runs to record the data used for further analysis, or to train additional models against identical data.

![Snapshot](../images/snapshot.png)

### Deployments
Every deployment is recorded to track models over time.

![Deployment](../images/deployment.png)

