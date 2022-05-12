# Snapshots

Snapshots are an artifact of calls to `pgml.train` that specify the relation_name. See [training](../../guides/training/) for ways to create new snapshots.

![Snapshots](../images/snapshot.png)

## Schema

```sql linenums="1" title="pgml.snapshots"
pgml.snapshots(
	id BIGSERIAL PRIMARY KEY,
	relation_name TEXT NOT NULL,
	y_column_name TEXT[] NOT NULL,
	test_size FLOAT4 NOT NULL,
	test_sampling TEXT NOT NULL,
	status TEXT NOT NULL,
	columns JSONB,
	analysis JSONB,
	created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp(),
	updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT clock_timestamp()
);
```

## Tables

Every snapshot has an accompaning table in the `pgml` schema. For example, the `Snapshot` with `id = 42` has all data recorded in the table `pgml.snaphot_42`. If the test_sampling was `random` for the training, the rows in the table were `ORDER BY random()` when it was created so that future samples can be consistently and efficiently randomized.
 