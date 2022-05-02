import plpy
import sklearn.datasets

from pgml_extension.sql import q
from pgml_extension.exceptions import PgMLException


def load(source: str):
    if source == "diabetes":
        load_diabetes()
    elif source == "digits":
        load_digits()
    elif source == "iris":
        load_iris()
    elif source == "linnerud":
        load_linnerud()
    elif source == "wine":
        load_wine()
    elif source == "breast_cancer":
        load_breast_cancer()
    elif source == "california_housing":
        load_california_housing()
    else:
        raise PgMLException(
            f"Invalid dataset name: {source}. Valid values are {{'diabetes', 'digits', 'iris', 'linnerud', 'wine', 'breast_cancer', 'california_housing'}}."
        )
    return "OK"


def load_diabetes():
    dataset = sklearn.datasets.load_diabetes()
    a = plpy.execute("DROP TABLE IF EXISTS pgml.diabetes")
    a = plpy.execute(
        """
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
        )"""
    )
    a = plpy.execute(f"""COMMENT ON TABLE pgml.diabetes IS {q(dataset["DESCR"])}""")

    for X, y in zip(dataset["data"], dataset["target"]):
        plpy.execute(
            f"""
            INSERT INTO pgml.diabetes (age, sex, bmi, bp, s1, s2, s3, s4, s5, s6, target) 
            VALUES ({",".join("%f" % x for x in list(X))}, {q(y)})"""
        )


def load_digits():
    dataset = sklearn.datasets.load_digits()
    a = plpy.execute("DROP TABLE IF EXISTS pgml.digits")
    a = plpy.execute("CREATE TABLE pgml.digits (image SMALLINT[][], target INTEGER)")
    a = plpy.execute(f"""COMMENT ON TABLE pgml.digits IS {q(dataset["DESCR"])}""")
    for X, y in zip(dataset["data"], dataset["target"]):
        height = width = 8
        image = [[0 for x in range(width)] for y in range(height)] 
        for i, x in enumerate(list(X)):
            image[int(i / height)][int(i % width)] = x
        sql_image = "ARRAY[" + ",".join(["ARRAY[" + ",".join("%i" % x for x in row) + "]" for row in image]) + "]"
            
        plpy.execute(
            f"""
            INSERT INTO pgml.digits (image, target) 
            VALUES ({sql_image}, {y})
        """
        )


def load_iris():
    dataset = sklearn.datasets.load_iris()
    a = plpy.execute("DROP TABLE IF EXISTS pgml.iris")
    a = plpy.execute(
        """
        CREATE TABLE pgml.iris (
            sepal_length FLOAT4, 
            sepal_width FLOAT4, 
            petal_length FLOAT4, 
            petal_width FLOAT4, 
            target INTEGER
        )"""
    )
    a = plpy.execute(f"""COMMENT ON TABLE pgml.iris IS {q(dataset["DESCR"])}""")

    for X, y in zip(dataset["data"], dataset["target"]):
        plpy.execute(
            f"""
            INSERT INTO pgml.iris (sepal_length, sepal_width, petal_length, petal_width, target) 
            VALUES ({",".join("%f" % x for x in list(X))}, {q(y)})"""
        )


def load_linnerud():
    dataset = sklearn.datasets.load_linnerud()
    a = plpy.execute("DROP TABLE IF EXISTS pgml.linnerud")
    a = plpy.execute(
        """
        CREATE TABLE pgml.linnerud(
            chins FLOAT4,
            situps FLOAT4,
            jumps FLOAT4,
            weight FLOAT4,
            waste FLOAT4,
            pulse FLOAT4
        )"""
    )
    a = plpy.execute(f"""COMMENT ON TABLE pgml.linnerud IS {q(dataset["DESCR"])}""")

    for X, y in zip(dataset["data"], dataset["target"]):
        plpy.execute(
            f"""
            INSERT INTO pgml.linnerud (chins, situps, jumps, weight, waste, pulse) 
            VALUES ({",".join("%f" % x for x in list(X))}, {q(y[0])}, {q(y[1])}, {q(y[2])})"""
        )


