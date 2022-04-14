--
-- CREATE EXTENSION
--
CREATE EXTENSION IF NOT EXISTS plpython3u;

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
RETURNS BYTEA
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

    return pickle.dumps(rfc)

$$ LANGUAGE plpython3u;

;

CREATE OR REPLACE FUNCTION scikit_learn_predict_example(model BYTEA, value INT)
RETURNS DOUBLE PRECISION
AS $$
    import pickle

    m = pickle.loads(model)

    r = m.predict([[value,]])
    return r[0]
$$ LANGUAGE plpython3u;

WITH model as (
    SELECT scikit_learn_train_example() AS pickle
)
SELECT value,
       weight,
       scikit_learn_predict_example((SELECT model.pickle FROM model), value::int) AS prediction
FROM scikit_train_view LIMIT 5;
