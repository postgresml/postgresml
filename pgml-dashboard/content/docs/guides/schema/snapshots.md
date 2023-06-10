# Snapshots

Snapshots are an artifact of calls to `pgml.train()` that specify the `relation_name` and `y_column_name` parameters. See [Training Overview](/docs/guides/training/overview/) for ways to create new snapshots.

![Snapshots](/dashboard/static/images/dashboard/snapshot.png)

## Schema

```postgresql
CREATE TABLE IF NOT EXISTS pgml.snapshots(
	id BIGSERIAL PRIMARY KEY,
	relation_name TEXT NOT NULL,
	y_column_name TEXT[] NOT NULL,
	test_size FLOAT4 NOT NULL,
	test_sampling pgml.sampling NOT NULL,
	status TEXT NOT NULL,
	columns JSONB,
	analysis JSONB,
	created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp()
);
```

## Snapshot Storage

Every snapshot has an accompanying table in the `pgml` schema. For example, the snapshot with the primary key `42` has all data saved in the `pgml.snaphot_42` table.

If the `test_sampling` was set to `random` during training, the rows in the table are ordered using `ORDER BY RANDOM()`, so that future samples can be consistently and efficiently randomized.
