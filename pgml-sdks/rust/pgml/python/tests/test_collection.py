import asyncio
import unittest
import pgml
import os
import hashlib
from pypika import PostgreSQLQuery as Query, Table, Parameter
from psycopg_pool import ConnectionPool
from psycopg import Connection

from typing import List, Any

import logging
from rich.logging import RichHandler
from rich.progress import track
import os
import pytest

FORMAT = "%(message)s"
logging.basicConfig(
    level=os.environ.get("LOGLEVEL", "ERROR"),
    format=FORMAT,
    datefmt="[%X]",
    handlers=[RichHandler()],
)
log = logging.getLogger("rich")

async def run_select_statement(
    conn: Connection, statement: str, order_by: str = "", ascending: bool = True
) -> List[Any]:
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
    order_statement = ""
    if order_by:
        order_statement = "ORDER BY t.%s" % order_by
        if ascending:
            order_statement += " ASC"
        else:
            order_statement += " DESC"

    if order_statement:
        json_conversion_statement = """
            SELECT array_to_json(array_agg(row_to_json(t) {order_statement}))
            FROM ({select_statement}) t;
            """.format(
            select_statement=statement,
            order_statement=order_statement,
        )
    else:
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

class TestCollection(unittest.IsolatedAsyncioTestCase):

    async def asyncSetUp(self) -> None:
        local_pgml = "postgres://postgres@127.0.0.1:5433/pgml_development"
        conninfo = os.environ.get("PGML_CONNECTION", local_pgml)
        self.pool = ConnectionPool(conninfo)
        self.db = pgml.Database(conninfo)
        self.collection_name = "test_collection_1"
        self.collection = await self.db.create_or_get_collection(self.collection_name)
        print(self.collection)
        self.documents = [
            {
                "id": hashlib.md5(f"abcded-{i}".encode("utf-8")).hexdigest(),
                "text": f"Lorem ipsum {i}",
                "source": "test_suite",
            }
            for i in range(4, 7)
        ]
        self.documents_no_ids = [
            {
                "text": f"Lorem ipsum {i}",
                "source": "test_suite_no_ids",
            }
            for i in range(1, 4)
        ]

        self.documents_with_metadata = [
            {
                "text": f"Lorem ipsum metadata",
                "source": f"url {i}",
                "url": f"/home {i}",
                "user": f"John Doe-{i+1}",
            }
            for i in range(8, 12)
        ]

        self.documents_with_reviews = [
            {
                "text": f"product is abc {i}",
                "reviews": i * 2,
            }
            for i in range(20, 25)
        ]

        self.documents_with_reviews_metadata = [
            {
                "text": f"product is abc {i}",
                "reviews": i * 2,
                "source": "amazon",
                "user": "John Doe",
            }
            for i in range(20, 25)
        ]

        self.documents_with_reviews_metadata += [
            {
                "text": f"product is abc {i}",
                "reviews": i * 2,
                "source": "ebay",
            }
            for i in range(20, 25)
        ]    

    async def test_documents_upsert(self):
        await self.collection.upsert_documents(self.documents)
        conn = self.pool.getconn()
        table = Table("documents",schema=self.collection_name)
        query = Query.from_(table).select("*")
        results = await run_select_statement(conn, str(query))
        self.pool.putconn(conn)
        assert len(results) >= len(self.documents)

    async def test_documents_upsert_no_ids(self):
        await self.collection.upsert_documents(self.documents_no_ids)
        conn = self.pool.getconn()
        table = Table("documents",schema=self.collection_name)
        query = Query.from_(table).select("*")
        results = await run_select_statement(conn, str(query))
        self.pool.putconn(conn)
        assert len(results) >= len(self.documents_no_ids)

    async def test_default_text_splitter(self):
        await self.collection.register_text_splitter()
        splitters = await self.collection.get_text_splitters()
        print(splitters)
        assert splitters[0]["name"] == "recursive_character"

    async def test_default_embeddings_model(self):
        await self.collection.register_model()
        models = await self.collection.get_models()

        assert len(models) == 1
        assert models[0]["name"] == "intfloat/e5-small"

    async def test_generate_chunks(self):
        await self.collection.upsert_documents(self.documents)
        await self.collection.upsert_documents(self.documents_no_ids)
        await self.collection.register_text_splitter()
        await self.collection.generate_chunks(splitter_id=1)
        splitter_params = {"chunk_size": "3", "chunk_overlap": "2"}
        await self.collection.register_text_splitter( splitter_name="recursive_character",
            splitter_params=splitter_params
        )
        await self.collection.generate_chunks(splitter_id=1)

    async def test_generate_embeddings(self):
        await self.collection.upsert_documents(self.documents)
        await self.collection.upsert_documents(self.documents_no_ids)
        self.collection.generate_chunks(splitter_id=1)
        self.collection.generate_embeddings()

    async def test_vector_search(self):
        await self.collection.upsert_documents(self.documents)
        await self.collection.upsert_documents(self.documents_no_ids)
        await self.collection.generate_chunks()
        await self.collection.generate_embeddings()
        results = await self.collection.vector_search("Lorem ipsum 1", top_k=2)
        assert abs(results[0][0] - 1.0) < 1e-5