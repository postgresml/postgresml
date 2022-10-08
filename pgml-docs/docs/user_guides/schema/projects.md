# Projects

Projects are an artifact of calls to `pgml.train`. See [training](/user_guides/training/overview/) for ways to create new projects.

![Projects](/images/dashboard/project.png)

## Schema

```sql linenums="1" title="pgml.projects"
pgml.projects(
	id BIGSERIAL PRIMARY KEY,
	name TEXT NOT NULL,
	task pgml.task NOT NULL,
	created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp()
);
```
