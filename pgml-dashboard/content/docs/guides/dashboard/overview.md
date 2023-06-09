# Dashboard

PostgresML comes with a web app to provide visibility into models and datasets in your database. If you're running [our Docker container](/docs/guides/setup/quick_start_with_docker/), you can view it running on [http://localhost:8000/](http://localhost:8000/).


## Generate example data

The test suite for PostgresML is composed by running the SQL files in the [examples directory](https://github.com/postgresml/postgresml/tree/master/pgml-extension/examples). You can use these examples to populate your local installation with some test data. The test suite only operates on the `pgml` schema, and is otherwise isolated from the rest of the PostgresML installation.

```bash
psql -f pgml-extension/sql/test.sql \
     -P pager \
     postgres://postgres@127.0.0.1:5433/pgml_development
```

### Projects

Projects organize Models that are all striving toward the same task. They aren't much more than a name to group a collection of models. You can see the currently deployed model for each project indicated by a <span class="material-symbols-outlined">star</span>.

![Project](/dashboard/static/images/dashboard/project.png)

### Models

Models are the result of training an algorithm on a snapshot of a dataset. They record metrics depending on their projects task, and are scored accordingly. Some models are the result of a hyperparameter search, and include additional analysis on the range of hyperparameters they are tested against.

![Model](/dashboard/static/images/dashboard/model.png)

### Snapshots

A snapshot is created during training runs to record the data used for further analysis, or to train additional models against identical data.

![Snapshot](/dashboard/static/images/dashboard/snapshot.png)

### Deployments

Every deployment is recorded to track models over time.

![Deployment](/dashboard/static/images/dashboard/deployment.png)

