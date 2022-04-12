import plpy

class Regression:
    """Provides continuous real number predictions learned from the training data.
    """    
    def __init__(
        model_name: str, 
        relation_name: str, 
        y_column_name: str, 
        implementation: str = "sklearn.linear_model"
    ) -> None:
        """Create a regression model from a table or view filled with training data.

        Args:
            model_name (str): a human friendly identifier
            relation_name (str): the table or view that stores the training data
            y_column_name (str): the column in the training data that acts as the label
            implementation (str, optional): the algorithm used to implement the regression. Defaults to "sklearn.linear_model".
        """

        data_source = f"SELECT * FROM {table_name}"

        # Start training.
        start = plpy.execute(f"""
            INSERT INTO pgml.model_versions
                (name, data_source, y_column)
            VALUES
                ('{table_name}', '{data_source}', '{y}')
            RETURNING *""", 1)

        id_ = start[0]["id"]
        name = f"{table_name}_{id_}"

        destination = models_directory(plpy)

        # Train!
        pickle, msq, r2 = train(plpy.cursor(data_source), y_column=y, name=name, destination=destination)
        X = []
        y = []
        columns = []

        for row in all_rows(cursor):
            row = row.copy()

            if y_column not in row:
                PgMLException(
                    f"Column `{y}` not found. Did you name your `y_column` correctly?"
                )

            y_ = row.pop(y_column)
            x_ = []

            # Always pull the columns in the same order from the row.
            # Python dict iteration is not always in the same order (hash table).
            if not columns:
                for col in row:
                    columns.append(col)

            for column in columns:
                x_.append(row[column])
            X.append(x_)
            y.append(y_)

        X_train, X_test, y_train, y_test = train_test_split(X, y)

        # Just linear regression for now, but can add many more later.
        lr = LinearRegression()
        lr.fit(X_train, y_train)

        # Test
        y_pred = lr.predict(X_test)
        msq = mean_squared_error(y_test, y_pred)
        r2 = r2_score(y_test, y_pred)

        path = os.path.join(destination, name)

        if save:
            with open(path, "wb") as f:
                pickle.dump(lr, f)

        return path, msq, r2


        plpy.execute(f"""
            UPDATE pgml.model_versions
            SET pickle = '{pickle}',
                successful = true,
                mean_squared_error = '{msq}',
                r2_score = '{r2}',
                ended_at = clock_timestamp()
            WHERE id = {id_}""")

        return name

            model
