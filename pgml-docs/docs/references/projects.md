# Projects

Projects are an artifact of calls to `pgml.train`. See [training](/guides/training/) for ways to create new projects.

![Projects](/images/project.png)

## Schema

```sql linenums="1" title="pgml.projects"
pgml.projects(
	id BIGSERIAL PRIMARY KEY,
	name TEXT NOT NULL,
	objective TEXT NOT NULL,
	created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp()
);
```
