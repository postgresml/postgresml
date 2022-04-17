import plpy
import sklearn.datasets

from pgml.sql import q
from pgml.exceptions import PgMLException

def load(source: str):
    if source == "digits":
        load_digits()
    elif source == "california_housing":
        load_california_housing()
    else:
        raise PgMLException(f"Invalid dataset name: {source}. Valid values are ['digits'].")
    return "OK"

def load_digits():
    dataset = sklearn.datasets.load_digits()
    a = plpy.execute("DROP TABLE IF EXISTS pgml.digits")
    a = plpy.execute("CREATE TABLE pgml.digits (image SMALLINT[], target INTEGER)")
    a = plpy.execute(f"""COMMENT ON TABLE pgml.digits IS {q(dataset["DESCR"])}""")
    for X, y in zip(dataset["data"], dataset["target"]):
        X = ",".join("%i" % x for x in list(X))
        plpy.execute(f"""INSERT INTO pgml.digits (image, target) VALUES ('{{{X}}}', {y})""")

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
    