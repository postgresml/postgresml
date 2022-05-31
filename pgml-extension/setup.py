import os
import setuptools

from pgml_extension import version

with open(os.path.join(os.path.dirname(__file__), "README.md"), "r") as fh:
    long_description = fh.read()

setuptools.setup(
    name="pgml-extension",
    version=version(),
    author="PostgresML Team",
    author_email="maintainers@postgresml.org",
    description="Simple machine learning in PostgreSQL.",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/postgresml/postgresml",
    install_requires=[
        "diptest",
        "sklearn",
        "xgboost",
        "lightgbm",
        "diptest",
        "psycopg2",
        "wheel",
        "click",
    ],
    extras_require={"dev": "pytest"},
    packages=setuptools.find_packages(exclude=("tests",)),
    package_data={
        'pgml_extension': ["sql/install/*.sql"]
    },
    include_package_data=True,
    classifiers=[
        "Programming Language :: Python :: 3",
        "Operating System :: OS Independent",
    ],
    python_requires=">=3.7", 
)
