import plpy
from sklearn.datasets import load_digits as d

from pgml.sql import q
from pgml.exceptions import PgMLException

def load(source: str):
    if source == "digits":
        load_digits()
    else:
        raise PgMLException(f"Invalid dataset name: {source}. Valid values are ['digits'].")
    return "OK"

def load_digits():
    dataset = d()
    a = plpy.execute("DROP TABLE IF EXISTS pgml.digits")
    a = plpy.execute("CREATE TABLE pgml.digits (image SMALLINT[], target INTEGER)")
    a = plpy.execute(f"""COMMENT ON TABLE pgml.digits IS {q(dataset["DESCR"])}""")
    for X, y in zip(dataset["data"], dataset["target"]):
        X = ",".join("%i" % x for x in list(X))
        plpy.execute(f"""INSERT INTO pgml.digits (image, target) VALUES ('{{{X}}}', {y})""")
