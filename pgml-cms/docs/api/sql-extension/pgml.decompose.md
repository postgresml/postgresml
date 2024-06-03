---
description: Decompose an input vector into it's principal components
---

# pgml.decompose()

Matrix decomposition reduces the number of dimensions in a vector, to improve relevance and reduce computation required. 

## API

```postgresql
pgml.decompose(
    project_name TEXT, -- project name
    vector REAL[]      -- features to decompose
)
```

### Parameters

| Parameter      | Example                         | Description                                                             |
|----------------|---------------------------------|-------------------------------------------------------------------------|
| `project_name` | `'My First PostgresML Project'` | The project name used to train a decomposition model in `pgml.train()`. |
| `vector`       | `ARRAY[0.1, 0.45, 1.0]`         | The feature vector to transform.                                        |

## Example

```postgresql
SELECT pgml.decompose('My PCA', ARRAY[0.1, 2.0, 5.0]);
```