def load_wine():
    dataset = sklearn.datasets.load_wine()
    a = plpy.execute("DROP TABLE IF EXISTS pgml.wine")
    a = plpy.execute(
        """
        CREATE TABLE pgml.wine (
            alcohol FLOAT4, 
            malic_acid FLOAT4, 
            ash FLOAT4, 
            alcalinity_of_ash FLOAT4,
            magnesium FLOAT4,
            total_phenols FLOAT4,
            flavanoids FLOAT4,
            nonflavanoid_phenols FLOAT4,
            proanthocyanins FLOAT4,
            color_intensity FLOAT4,
            hue FLOAT4,
            "od280/od315_of_diluted_wines" FLOAT4,
            proline FLOAT4,
            target INT
        )"""
    )
    a = plpy.execute(f"""COMMENT ON TABLE pgml.wine IS {q(dataset["DESCR"])}""")

    for X, y in zip(dataset["data"], dataset["target"]):
        plpy.execute(
            f"""
            INSERT INTO pgml.wine (alcohol, malic_acid, ash, alcalinity_of_ash, magnesium, total_phenols, flavanoids, nonflavanoid_phenols, proanthocyanins, color_intensity, hue, "od280/od315_of_diluted_wines", proline, target) 
            VALUES ({",".join("%f" % x for x in list(X))}, {q(y)})"""
        )


def load_breast_cancer():
    dataset = sklearn.datasets.load_breast_cancer()
    a = plpy.execute("DROP TABLE IF EXISTS pgml.breast_cancer")
    a = plpy.execute(
        """
        CREATE TABLE pgml.breast_cancer (
            "mean radius" FLOAT4, 
            "mean texture" FLOAT4, 
            "mean perimeter" FLOAT4, 
            "mean area" FLOAT4,
            "mean smoothness" FLOAT4,
            "mean compactness" FLOAT4,
            "mean concavity" FLOAT4,
            "mean concave points" FLOAT4,
            "mean symmetry" FLOAT4,
            "mean fractal dimension" FLOAT4,
            "radius error" FLOAT4,
            "texture error" FLOAT4,
            "perimeter error" FLOAT4,
            "area error" FLOAT4,
            "smoothness error" FLOAT4,
            "compactness error" FLOAT4,
            "concavity error" FLOAT4,
            "concave points error" FLOAT4,
            "symmetry error" FLOAT4,
            "fractal dimension error" FLOAT4,
            "worst radius" FLOAT4,
            "worst texture" FLOAT4,
            "worst perimeter" FLOAT4,
            "worst area" FLOAT4,
            "worst smoothness" FLOAT4,
            "worst compactness" FLOAT4,
            "worst concavity" FLOAT4,
            "worst concave points" FLOAT4,
            "worst symmetry" FLOAT4,
            "worst fractal dimension" FLOAT4,
            "perimeter" FLOAT4,
            "malignant" BOOLEAN
        )"""
    )
    a = plpy.execute(f"""COMMENT ON TABLE pgml.breast_cancer IS {q(dataset["DESCR"])}""")

    for X, y in zip(dataset["data"], dataset["target"]):
        plpy.execute(
            f"""
            INSERT INTO pgml.breast_cancer ("mean radius", "mean texture", "mean perimeter", "mean area", "mean smoothness", "mean compactness", "mean concavity", "mean concave points", "mean symmetry", "mean fractal dimension", "radius error", "texture error", "perimeter error", "area error", "smoothness error", "compactness error", "concavity error", "concave points error", "symmetry error", "fractal dimension error", "worst radius", "worst texture", "worst perimeter", "worst area", "worst smoothness", "worst compactness", "worst concavity", "worst concave points", "worst symmetry", "worst fractal dimension", "malignant") 
            VALUES ({",".join("%f" % x for x in list(X))}, {q(y) == 0})"""
        )


def load_california_housing():
    dataset = sklearn.datasets.fetch_california_housing()
    a = plpy.execute("DROP TABLE IF EXISTS pgml.california_housing")
    a = plpy.execute(
        """
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
        )"""
    )
    a = plpy.execute(f"""COMMENT ON TABLE pgml.california_housing IS {q(dataset["DESCR"])}""")
    for X, y in zip(dataset["data"], dataset["target"]):
        plpy.execute(
            f"""
            INSERT INTO pgml.california_housing (median_income, house_age, avg_rooms, avg_bedrooms, population, avg_occupants, latitude, longitude, target) 
            VALUES ({q(X[0])}, {q(X[1])}, {q(X[2])}, {q(X[3])}, {q(X[4])}, {q(X[5])}, {q(X[6])}, {q(X[7])}, {q(y)})"""
        )
