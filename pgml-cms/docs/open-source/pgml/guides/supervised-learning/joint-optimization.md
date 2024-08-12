# Joint Optimization

Some algorithms support joint optimization of the task across multiple outputs, which can improve results compared to using multiple independent models.

To leverage multiple outputs in PostgresML, you'll need to substitute the standard usage of `pgml.train()` with `pgml.train_joint()`, which has the same API, except the notable exception of `y_column_name` parameter, which now accepts an array instead of a simple string.

```postgresql
SELECT * FROM pgml.train_join(
    'My Joint Project',
    task => 'regression',
    relation_name => 'my_table',
    y_column_name => ARRAY['target_a', 'target_b'],
);
```

You can read more in [scikit-learn](https://scikit-learn.org/stable/modules/classes.html#module-sklearn.multioutput) documentation.
