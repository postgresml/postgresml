import psycopg
from psycopg import sql
from psycopg_pool import ConnectionPool
from psycopg import Connection

import logging
from rich.logging import RichHandler
from rich.progress import track

from typing import List, Dict, Optional, Any
import hashlib
import json
import uuid

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
        self._create_transforms_table()
        self.register_text_splitter()
        self.register_model()

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
                            task        text NOT NULL, \
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
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS task_index ON %s (task);"
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

    def _create_transforms_table(self) -> None:
        conn = self.pool.getconn()
        self.transforms_table = self.name + ".transforms"
        create_statement = (
            "CREATE TABLE IF NOT EXISTS %s (\
                            oid         regclass PRIMARY KEY,\
                            created_at  timestamptz NOT NULL DEFAULT now(), \
                            task        text NOT NULL, \
                            splitter    int8 NOT NULL REFERENCES %s\
                              ON DELETE CASCADE\
                              ON UPDATE CASCADE\
                              DEFERRABLE INITIALLY DEFERRED,\
                            model       int8 NOT NULL REFERENCES %s\
                              ON DELETE CASCADE\
                              ON UPDATE CASCADE\
                              DEFERRABLE INITIALLY DEFERRED,\
                            UNIQUE (task, splitter, model)\
                    );"
            % (self.transforms_table, self.splitters_table, self.models_table)
        )
        run_create_or_insert_statement(conn, create_statement)

        index_statement = (
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS created_at_index ON %s (created_at);"
            % (self.transforms_table)
        )
        run_create_or_insert_statement(conn, index_statement, autocommit=True)

        index_statement = (
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS task_index ON %s (task);"
            % (self.transforms_table)
        )
        run_create_or_insert_statement(conn, index_statement, autocommit=True)

        index_statement = (
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS splitter_index ON %s (splitter);"
            % (self.transforms_table)
        )
        run_create_or_insert_statement(conn, index_statement, autocommit=True)

        index_statement = (
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS model_index ON %s (model);"
            % (self.transforms_table)
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

        return results[0]["id"]

    def get_text_splitters(self) -> Dict[str, Any]:
        conn = self.pool.getconn()
        select_statement = "SELECT id, name, parameters FROM %s" % (
            self.splitters_table
        )
        splitters = run_select_statement(conn, select_statement)
        self.pool.putconn(conn)
        return splitters

    def generate_chunks(self, splitter_id: int = 1) -> None:
        conn = self.pool.getconn()
        log.info("Using splitter id %d" % splitter_id)
        select_statement = "SELECT name, parameters FROM %s WHERE id = %d" % (
            self.splitters_table,
            splitter_id,
        )
        results = run_select_statement(conn, select_statement)
        splitter_name = results[0]["name"]
        splitter_params = results[0]["parameters"]
        if splitter_name == "RecursiveCharacterTextSplitter":
            text_splitter = RecursiveCharacterTextSplitter(**splitter_params)
        else:
            raise ValueError("%s is not supported" % splitter_name)

        # Get all documents
        # todo: get documents that are not chunked
        select_statement = "SELECT id, text FROM %s" % self.documents_table
        results = run_select_statement(conn, select_statement)
        for result in results:
            log.debug("Came into chunk insertion")
            document = result["id"]
            text = result["text"]
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

    def register_model(
        self,
        task: Optional[str] = "embedding",
        model_name: Optional[str] = "intfloat/e5-small",
        model_params: Optional[Dict[str, Any]] = {},
    ) -> None:
        conn = self.pool.getconn()
        select_statement = (
            "SELECT * FROM %s WHERE name = %s AND parameters = %s AND task = %s"
            % (
                self.models_table,
                sql.Literal(model_name).as_string(conn),
                sql.Literal(json.dumps(model_params)).as_string(conn),
                sql.Literal(task).as_string(conn),
            )
        )
        results = run_select_statement(conn, select_statement)
        if len(results) > 0:
            log.info(
                "Model %s for %s task with parameters %s already exists in %s"
                % (model_name, task, model_params, self.models_table)
            )
        else:
            insert_statement = (
                "INSERT INTO %s (task, name, parameters) VALUES (%s, %s, %s)"
                % (
                    self.models_table,
                    sql.Literal(task).as_string(conn),
                    sql.Literal(model_name).as_string(conn),
                    sql.Literal(json.dumps(model_params)).as_string(conn),
                )
            )
            run_create_or_insert_statement(conn, insert_statement)
            results = run_select_statement(conn, select_statement)
        self.pool.putconn(conn)

        return results[0]["id"]

    def get_models(self) -> Dict[str, Any]:
        conn = self.pool.getconn()
        select_statement = "SELECT id, task, name, parameters FROM %s" % (
            self.models_table
        )
        models = run_select_statement(conn, select_statement)
        self.pool.putconn(conn)
        return models

    def _get_embeddings(
        self,
        conn: Connection,
        text: str = "hello world",
        model_name: str = "",
        parameters: Dict[str, Any] = {},
    ) -> int:
        if parameters:
            embed_statement = (
                "SELECT pgml.embed(transformer => %s, text => '%s', kwargs => %s) as embedding;"
                % (
                    sql.Literal(model_name).as_string(conn),
                    sql.Literal(json.dumps(parameters)).as_string(conn),
                    text,
                )
            )
        else:
            embed_statement = (
                "SELECT pgml.embed(transformer => %s, text => '%s') as embedding;"
                % (
                    sql.Literal(model_name).as_string(conn),
                    text,
                )
            )
        vector = run_select_statement(conn, embed_statement)[0]["embedding"]
        return vector

    def _create_or_get_embeddings_table(
        self,
        conn: Connection,
        model_id: Optional[int] = 1,
        splitter_id: Optional[int] = 1,
    ) -> str:
        select_statement = (
            "SELECT oid FROM %s WHERE task='embedding' AND model = %d AND splitter = %d"
            % (self.transforms_table, model_id, splitter_id)
        )
        results = run_select_statement(conn, select_statement)
        select_statement = "SELECT name, parameters FROM %s WHERE id = %d" % (
            self.models_table,
            model_id,
        )
        model_results = run_select_statement(conn, select_statement)
        model_name = model_results[0]["name"]
        model_parameters = model_results[0]["parameters"]

        if len(results) > 0:
            table_name = results[0]["oid"]
        else:
            table_name = self.name + ".embeddings_" + str(uuid.uuid4())[:6]
            embeddings_size = len(
                self._get_embeddings(
                    conn=conn, model_name=model_name, parameters=model_parameters
                )
            )
            create_statement = (
                "CREATE TABLE IF NOT EXISTS %s ( \
                                id          serial8 PRIMARY KEY,\
                                created_at  timestamptz NOT NULL DEFAULT now(),\
                                chunk       int8 NOT NULL REFERENCES %s\
                                          ON DELETE CASCADE\
                                          ON UPDATE CASCADE\
                                          DEFERRABLE INITIALLY DEFERRED,\
                                embedding   vector(%d) NOT NULL \
                                );"
                % (table_name, self.chunks_table, embeddings_size)
            )
            run_create_or_insert_statement(conn, create_statement)

            # Insert this name in transforms table
            oid_select_statement = "SELECT '%s'::regclass::oid;" % table_name
            results = run_select_statement(conn, oid_select_statement)
            oid = results[0]["oid"]

            insert_statement = (
                "INSERT INTO %s (oid, task, model, splitter) VALUES (%d, 'embedding', %d, %d)"
                % (self.transforms_table, oid, model_id, splitter_id)
            )
            run_create_or_insert_statement(conn, insert_statement)

            index_statement = (
                "CREATE INDEX CONCURRENTLY IF NOT EXISTS created_at_index ON %s (created_at);"
                % table_name
            )
            run_create_or_insert_statement(conn, index_statement, autocommit=True)

            index_statement = (
                "CREATE INDEX CONCURRENTLY IF NOT EXISTS chunk_index ON %s (chunk);"
                % table_name
            )
            run_create_or_insert_statement(conn, index_statement, autocommit=True)

            index_statement = (
                "CREATE INDEX CONCURRENTLY IF NOT EXISTS vector_index ON %s USING ivfflat (embedding vector_cosine_ops);"
                % table_name
            )

        return table_name

    def generate_embeddings(
        self, model_id: Optional[int] = 1, splitter_id: Optional[int] = 1
    ) -> None:
        conn = self.pool.getconn()
        embeddings_table = self._create_or_get_embeddings_table(
            conn, model_id=model_id, splitter_id=splitter_id
        )
        select_statement = "SELECT name, parameters FROM %s WHERE id = %d;" % (
            self.models_table,
            model_id,
        )
        results = run_select_statement(conn, select_statement)

        model = results[0]["name"]
        model_params = results[0]["parameters"]
        # get all chunks
        # todo: get all chunks that don't have embeddings
        if model_params:
            embeddings_statement = (
                "SELECT id, chunk, pgml.embed(text => chunk, transformer => %s, kwargs => %s) FROM %s WHERE splitter = %d;"
                % (
                    sql.Literal(model).as_string(conn),
                    sql.Literal(json.dumps(model_params)).as_string(conn),
                    self.chunks_table,
                    splitter_id,
                )
            )
        else:
            embeddings_statement = (
                "SELECT id, chunk, pgml.embed(text => chunk, transformer => %s) FROM %s WHERE splitter = %d;"
                % (
                    sql.Literal(model).as_string(conn),
                    self.chunks_table,
                    splitter_id,
                )
            )
        results = run_select_statement(conn, embeddings_statement)
        for result in results:
            insert_statement = "INSERT INTO %s (chunk, embedding) VALUES (%d, %s);" % (
                embeddings_table,
                result["id"],
                sql.Literal(result["embed"]).as_string(conn),
            )
            run_create_or_insert_statement(conn, insert_statement)
        self.pool.putconn(conn)

    def vector_search(
        self,
        query: str,
        query_parameters: Optional[Dict[str, Any]] = {},
        top_k: int = 5,
        model_id: int = 1,
        splitter_id: int = 1,
    ) -> Dict[str, Any]:
        conn = self.pool.getconn()
        select_statement = "SELECT name FROM %s WHERE id = %d;" % (
            self.models_table,
            model_id,
        )
        results = run_select_statement(conn, select_statement)

        model = results[0]["name"]

        query_embeddings = self._get_embeddings(
            conn, query, model_name=model, parameters=query_parameters
        )

        embeddings_table = self._create_or_get_embeddings_table(
            conn, model_id=model_id, splitter_id=splitter_id
        )

        select_statement = (
            "SELECT chunk, 1 - (%s.embedding <=> %s::float8[]::vector) AS score FROM %s ORDER BY score DESC LIMIT %d;"
            % (
                embeddings_table,
                sql.Literal(query_embeddings).as_string(conn),
                embeddings_table,
                top_k,
            )
        )
        results = run_select_statement(conn, select_statement)
        search_results = []

        for result in results:
            _out = {}
            _out["score"] = result["score"]
            select_statement = "SELECT chunk, document FROM %s WHERE id = %d" % (
                self.chunks_table,
                result["chunk"],
            )
            chunk_result = run_select_statement(conn, select_statement)
            _out["text"] = chunk_result[0]["chunk"]
            select_statement = "SELECT text, metadata FROM %s WHERE id = %d" % (
                self.documents_table,
                chunk_result[0]["document"],
            )
            document_result = run_select_statement(conn, select_statement)
            _out["metadata"] = document_result[0]["metadata"]
            search_results.append(_out)
        self.pool.putconn(conn)

        return search_results
