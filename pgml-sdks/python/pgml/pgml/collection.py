import psycopg
from psycopg import sql
from psycopg_pool import ConnectionPool

import logging
from rich.logging import RichHandler
from rich.progress import track

from typing import List, Dict, Optional, Any
import hashlib
import json

from .dbutils import *

from langchain.text_splitter import RecursiveCharacterTextSplitter

FORMAT = "%(message)s"
logging.basicConfig(
    level=os.environ.get("LOGLEVEL", "INFO"),
    format=FORMAT,
    datefmt="[%X]",
    handlers=[RichHandler()],
)
log = logging.getLogger("rich")


class Collection:
    def __init__(self, pool: ConnectionPool, name: str) -> None:
        self.pool = pool
        self.name = name
        self._create_documents_table()
        self._create_splitter_table()
        self._create_models_table()
        self._create_chunks_table()
        self.register_text_splitter()
        self.register_embeddings_model()

    
    def _create_documents_table(self) -> None:
        self.documents_table = self.name + ".documents"
        conn = self.pool.getconn()

        create_schema_statement = "CREATE SCHEMA IF NOT EXISTS %s" % self.name
        run_create_or_insert_statement(conn, create_schema_statement)
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
        run_create_or_insert_statement(conn, create_table_statement)

        create_index_statement = (
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS \
                                    created_at_index ON %s \
                            (created_at);"
            % (self.documents_table)
        )
        run_create_or_insert_statement(conn, create_index_statement, autocommit=True)

        create_index_statement = (
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS \
                document_index ON %s (document);"
            % (self.documents_table)
        )
        run_create_or_insert_statement(conn, create_index_statement, autocommit=True)

        create_table_statement = (
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS \
            metadata_index ON %s USING GIN (metadata jsonb_path_ops);"
            % (self.documents_table)
        )
        run_create_or_insert_statement(conn, create_index_statement, autocommit=True)

        self.pool.putconn(conn)

    def _create_splitter_table(self) -> None:
        conn = self.pool.getconn()
        self.splitters_table = self.name + ".splitters"
        create_statement = (
            "CREATE TABLE IF NOT EXISTS %s (\
                            id          serial8 PRIMARY KEY, \
                            created_at  timestamptz NOT NULL DEFAULT now(), \
                            name        text NOT NULL, \
                            parameters  jsonb NOT NULL DEFAULT '{}'\
                    );"
            % (self.splitters_table)
        )
        run_create_or_insert_statement(conn, create_statement)
        index_statement = (
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS created_at_index ON %s (created_at);"
            % (self.splitters_table)
        )
        run_create_or_insert_statement(conn, index_statement, autocommit=True)

        index_statement = (
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS name_index ON %s (name);"
            % (self.splitters_table)
        )
        run_create_or_insert_statement(conn, index_statement, autocommit=True)

        index_statement = (
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS parameters_index ON %s USING GIN (parameters jsonb_path_ops);"
            % (self.splitters_table)
        )
        run_create_or_insert_statement(conn, index_statement, autocommit=True)

        self.pool.putconn(conn)

    def _create_models_table(self) -> None:
        conn = self.pool.getconn()
        self.models_table = self.name + ".models"
        create_statement = (
            "CREATE TABLE IF NOT EXISTS %s (\
                            id          serial8 PRIMARY KEY, \
                            created_at  timestamptz NOT NULL DEFAULT now(), \
                            name        text NOT NULL, \
                            parameters  jsonb NOT NULL DEFAULT '{}'\
                    );"
            % (self.models_table)
        )
        run_create_or_insert_statement(conn, create_statement)
        index_statement = (
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS created_at_index ON %s (created_at);"
            % (self.models_table)
        )
        run_create_or_insert_statement(conn, index_statement, autocommit=True)

        index_statement = (
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS name_index ON %s (name);"
            % (self.models_table)
        )
        run_create_or_insert_statement(conn, index_statement, autocommit=True)

        index_statement = (
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS parameters_index ON %s USING GIN (parameters jsonb_path_ops);"
            % (self.models_table)
        )
        run_create_or_insert_statement(conn, index_statement, autocommit=True)

        self.pool.putconn(conn)

    def _create_chunks_table(self):
        conn = self.pool.getconn()
        self.chunks_table = self.name + ".chunks"

        create_statement = (
            "CREATE TABLE IF NOT EXISTS %s ( \
                            id          serial8 PRIMARY KEY,\
                            created_at  timestamptz NOT NULL DEFAULT now(),\
                            document    int8 NOT NULL REFERENCES %s\
                              ON DELETE CASCADE\
                              ON UPDATE CASCADE\
                              DEFERRABLE INITIALLY DEFERRED,\
                            splitter    int8 NOT NULL REFERENCES %s\
                              ON DELETE CASCADE\
                              ON UPDATE CASCADE\
                              DEFERRABLE INITIALLY DEFERRED,\
                            chunk_id    int8 NOT NULL,\
                            chunk     text NOT NULL);"
            % (self.chunks_table, self.documents_table, self.splitters_table)
        )
        run_create_or_insert_statement(conn, create_statement)

        index_statement = (
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS created_at_index ON %s (created_at);"
            % self.chunks_table
        )
        run_create_or_insert_statement(conn, index_statement, autocommit=True)

        index_statement = (
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS document_index ON %s (document);"
            % self.chunks_table
        )
        run_create_or_insert_statement(conn, index_statement, autocommit=True)

        index_statement = (
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS splitter_index ON %s (splitter);"
            % self.chunks_table
        )
        run_create_or_insert_statement(conn, index_statement, autocommit=True)

        self.pool.putconn(conn)

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

            run_drop_or_delete_statement(conn, delete_statement)
            insert_statement = (
                "INSERT INTO %s (text, document, metadata) VALUES (%s, %s, %s)"
                % (
                    self.documents_table,
                    sql.Literal(text).as_string(conn),
                    sql.Literal(document_id).as_string(conn),
                    sql.Literal(json.dumps(metadata)).as_string(conn),
                )
            )
            run_create_or_insert_statement(conn, insert_statement, verbose)

        self.pool.putconn(conn)

    def register_text_splitter(
        self,
        splitter_name: Optional[str] = "RecursiveCharacterTextSplitter",
        splitter_params: Optional[Dict[str, Any]] = {},
    ) -> None:
        conn = self.pool.getconn()
        select_statement = "SELECT * FROM %s WHERE name = %s AND parameters = %s" % (
            self.splitters_table,
            sql.Literal(splitter_name).as_string(conn),
            sql.Literal(json.dumps(splitter_params)).as_string(conn),
        )
        results = run_select_statement(conn, select_statement)
        if len(results) > 0:
            log.info(
                "Splitter %s with parameters %s already exists in %s"
                % (splitter_name, splitter_params, self.splitters_table)
            )
        else:
            insert_statement = "INSERT INTO %s (name, parameters) VALUES (%s, %s)" % (
                self.splitters_table,
                sql.Literal(splitter_name).as_string(conn),
                sql.Literal(json.dumps(splitter_params)).as_string(conn),
            )
            run_create_or_insert_statement(conn, insert_statement)
            results = run_select_statement(conn, select_statement)
        self.pool.putconn(conn)

        return results[0][0]

    def get_text_splitters(self) -> Dict[str,Any]:
        conn = self.pool.getconn()
        select_statement = "SELECT id, name, parameters FROM %s"%(self.splitters_table)
        results = run_select_statement(conn,select_statement)
        splitters = []
        for result in results:
            splitters.append({"id": result[0], "name": result[1], "parameters": result[2]})

        self.pool.putconn(conn)
        return splitters

    def get_text_chunks(
        self,
        splitter_id: int = 1
    ) -> None:
        conn = self.pool.getconn()
        log.info("Using splitter id %d" % splitter_id)
        select_statement = "SELECT name, parameters FROM %s WHERE id = %d"%(self.splitters_table,splitter_id)
        results = run_select_statement(conn,select_statement)
        splitter_name = results[0][0]
        splitter_params = results[0][1]
        if splitter_name == "RecursiveCharacterTextSplitter":
            text_splitter = RecursiveCharacterTextSplitter(**splitter_params)
        else:
            raise ValueError("%s is not supported" % splitter_name)
        # Get all documents
        select_statement = "SELECT id, text FROM %s" % self.documents_table
        results = run_select_statement(conn, select_statement)
        for result in results:
            log.debug("Came into chunk insertion")
            document = result[0]
            text = result[1]
            chunks = text_splitter.create_documents([text])
            for chunk_id, chunk in enumerate(chunks):
                insert_statement = (
                    "INSERT INTO %s (document,splitter,chunk_id, chunk) VALUES (%s, %s, %s, %s);"
                    % (
                        self.chunks_table,
                        sql.Literal(document).as_string(conn),
                        sql.Literal(splitter_id).as_string(conn),
                        sql.Literal(chunk_id).as_string(conn),
                        sql.Literal(chunk.page_content).as_string(conn),
                    )
                )
                run_create_or_insert_statement(conn, insert_statement)
        self.pool.putconn(conn)

    def register_embeddings_model(
        self,
        model_name: Optional[str] = "hkunlp/instructor-base",
        model_params: Optional[Dict[str, Any]] = {},
    ) -> None:
        conn = self.pool.getconn()
        select_statement = "SELECT * FROM %s WHERE name = %s AND parameters = %s" % (
            self.models_table,
            sql.Literal(model_name).as_string(conn),
            sql.Literal(json.dumps(model_params)).as_string(conn),
        )
        results = run_select_statement(conn, select_statement)
        if len(results) > 0:
            log.info(
                "Splitter %s with parameters %s already exists in %s"
                % (model_name, model_params, self.models_table)
            )
        else:
            insert_statement = "INSERT INTO %s (name, parameters) VALUES (%s, %s)" % (
                self.models_table,
                sql.Literal(model_name).as_string(conn),
                sql.Literal(json.dumps(model_params)).as_string(conn),
            )
            run_create_or_insert_statement(conn, insert_statement)
            results = run_select_statement(conn, select_statement)
        self.pool.putconn(conn)

        return results[0][0]
    
    def get_embeddings_model(self) -> Dict[str,Any]:
        conn = self.pool.getconn()
        select_statement = "SELECT id, name, parameters FROM %s"%(self.models_table)
        results = run_select_statement(conn,select_statement)
        splitters = []
        for result in results:
            splitters.append({"id": result[0], "name": result[1], "parameters": result[2]})

        self.pool.putconn(conn)
        return splitters