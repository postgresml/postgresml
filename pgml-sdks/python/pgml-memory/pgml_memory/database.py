import psycopg
from psycopg import sql
from psycopg_pool import ConnectionPool


class Database:
    def __init__(self, conninfo: str) -> None:
        """Initialize Database object
        Creates pgml.collections table if it doesn't exist

        Args:
            conninfo (str) : Postgres connection info postgresql://username:password@host:port/db

        Returns:
            None
        """
        self.pool = ConnectionPool(conninfo)
        conn = self.pool.getconn()

        create_statement = "CREATE TABLE IF NOT EXISTS pgml.collections (id  serial8 PRIMARY KEY,created_at  timestamptz NOT NULL DEFAULT now(),name  text NOT NULL)"
        conn.execute(create_statement)
        conn.commit()
        self.pool.putconn(conn)
