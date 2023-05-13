import psycopg
from psycopg import sql
from psycopg_pool import ConnectionPool


class Collection:
    def __init__(self, pool: ConnectionPool, name: str) -> None:

        pass