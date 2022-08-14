"""
Install the extension into the database.
"""
import os
import sys
import psycopg2
import click
import pkg_resources


@click.command()
@click.option("--database-url", required=True, help="Connection string for the database.")
def main(database_url):
    conn = psycopg2.connect(database_url)
    cur = conn.cursor()

    for f in [
        "sql/install/schema.sql",
        "sql/install/models.sql",
        "sql/install/vectors.sql",
        "sql/install/transformers.sql",
    ]:
        data = pkg_resources.resource_string("pgml_extension", f)
        cur.execute(data)
        conn.commit()


main()
