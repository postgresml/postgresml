import os
import setuptools

from pgml_extension import __version__

with open(os.path.join(os.path.dirname(__file__), "README.md"), "r") as fh:
    long_description = fh.read()

setuptools.setup(
    name="pgml-extension",
    version=__version__,
    author="PostgresML Team",
    author_email="maintainers@postgresml.org",
    description="Simple machine learning in PostgreSQL.",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/postgresml/postgresml",
    install_requires=[
        "diptest",
        "numpy",
        "pandas",
        "sklearn",
        "xgboost",
        "lightgbm",
        "psycopg2",
        "wheel",
        "click",
        "torch",
        "transformers",
        "datasets",
        # translation/nlp
        "sentencepiece",
        "sacremoses",
        "sacrebleu",
        "rouge",
    ],
    extras_require={"dev": "pytest"},
    packages=setuptools.find_packages(exclude=("tests",)),
    package_data={"pgml_extension": ["sql/install/*.sql"]},
    include_package_data=True,
    classifiers=[
        "Programming Language :: Python :: 3",
        "Operating System :: OS Independent",
    ],
    python_requires=">=3.7",
)
