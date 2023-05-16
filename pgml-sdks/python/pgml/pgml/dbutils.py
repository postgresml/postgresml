from psycopg_pool import ConnectionPool
from psycopg import Connection
from typing import List, Any

import logging
from rich.logging import RichHandler
from rich.progress import track
import os

FORMAT = "%(message)s"
logging.basicConfig(
    level=os.environ.get("LOGLEVEL", "INFO"),
    format=FORMAT,
    datefmt="[%X]",
    handlers=[RichHandler()],
)
log = logging.getLogger("rich")


def run_create_or_insert_statement(
    conn: Connection, statement: str, autocommit: bool = False
) -> None:
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
    results = cur.fetchall()
    conn.commit()
    cur.close()
    return results


def run_drop_or_delete_statement(conn: Connection, statement: str) -> None:
    log.info("Running %s .. " % statement)
    cur = conn.cursor()
    cur.execute(statement)
    conn.commit()
    cur.close()
