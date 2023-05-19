import psycopg
from psycopg import sql
from psycopg_pool import ConnectionPool

import logging
from rich.logging import RichHandler
from rich.progress import track

from typing import List, Dict, Optional

from .collection import Collection
from .dbutils import *
import os

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
        """Initialize Database object
        Creates pgml.collections table if it doesn't exist

        Args:
            conninfo (str) : Postgres connection info postgresql://username:password@host:port/db

        Returns:
            None
        """
        self.conninfo = conninfo
        self.pool = ConnectionPool(conninfo, min_size=min_connections)
        log.info("Creating table pgml.collections")
        create_statement = "CREATE TABLE IF NOT EXISTS pgml.collections \
                            (id  serial8 PRIMARY KEY,\
                            created_at  timestamptz NOT NULL DEFAULT now(),\
                            name  text NOT NULL)"
        conn = self.pool.getconn()
        run_create_or_insert_statement(conn, create_statement)
        self.pool.putconn(conn)

    def create_collection(self, name: str) -> Collection:
        # Get collection names
        conn = self.pool.getconn()
        results = run_select_statement(conn, "SELECT name FROM pgml.collections")
        name = name.lower()
        names = [res["name"] for res in results]

        if name in names:
            log.info("Collection with name %s already exists.." % name)
        else:
            insert_statement = "INSERT INTO pgml.collections (name) VALUES ('%s')" % (
                name
            )
            run_create_or_insert_statement(conn, insert_statement)
            # Create collection object
        self.pool.putconn(conn)
        return Collection(self.pool, name)

    def delete_collection(self, name: str) -> None:
        conn = self.pool.getconn()
        results = run_select_statement(
            conn, "SELECT nspname FROM pg_catalog.pg_namespace;"
        )
        names = [res["nspname"] for res in results]
        if name in names:
            drop_statement = "DROP SCHEMA IF EXISTS %s CASCADE" % name
            run_drop_or_delete_statement(conn, drop_statement)

        self.pool.putconn(conn)
