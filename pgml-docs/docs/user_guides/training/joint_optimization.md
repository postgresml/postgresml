# Joint Optimization

Some algorithms support joint optimization of the objective across multiple outputs, and can improve results compared to using multiple independent models. To leverage multiple outputs in PostgresML, you'll need to substitue the standard usage of `pgml.train` and `pgml.predict` with `pgml.train_multi` and `pgml.predict_multi`. The `_multi` functions are identical, except `train_multi` takes an array of `y_column_names TEXT[]`, and `predict_multi` returns an array of outputs correspondingly.

Read more at [scikit-learn](https://scikit-learn.org/stable/modules/classes.html#module-sklearn.multioutput).