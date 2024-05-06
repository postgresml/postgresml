---
description: Decompose an input vector into it's principal components
---

# pgml.decompose()


Chunks are pieces of documents split using some specified splitter. This is typically done before embedding.

## API

```sql
pgml.decompose(
    project_name TEXT, -- project name
    vector REAL[]      -- features to decompose
)
```

### Parameters

| Parameter      | Example                         | Description                                              |
|----------------|---------------------------------|----------------------------------------------------------|
| `project_name` | `'My First PostgresML Project'` | The project name used to train models in `pgml.train()`. |
| `vector`       | `ARRAY[0.1, 0.45, 1.0]`         | The feature vector that needs decomposition.             |

## Example

```sql
SELECT pgml.decompose('My PCA', ARRAY[0.1, 2.0, 5.0]);
```

!!! example

```sql
SELECT *,
    pgml.decompose(
        'Buy it Again',
        ARRAY[
            user.location_id,
            NOW() - user.created_at,
            user.total_purchases_in_dollars
        ]
    ) AS buying_score
FROM users
WHERE tenant_id = 5
ORDER BY buying_score
LIMIT 25;
```

!!!