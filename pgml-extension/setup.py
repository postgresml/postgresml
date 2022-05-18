import os
import setuptools
import sys

from setuptools.command.install import install
from setuptools.command.develop import develop 
from pathlib import Path

from pgml_extension import version

class InstallCommand(install):
    description = "Installs the pgml-extension in the database."
    user_options = install.user_options + [
        ("database-url=", None, "Specify the PostgreSQL database url."),
    ]

    def initialize_options(self):
        install.initialize_options(self)
        self.database_url = None

    def finalize_options(self):
        install.finalize_options(self)
        assert self.database_url is None or self.database_url.startswith("postgres://"), f'Invalid database_url={self.database_url}'

    def run(self):
        install.run(self)
        if self.database_url:
            install_sql(os.path.join(os.path.dirname(__file__), "sql/install.sql"), self.database_url)

class DevelopCommand(develop):
    description = "Installs the pgml-extension in the database."
    user_options = develop.user_options + [
        ("database-url=", None, "Specify the PostgreSQL database url."),
    ]

    def initialize_options(self):
        develop.initialize_options(self)
        self.database_url = None

    def finalize_options(self):
        develop.finalize_options(self)
        assert self.database_url is None or self.database_url.startswith("postgres://"), f'Invalid database_url={self.database_url}'

    def run(self):
        develop.run(self)
        if self.database_url:
            install_sql(os.path.join(os.path.dirname(__file__), "sql/test.sql"), self.database_url)

def install_sql(filename, database_url):
    if database_url is None:
        print(f"WARNING: No --database_url has been set. Skipping database installation.")

    command = f"psql -f {filename} {database_url} -P pager"
    print(f"Running {command}")
    exit_status = os.system(command)
    if exit_status != 0:
        message = f"""
        
        Failure running `{command}`.

        The installation failed installing the extension inside PostgreSQL,
        Can you connect to the {database_url} locally? e.g.

        psql {database_url}
        
        """

        sys.exit(message)

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
    cmdclass={
        'develop': DevelopCommand,
        'install': InstallCommand,
    },
    install_requires=[
        "diptest",
        "sklearn",
        "xgboost",
        "lightgbm",
    ],
    extras_require={"dev": "pytest"},
    packages=setuptools.find_packages(exclude=("tests",)),
    package_data={
        '': ['MIT-LICENSE.txt'],
        'sql': [path.as_posix() for path in Path(".").rglob("*.sql")]
    },
    include_package_data=True,
    classifiers=[
        "Programming Language :: Python :: 3",
        "Operating System :: OS Independent",
    ],
    python_requires=">=3.7", 
)
