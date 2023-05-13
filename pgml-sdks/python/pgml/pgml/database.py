import psycopg
from psycopg import sql
from psycopg_pool import ConnectionPool

import logging
from rich.logging import RichHandler
from rich.progress import track


from .collection import Collection


FORMAT = "%(message)s"
logging.basicConfig(
    level="NOTSET", format=FORMAT, datefmt="[%X]", handlers=[RichHandler()]
)

log = logging.getLogger("rich")

class Database:
    def __init__(self, conninfo: str) -> None:
        """Initialize Database object
        Creates pgml.collections table if it doesn't exist

        Args:
            conninfo (str) : Postgres connection info postgresql://username:password@host:port/db

        Returns:
            None
        """
        self.conninfo = conninfo
        self.pool = ConnectionPool(conninfo)
        conn = self.pool.getconn()
        log.info("Creating table pgml.collections")
        create_statement = "CREATE TABLE IF NOT EXISTS pgml.collections \
                            (id  serial8 PRIMARY KEY,\
                            created_at  timestamptz NOT NULL DEFAULT now(),\
                            name  text NOT NULL)"
        conn.execute(create_statement)
        conn.commit()
        self.pool.putconn(conn)


    def create_collection(self, name: str) -> None:
        conn = self.pool.getconn()
        cur = conn.cursor()
        cur.execute("SELECT name FROM pgml.collections")
        name = name.lower()
        names = [res[0] for res in cur.fetchall()]
        if name in names:
            log.info("%s already exists.."%name)
        else:
            insert_statement = "INSERT INTO pgml.collections (name) VALUES ('%s')"%(name)
            cur.execute(insert_statement)

        conn.commit()
        cur.close()
        self.pool.putconn(conn)

        # Create collection object
        Collection(self.pool, name)
        