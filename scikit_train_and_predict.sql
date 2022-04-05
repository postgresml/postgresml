--
-- Data table.
--
DROP TABLE IF EXISTS scikit_train_data CASCADE;
CREATE TABLE scikit_train_data (
    id BIGSERIAL PRIMARY KEY,
    value BIGINT,
    weight DOUBLE PRECISION
);

--
-- View of the data table, just to demonstrate that views work.
--
DROP VIEW IF EXISTS scikit_train_view;
CREATE VIEW scikit_train_view AS SELECT * FROM scikit_train_data;

--
-- Insert some dummy data into the data table.
--
INSERT INTO scikit_train_data (value, weight) SELECT generate_series(1, 500), 5.0;


CREATE OR REPLACE FUNCTION scikit_learn_train_example()
RETURNS TEXT
AS $$
    from sklearn.ensemble import RandomForestClassifier
    import pickle

    cursor = plpy.cursor("SELECT value, weight FROM scikit_train_view")
    X = []
    y = []

    while True:
        rows = cursor.fetch(5)
        if not rows:
            break
        for row in rows:
            X.append([row["value"],])
            y.append(row["weight"])
    rfc = RandomForestClassifier()
    rfc.fit(X, y)

    with open("/tmp/postgresml-rfc.pickle", "wb") as f:
        pickle.dump(rfc, f)
    return "OK"

$$ LANGUAGE plpython3u;

SELECT scikit_learn_train_example();

CREATE OR REPLACE FUNCTION scikit_learn_predict_example(value INT)
RETURNS DOUBLE PRECISION
AS $$
    import pickle

    with open("/tmp/postgresml-rfc.pickle", "rb") as f:
        m = pickle.load(f)

    r = m.predict([[value,]])
    return r[0]
$$ LANGUAGE plpython3u;

SELECT value,
       weight,
       scikit_learn_predict_example(value::int) AS prediction
FROM scikit_train_view LIMIT 5;
