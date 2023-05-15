# Projects

Projects are an artifact of calls to `pgml.train()`. See [Training Overview](/docs/guides/training/overview/) for ways to create new projects.

![Projects](/dashboard/static/images/dashboard/project.png)

## Schema

```postgresql
CREATE TABLE IF NOT EXISTS pgml.projects(
	id BIGSERIAL PRIMARY KEY,
	name TEXT NOT NULL,
	task pgml.task NOT NULL,
	created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp()
);
```
