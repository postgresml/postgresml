import psycopg
from psycopg import sql
from psycopg_pool import ConnectionPool

import logging
from rich.logging import RichHandler
from rich.progress import track

from typing import List, Dict, Optional
import hashlib
import json

from .dbutils import *

FORMAT = "%(message)s"
logging.basicConfig(
    level=os.environ.get("LOGLEVEL", "INFO"), format=FORMAT, datefmt="[%X]", handlers=[RichHandler()]
)
log = logging.getLogger("rich")


class Collection:
    def __init__(self, pool: ConnectionPool, name: str) -> None:
        self.pool = pool
        self.name = name
        self.documents_table = name + ".documents"

        create_schema_statement = "CREATE SCHEMA IF NOT EXISTS %s" % name
        run_create_or_insert_statement(self.pool, create_schema_statement)
        create_table_statement = (
            "CREATE TABLE IF NOT EXISTS %s (\
                                id          serial8 PRIMARY KEY,\
                                created_at  timestamptz NOT NULL DEFAULT now(),\
                                document    uuid NOT NULL,\
                                metadata    jsonb NOT NULL DEFAULT '{}',\
                                text        text NOT NULL,\
                                UNIQUE (document)\
                                );"
            % self.documents_table
        )
        run_create_or_insert_statement(self.pool, create_table_statement)

        create_index_statement_1 = (
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS \
                                    created_at_index ON %s \
                            (created_at);"
            % (self.documents_table)
        )
        run_create_or_insert_statement(
            self.pool, create_index_statement_1, autocommit=True
        )

        create_index_statement_2 = (
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS \
                document_index ON %s (document)"
            % (self.documents_table)
        )
        run_create_or_insert_statement(
            self.pool, create_index_statement_2, autocommit=True
        )

        create_index_statement_3 = (
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS \
            metadata_index ON %s USING GIN (metadata jsonb_path_ops);"
            % (self.documents_table)
        )
        run_create_or_insert_statement(
            self.pool, create_index_statement_3, autocommit=True
        )

    def upsert_documents(
        self,
        documents: List[Dict[str, str]],
        text_key: Optional[str] = "text",
        id_key: Optional[str] = "id",
        verbose: bool = False,
    ) -> None:
        conn = self.pool.getconn()
        for document in documents:
            if text_key in list(document.keys()):
                text = document.pop(text_key)
            else:
                logging.warning(
                    "%s in not present, skipping this document..." % text_key
                )
                continue
            if id_key in list(document.keys()):
                document_id = document.pop(id_key)
            else:
                log.info("id key is not present.. hashing")
                document_id = hashlib.md5(text.encode("utf-8")).hexdigest()
            metadata = document
            delete_statement = "DELETE FROM %s WHERE document = %s" % (
                self.documents_table,
                sql.Literal(document_id).as_string(conn),
            )

            run_drop_or_delete_statement(self.pool, delete_statement)
            insert_statement = (
                "INSERT INTO %s (text, document, metadata) VALUES (%s, %s, %s)"
                % (
                    self.documents_table,
                    sql.Literal(text).as_string(conn),
                    sql.Literal(document_id).as_string(conn),
                    sql.Literal(json.dumps(metadata)).as_string(conn),
                )
            )
            run_create_or_insert_statement(self.pool, insert_statement, verbose)

        self.pool.putconn(conn)
