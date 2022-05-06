# Snapshots

Snapshots are an artifact of calls to `pgml.train` that specify the relation_name. See [training](/guides/training/) for ways to create new snapshots.

![Snapshots](/images/snapshot.png)

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
