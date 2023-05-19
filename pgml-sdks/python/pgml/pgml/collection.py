import psycopg
from psycopg import sql
from psycopg_pool import ConnectionPool
from psycopg import Connection

import logging
from rich.logging import RichHandler
from rich.progress import track
from rich import print as rprint


from typing import List, Dict, Optional, Any
import hashlib
import json
import uuid
import os
from .dbutils import (
    run_drop_or_delete_statement,
    run_create_or_insert_statement,
    run_select_statement,
)

from langchain.text_splitter import RecursiveCharacterTextSplitter

FORMAT = "%(message)s"
logging.basicConfig(
    level=os.environ.get("LOGLEVEL", "ERROR"),
    format=FORMAT,
    datefmt="[%X]",
    handlers=[RichHandler()],
)
log = logging.getLogger("rich")

"""
Collection class to store tables for documents, chunks, models, splitters, and embeddings
"""


class Collection:
    def __init__(self, pool: ConnectionPool, name: str) -> None:
        """
        The function initializes an object with a connection pool and a name, and creates several tables
        while registering a text splitter and a model.

        :param pool: `pool` is an instance of `ConnectionPool` class which manages a pool of database
        connections
        :type pool: ConnectionPool
        :param name: The `name` parameter is a string that represents the name of an object being
        initialized. It is used as an identifier for the object within the code
        :type name: str
        """
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
        """
        This function creates a table and indexes for storing documents in a PostgreSQL database.
        """
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
        """
        This function creates a table with specific columns and indexes in a PostgreSQL database.
        """
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
        """
        This function creates a table in a PostgreSQL database with specific columns and indexes.
        """
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
        """
        This function creates a transforms table in a PostgreSQL database and adds indexes to it.
        """
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
        """
        This function creates a table for storing document chunks and their metadata in a PostgreSQL
        database.
        """
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
        """
        The function `upsert_documents` inserts or updates documents in a database table based on their ID,
        text, and metadata.

        :param documents: A list of dictionaries, where each dictionary represents a document to be upserted
        into a database table. Each dictionary should contain metadata about the document, as well as the
        actual text of the document
        :type documents: List[Dict[str, str]]
        :param text_key: The key in the dictionary that corresponds to the text of the document, defaults to
        text
        :type text_key: Optional[str] (optional)
        :param id_key: The `id_key` parameter is an optional string parameter that specifies the key in the
        dictionary of each document that contains the unique identifier for that document. If this key is
        present in the dictionary, its value will be used as the document ID. If it is not present, a hash
        of the document, defaults to id
        :type id_key: Optional[str] (optional)
        :param verbose: A boolean parameter that determines whether or not to print verbose output during
        the upsert process. If set to True, additional information will be printed to the console during the
        upsert process. If set to False, only essential information will be printed, defaults to False
        :type verbose: bool (optional)
        """
        conn = self.pool.getconn()
        for document in track(documents, description="Upserting documents"):
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
        """
        This function registers a text splitter with a given name and parameters in a database table if it
        does not already exist.

        :param splitter_name: The name of the text splitter being registered. It is an optional parameter
        and defaults to "RecursiveCharacterTextSplitter" if not provided, defaults to
        RecursiveCharacterTextSplitter
        :type splitter_name: Optional[str] (optional)
        :param splitter_params: splitter_params is a dictionary that contains parameters for a text
        splitter. These parameters can be used to customize the behavior of the text splitter. The function
        takes this dictionary as an optional argument and if it is not provided, an empty dictionary is used
        as the default value
        :type splitter_params: Optional[Dict[str, Any]]
        :return: the id of the splitter that was either found in the database or inserted into the database.
        """
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

    def get_text_splitters(self) -> List[Dict[str, Any]]:
        """
        This function retrieves a list of dictionaries containing information about text splitters from a
        database.
        :return: The function `get_text_splitters` is returning a list of dictionaries, where each
        dictionary contains the `id`, `name`, and `parameters` of a text splitter.
        """
        conn = self.pool.getconn()
        select_statement = "SELECT id, name, parameters FROM %s" % (
            self.splitters_table
        )
        splitters = run_select_statement(conn, select_statement)
        self.pool.putconn(conn)
        return splitters

    def generate_chunks(self, splitter_id: int = 1) -> None:
        """
        This function generates chunks of text from unchunked documents using a specified text splitter.

        :param splitter_id: The ID of the splitter to use for generating chunks, defaults to 1
        :type splitter_id: int (optional)
        """
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

        select_statement = (
            "SELECT id, text FROM %s WHERE id NOT IN (SELECT document FROM %s WHERE splitter = %d);"
            % (self.documents_table, self.chunks_table, splitter_id)
        )

        results = run_select_statement(conn, select_statement)
        for result in track(results, description="Generating chunks"):
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
        """
        This function registers a model in a database if it does not already exist.

        :param task: The type of task the model is being registered for, with a default value of
        "embedding", defaults to embedding
        :type task: Optional[str] (optional)
        :param model_name: The name of the model being registered, defaults to intfloat/e5-small
        :type model_name: Optional[str] (optional)
        :param model_params: model_params is a dictionary that contains the parameters for the model being
        registered. These parameters can be used to configure the model for a specific task. The dictionary
        can be empty if no parameters are needed
        :type model_params: Optional[Dict[str, Any]]
        :return: the id of the registered model.
        """
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

    def get_models(self) -> List[Dict[str, Any]]:
        """
        The function retrieves a list of dictionaries containing information about models from a database
        table.
        :return: The function `get_models` is returning a list of dictionaries, where each dictionary
        represents a model and contains the following keys: "id", "task", "name", and "parameters". The
        values associated with these keys correspond to the respective fields in the database table
        specified by `self.models_table`.
        """
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
    ) -> List[float]:
        """
        This function retrieves embeddings for a given text using a specified model and returns the
        resulting vector.

        :param conn: The `conn` parameter is a `Connection` object that represents a connection to a
        PostgreSQL database. It is used to execute SQL statements and retrieve results from the database
        :type conn: Connection
        :param text: The text that needs to be embedded into a vector representation, defaults to hello
        world
        :type text: str (optional)
        :param model_name: The name of the transformer model that will be used to generate the embeddings
        for the given text
        :type model_name: str
        :param parameters: The `parameters` parameter is a dictionary that can be used to pass additional
        parameters to the function. It has a default value of an empty dictionary `{}`. These parameters can
        be used to customize the behavior of the function, such as specifying the batch size or the maximum
        sequence length. However, in
        :type parameters: Dict[str, Any]
        :return: a vector of embeddings for the given text using the specified model and parameters.
        """
        embed_statement = (
            "SELECT pgml.embed(transformer => %s, text => '%s', kwargs => %s) as embedding;"
            % (
                sql.Literal(model_name).as_string(conn),
                text,
                sql.Literal(json.dumps(parameters)).as_string(conn),
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
        """
        This function creates or retrieves a table for storing embeddings based on the given model and
        splitter IDs.

        :param conn: The database connection object used to interact with the database
        :type conn: Connection
        :param model_id: The ID of the model used for generating the embeddings, defaults to 1
        :type model_id: Optional[int] (optional)
        :param splitter_id: The `splitter_id` parameter is an optional integer that represents the ID of
        the splitter used for the embeddings table. It is used in the SQL query to select the embeddings
        table with a specific splitter ID. If no splitter ID is provided, the default value of 1 is
        used, defaults to 1
        :type splitter_id: Optional[int] (optional)
        :return: a string which is the name of the embeddings table created or retrieved from the
        database.
        """
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
        """
        This function generates embeddings for chunks of text using a specified model and inserts them into
        a database table.

        :param model_id: The ID of the model to use for generating embeddings, defaults to 1
        :type model_id: Optional[int] (optional)
        :param splitter_id: The `splitter_id` parameter is an optional integer that specifies the ID of the
        data splitter to use for generating embeddings. If not provided, it defaults to 1, defaults to 1
        :type splitter_id: Optional[int] (optional)
        """
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

        # get all chunks that don't have embeddings
        embeddings_statement = (
            "SELECT id, chunk, \
                pgml.embed(text => chunk, transformer => %s, kwargs => %s) FROM %s \
                WHERE splitter = %d AND id NOT IN (SELECT chunk FROM %s);"
            % (
                sql.Literal(model).as_string(conn),
                sql.Literal(json.dumps(model_params)).as_string(conn),
                self.chunks_table,
                splitter_id,
                embeddings_table,
            )
        )

        rprint("Generating embeddings using %s ... " % model)
        results = run_select_statement(conn, embeddings_statement)
        for result in track(results, description="Inserting embeddings"):
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
    ) -> List[Dict[str, Any]]:
        """
        This function performs a vector search on a database using a query and returns the top matching
        results.

        :param query: The search query string
        :type query: str
        :param query_parameters: Optional dictionary of additional parameters to be used in generating
        the query embeddings. These parameters are specific to the model being used and can be used to
        fine-tune the search results. If no parameters are provided, default values will be used
        :type query_parameters: Optional[Dict[str, Any]]
        :param top_k: The number of search results to return, sorted by relevance score, defaults to 5
        :type top_k: int (optional)
        :param model_id: The ID of the model to use for generating embeddings, defaults to 1
        :type model_id: int (optional)
        :param splitter_id: The `splitter_id` parameter is an integer that identifies the specific
        splitter used to split the documents into chunks. It is used to retrieve the embeddings table
        associated with the specified splitter, defaults to 1
        :type splitter_id: int (optional)
        :return: a list of dictionaries containing search results for a given query. Each dictionary
        contains the following keys: "score", "text", and "metadata". The "score" key contains a float
        value representing the similarity score between the query and the search result. The "text" key
        contains the text of the search result, and the "metadata" key contains any metadata associated
        with the search result
        """
        conn = self.pool.getconn()
        select_statement = "SELECT name FROM %s WHERE id = %d;" % (
            self.models_table,
            model_id,
        )
        results = run_select_statement(conn, select_statement)

        model = results[0]["name"]

        embeddings_table_statement = (
            "SELECT oid FROM %s WHERE model = %d AND splitter = %d"
            % (self.transforms_table, model_id, splitter_id)
        )

        embedding_table_results = run_select_statement(conn, embeddings_table_statement)
        if embedding_table_results:
            embeddings_table = embedding_table_results[0]["oid"]
        else:
            rprint(
                "Embeddings for model id %d and splitter id %d do not exist.\nPlease run collection.generate_embeddings(model_id = %d, splitter_id = %d)"
                % (model_id, splitter_id, model_id, splitter_id)
            )
            return []

        query_embeddings = self._get_embeddings(
            conn, query, model_name=model, parameters=query_parameters
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
