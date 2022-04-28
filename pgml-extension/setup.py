import setuptools
import os
from pgml_extension import version

with open(os.path.join(os.path.dirname(__file__), "README.md"), "r") as fh:
    long_description = fh.read()

setuptools.setup(
    name="pgml-extension",
    version=version(),
    author="PostgresML",
    author_email="hello@postgresml.com",
    description="Run machine learning inside Postgres.",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/postgresml/postgresml",
    install_requires=[
        "sklearn",
        "xgboost",
        "diptest",
    ],
    extras_require={"dev": "pytest"},
    packages=setuptools.find_packages(exclude=("tests",)),
    classifiers=[
        "Programming Language :: Python :: 3",
        "Operating System :: OS Independent",
    ],
    python_requires=">=3.7",  # f strings
)
