import plpy
import sklearn.datasets

from pgml.sql import q
from pgml.exceptions import PgMLException

def load(source: str):
    if source == "diabetes":
        load_diabetes()
    elif source == "digits":
        load_digits()
    elif source == "iris":
        load_iris()
    elif source == "california_housing":
        load_california_housing()
    else:
        raise PgMLException(f"Invalid dataset name: {source}. Valid values are {{'diabetes', 'digits', 'iris', 'california_housing'}}.")
    return "OK"

def load_diabetes():
    dataset = sklearn.datasets.load_diabetes()
    a = plpy.execute("DROP TABLE IF EXISTS pgml.diabetes")
    a = plpy.execute("""
        CREATE TABLE pgml.diabetes (
            age FLOAT4, 
            sex FLOAT4, 
            bmi FLOAT4, 
            bp FLOAT4, 
            s1 FLOAT4, 
            s2 FLOAT4, 
            s3 FLOAT4, 
            s4 FLOAT4, 
            s5 FLOAT4, 
            s6 FLOAT4, 
            target INTEGER
        )""")
    a = plpy.execute(f"""COMMENT ON TABLE pgml.diabetes IS {q(dataset["DESCR"])}""")

    for X, y in zip(dataset["data"], dataset["target"]):
        plpy.execute(f"""
            INSERT INTO pgml.diabetes (age, sex, bmi, bp, s1, s2, s3, s4, s5, s6, target) 
            VALUES ({q(X[0])}, {q(X[1])}, {q(X[2])}, {q(X[3])}, {q(X[4])}, {q(X[5])}, {q(X[6])}, {q(X[7])}, {q(X[8])}, {q(X[9])}, {q(y)})""")

def load_digits():
    dataset = sklearn.datasets.load_digits()
    a = plpy.execute("DROP TABLE IF EXISTS pgml.digits")
    a = plpy.execute("CREATE TABLE pgml.digits (image SMALLINT[], target INTEGER)")
    a = plpy.execute(f"""COMMENT ON TABLE pgml.digits IS {q(dataset["DESCR"])}""")
    for X, y in zip(dataset["data"], dataset["target"]):
        X = ",".join("%i" % x for x in list(X))
        plpy.execute(f"""INSERT INTO pgml.digits (image, target) VALUES ('{{{X}}}', {y})""")

def load_iris():
    dataset = sklearn.datasets.load_iris()
    a = plpy.execute("DROP TABLE IF EXISTS pgml.iris")
    a = plpy.execute("""
        CREATE TABLE pgml.iris (
            sepal_length FLOAT4, 
            sepal_width FLOAT4, 
            petal_length FLOAT4, 
            petal_width FLOAT4, 
            target INTEGER
        )""")
    a = plpy.execute(f"""COMMENT ON TABLE pgml.iris IS {q(dataset["DESCR"])}""")

    for X, y in zip(dataset["data"], dataset["target"]):
        plpy.execute(f"""
            INSERT INTO pgml.iris (sepal_length, sepal_width, petal_length, petal_width, target) 
            VALUES ({q(X[0])}, {q(X[1])}, {q(X[2])}, {q(X[3])}, {q(X[4])})""")

def load_california_housing():
    dataset = sklearn.datasets.fetch_california_housing()
    a = plpy.execute("DROP TABLE IF EXISTS pgml.california_housing")
    a = plpy.execute("""
        CREATE TABLE pgml.california_housing (
            median_income FLOAT4, -- median income in block group
            house_age FLOAT4, -- median house age in block group
            avg_rooms FLOAT4, -- average number of rooms per household
            avg_bedrooms FLOAT4, -- average number of bedrooms per household
            population FLOAT4, -- block group population
            avg_occupants FLOAT4, -- average number of household members
            latitude FLOAT4, -- block group latitude
            longitude FLOAT4, -- block group longitudetarget INTEGER
            target FLOAT4
        )""")
    a = plpy.execute(f"""COMMENT ON TABLE pgml.california_housing IS {q(dataset["DESCR"])}""")
    for X, y in zip(dataset["data"], dataset["target"]):
        plpy.execute(f"""
            INSERT INTO pgml.california_housing (median_income, house_age, avg_rooms, avg_bedrooms, population, avg_occupants, latitude, longitude, target) 
            VALUES ({q(X[0])}, {q(X[1])}, {q(X[2])}, {q(X[3])}, {q(X[4])}, {q(X[5])}, {q(X[6])}, {q(X[7])}, {q(y)})""")
