import psycopg
from psycopg import sql
from psycopg_pool import ConnectionPool

import logging
from rich.logging import RichHandler
from rich.progress import track

from typing import List, Dict, Optional

from .collection import Collection
from .dbutils import (
    run_select_statement,
    run_create_or_insert_statement,
    run_drop_or_delete_statement,
)
import os
import datetime

FORMAT = "%(message)s"
logging.basicConfig(
    level=os.environ.get("LOGLEVEL", "ERROR"),
    format=FORMAT,
    datefmt="[%X]",
    handlers=[RichHandler()],
)
log = logging.getLogger("rich")


class Database:
    def __init__(self, conninfo: str, min_connections: Optional[int] = 1) -> None:
        """
        This function initializes a connection pool and creates a table in a PostgreSQL database if it does
        not already exist.

        :param conninfo: A string containing the connection information for the PostgreSQL database, such
        as the host, port, database name, username, and password
        :type conninfo: str
        :param min_connections: The minimum number of connections that should be maintained in the
        connection pool at all times. If there are no available connections in the pool when a new
        connection is requested, a new connection will be created up to the maximum size of the pool,
        defaults to 1
        :type min_connections: Optional[int] (optional)
        """
        self.conninfo = conninfo
        self.pool = ConnectionPool(conninfo, min_size=min_connections)
        log.info("Creating table pgml.collections")
        create_statement = "CREATE TABLE IF NOT EXISTS pgml.collections \
                            (id  serial8 PRIMARY KEY,\
                            created_at  timestamptz NOT NULL DEFAULT now(),\
                            name  text NOT NULL,\
                            active BOOLEAN DEFAULT TRUE,\
                            UNIQUE (name)\
                            )"
        conn = self.pool.getconn()
        run_create_or_insert_statement(conn, create_statement)
        self.pool.putconn(conn)

    def create_or_get_collection(self, name: str) -> Collection:
        """
        This function creates a new collection in a PostgreSQL database if it does not already exist and
        returns a Collection object.

        :param name: The name of the collection to be created
        :type name: str
        :return: A Collection object is being returned.
        """
        # Get collection names
        conn = self.pool.getconn()
        results = run_select_statement(
            conn, "SELECT name FROM pgml.collections WHERE active = TRUE"
        )
        name = name.lower()
        names = []

        if results:
            names = [res["name"] for res in results]

        if name in names:
            log.info("Collection with name %s already exists.." % name)
        else:
            insert_statement = "INSERT INTO pgml.collections (name) VALUES ('%s')" % (
                name
            )
            run_create_or_insert_statement(conn, insert_statement)

        self.pool.putconn(conn)
        return Collection(self.pool, name)

    def archive_collection(self, name: str) -> None:
        """
        This function deletes a PostgreSQL schema if it exists.

        :param name: The name of the collection (or schema) to be deleted
        :type name: str
        """
        conn = self.pool.getconn()
        cur = conn.cursor()
        timestamp = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
        archive_table_name = name + "_archive_" + timestamp

        results = run_select_statement(conn, "SELECT nspname FROM pg_namespace")
        name_spaces = [res["nspname"] for res in results]

        if name in name_spaces:
            alter_schema_statement = (
                f"ALTER SCHEMA {name} RENAME TO {archive_table_name};"
            )
            cur.execute(alter_schema_statement)

        results = run_select_statement(conn, "SELECT name FROM pgml.collections")
        names = [res["name"] for res in results]

        if name in names:
            update_statement = f"UPDATE pgml.collections SET name = '{archive_table_name}', active = FALSE WHERE name = '{name}'"
            cur.execute(update_statement)

        conn.commit()
        cur.close()

        self.pool.putconn(conn)
