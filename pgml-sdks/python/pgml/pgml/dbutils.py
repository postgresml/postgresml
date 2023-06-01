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
    """
    The function runs a select statement on a database connection and returns the results as a list of
    dictionaries.

    :param conn: The `conn` parameter is a connection object that represents a connection to a database.
    It is used to execute SQL statements and retrieve results from the database
    :type conn: Connection
    :param statement: The SQL SELECT statement to be executed on the database
    :type statement: str
    :return: The function `run_select_statement` returns a list of dictionaries, where each dictionary
    represents a row of the result set of the SQL query specified in the `statement` parameter. The keys
    of each dictionary are the column names of the result set, and the values are the corresponding
    values of the row.
    """

    statement = statement.strip().rstrip(";")
    cur = conn.cursor()
    json_conversion_statement = """
            SELECT array_to_json(array_agg(row_to_json(t)))
            FROM ({select_statement}) t;
            """.format(
        select_statement=statement
    )
    log.info("Running %s .. " % json_conversion_statement)
    cur.execute(json_conversion_statement)
    results = cur.fetchall()
    conn.commit()
    cur.close()

    output = []
    if results:
        if results[0][0]:
            output = results[0][0]

    return output


def run_drop_or_delete_statement(conn: Connection, statement: str) -> None:
    """
    This function executes a given SQL statement to drop or delete data from a database using a provided
    connection object.

    :param conn: The parameter `conn` is of type `Connection`, which is likely a connection object to a
    database. It is used to execute SQL statements on the database
    :type conn: Connection
    :param statement: The SQL statement to be executed on the database connection object
    :type statement: str
    """
    log.info("Running %s .. " % statement)
    cur = conn.cursor()
    cur.execute(statement)
    conn.commit()
    cur.close()
