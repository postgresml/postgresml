# Models

Models are an artifact of calls to `pgml.train`. See [training](/user_guides/training/overview/) for ways to create new models.

![Models](/images/dashboard/model.png)

## Schema

```sql linenums="1" title="pgml.models"
pgml.models(
	id BIGSERIAL PRIMARY KEY,
	project_id BIGINT NOT NULL,
	snapshot_id BIGINT NOT NULL,
	algorithm_name TEXT NOT NULL,
	hyperparams JSONB NOT NULL,
	status TEXT NOT NULL,
	search TEXT,
	search_params JSONB NOT NULL,
	search_args JSONB NOT NULL,
	created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	metrics JSONB,
	CONSTRAINT project_id_fk FOREIGN KEY(project_id) REFERENCES pgml.projects(id),
	CONSTRAINT snapshot_id_fk FOREIGN KEY(snapshot_id) REFERENCES pgml.snapshots(id)
);
```

