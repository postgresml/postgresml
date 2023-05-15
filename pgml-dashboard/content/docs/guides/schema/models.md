# Models

Models are an artifact of calls to `pgml.train()`. See [Training Overview](/docs/guides/training/overview/) for ways to create new models.

![Models](/dashboard/static/images/dashboard/model.png)

## Schema

```postgresql
CREATE TABLE IF NOT EXISTS pgml.models(
	id BIGSERIAL PRIMARY KEY,
	project_id BIGINT NOT NULL,
	snapshot_id BIGINT NOT NULL,
	num_features INT NOT NULL,
	algorithm TEXT NOT NULL,
	runtime pgml.runtime DEFAULT 'python'::pgml.runtime,
	hyperparams JSONB NOT NULL,
	status TEXT NOT NULL,
	metrics JSONB,
	search TEXT,
	search_params JSONB NOT NULL,
	search_args JSONB NOT NULL,
	created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	CONSTRAINT project_id_fk FOREIGN KEY(project_id) REFERENCES pgml.projects(id) ON DELETE CASCADE,
	CONSTRAINT snapshot_id_fk FOREIGN KEY(snapshot_id) REFERENCES pgml.snapshots(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS pgml.files(
	id BIGSERIAL PRIMARY KEY,
	model_id BIGINT NOT NULL,
	path TEXT NOT NULL,
	part INTEGER NOT NULL,
	created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	data BYTEA NOT NULL,
	CONSTRAINT model_id_fk FOREIGN KEY(model_id) REFERENCES pgml.models(id) ON DELETE CASCADE
);
```

## Files

Models are partitioned into parts and stored in the `pgml.files` table. Most models are relatively small (just a few megabytes), but some neural networks can grow to gigabytes in size, and would therefore exceed the maximum possible size of a column Postgres.

Partitioning fixes that limitation and allows us to store models up to 32TB in size (or larger, if we employ table partitioning).
