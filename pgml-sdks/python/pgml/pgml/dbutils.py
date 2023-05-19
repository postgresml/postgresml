from psycopg_pool import ConnectionPool
from psycopg import Connection
from typing import List, Any

import logging
from rich.logging import RichHandler
from rich.progress import track
import os

FORMAT = "%(message)s"
logging.basicConfig(
    level=os.environ.get("LOGLEVEL", "ERROR"),
    format=FORMAT,
    datefmt="[%X]",
    handlers=[RichHandler()],
)
log = logging.getLogger("rich")


def run_create_or_insert_statement(
    conn: Connection, statement: str, autocommit: bool = False
) -> None:
    """
    This function executes a SQL statement on a database connection and optionally commits the changes.

    :param conn: The `conn` parameter is a connection object that represents a connection to a database.
    It is used to execute SQL statements and manage transactions
    :type conn: Connection

    :param statement: The SQL statement to be executed
    :type statement: str

    :param autocommit: A boolean parameter that determines whether the transaction should be
    automatically committed after executing the statement. If set to True, the transaction will be
    committed automatically. If set to False, the transaction will need to be manually committed using
    the conn.commit() method, defaults to False
    :type autocommit: bool (optional)

    """
    log.info("Running %s .. " % statement)
    if autocommit:
        conn.autocommit = autocommit
    cur = conn.cursor()
    try:
        cur.execute(statement)
    except Exception as e:
        print(e)

    if not autocommit:
        conn.commit()

    cur.close()


def run_select_statement(conn: Connection, statement: str) -> List[Any]:
    log.info("Running %s .. " % statement)
    cur = conn.cursor()
    cur.execute(statement)
    qresults = cur.fetchall()
    colnames = [desc[0] for desc in cur.description]
    results = []
    for result in qresults:
        _dict = {}
        for _id, col in enumerate(colnames):
            _dict[col] = result[_id]
        results.append(_dict)
    conn.commit()
    cur.close()

    return results


def run_drop_or_delete_statement(conn: Connection, statement: str) -> None:
    log.info("Running %s .. " % statement)
    cur = conn.cursor()
    cur.execute(statement)
    conn.commit()
    cur.close()
