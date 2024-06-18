import os

DATABASE_URL = os.environ.get("PGML_DATABASE_URL", "postgres:///pgml")
