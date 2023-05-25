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
        self._cache_embeddings_table_names = {}
        self._cache_model_names = {}

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
                                source_uuid uuid NOT NULL,\
                                metadata    jsonb NOT NULL DEFAULT '{}',\
                                text        text NOT NULL,\
                                UNIQUE (source_uuid)\
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
        # oid --> table_name
        create_statement = (
            "CREATE TABLE IF NOT EXISTS %s (\
                            table_name  text PRIMARY KEY,\
                            created_at  timestamptz NOT NULL DEFAULT now(), \
                            task        text NOT NULL, \
                            splitter_id    int8 NOT NULL REFERENCES %s\
                              ON DELETE CASCADE\
                              ON UPDATE CASCADE\
                              DEFERRABLE INITIALLY DEFERRED,\
                            model_id      int8 NOT NULL REFERENCES %s\
                              ON DELETE CASCADE\
                              ON UPDATE CASCADE\
                              DEFERRABLE INITIALLY DEFERRED,\
                            UNIQUE (task, splitter_id, model_id)\
                    );"
            % (self.transforms_table, self.splitters_table, self.models_table)
        )
        run_create_or_insert_statement(conn, create_statement)

        index_statement = (
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS created_at_index ON %s (created_at);"
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

        # document --> document_id, chunk_id --> chunk_index
        create_statement = (
            "CREATE TABLE IF NOT EXISTS %s ( \
                            id          serial8 PRIMARY KEY,\
                            created_at  timestamptz NOT NULL DEFAULT now(),\
                            document_id    int8 NOT NULL REFERENCES %s\
                              ON DELETE CASCADE\
                              ON UPDATE CASCADE\
                              DEFERRABLE INITIALLY DEFERRED,\
                            splitter_id    int8 NOT NULL REFERENCES %s\
                              ON DELETE CASCADE\
                              ON UPDATE CASCADE\
                              DEFERRABLE INITIALLY DEFERRED,\
                            chunk_index    int8 NOT NULL,\
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
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS document_id_index ON %s (document_id);"
            % self.chunks_table
        )
        run_create_or_insert_statement(conn, index_statement, autocommit=True)

        index_statement = (
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS splitter_id_index ON %s (splitter_id);"
            % self.chunks_table
        )
        run_create_or_insert_statement(conn, index_statement, autocommit=True)

        self.pool.putconn(conn)

    def upsert_documents(
        self,
        documents: List[Dict[str, Any]],
        text_key: Optional[str] = "text",
        id_key: Optional[str] = "id",
    ) -> None:
        """
        The function `upsert_documents` inserts or updates documents in a database table based on their ID,
        text, and metadata.

        :param documents: A list of dictionaries, where each dictionary represents a document to be upserted
        into a database table. Each dictionary should contain metadata about the document, as well as the
        actual text of the document
        :type documents: List[Dict[str, Any]]
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

            _uuid = ""
            if id_key not in list(document.keys()):
                log.info("id key is not present.. hashing")
                source_uuid = hashlib.md5(text.encode("utf-8")).hexdigest()
            else:
                _uuid = document.pop(id_key)
                try:
                    source_uuid = str(uuid.UUID(_uuid))
                except Exception:
                    source_uuid = hashlib.md5(text.encode("utf-8")).hexdigest()

            metadata = document

            upsert_statement = "INSERT INTO {documents_table} (text, source_uuid, metadata) VALUES ({text}, {source_uuid}, {metadata}) \
                ON CONFLICT (source_uuid) \
                DO UPDATE SET text = {text}, metadata = {metadata}".format(
                documents_table=self.documents_table,
                text=sql.Literal(text).as_string(conn),
                metadata=sql.Literal(json.dumps(metadata)).as_string(conn),
                source_uuid=sql.Literal(source_uuid).as_string(conn),
            )
            run_create_or_insert_statement(conn, upsert_statement)

            # put the text and id back in document
            document[text_key] = text
            if _uuid:
                document[id_key] = source_uuid

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
        if results:
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
            "SELECT id, text FROM %s WHERE id NOT IN (SELECT document_id FROM %s WHERE splitter_id = %d);"
            % (self.documents_table, self.chunks_table, splitter_id)
        )

        results = run_select_statement(conn, select_statement)
        for result in track(results, description="Generating chunks"):
            log.debug("Came into chunk insertion")
            document = result["id"]
            text = result["text"]
            chunks = text_splitter.create_documents([text])
            for chunk_index, chunk in enumerate(chunks):
                insert_statement = (
                    "INSERT INTO %s (document_id,splitter_id,chunk_index, chunk) VALUES (%s, %s, %s, %s);"
                    % (
                        self.chunks_table,
                        sql.Literal(document).as_string(conn),
                        sql.Literal(splitter_id).as_string(conn),
                        sql.Literal(chunk_index).as_string(conn),
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
        if results:
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
            "SELECT table_name FROM %s WHERE task='embedding' AND model_id = %d AND splitter_id = %d"
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

        if results:
            table_name = results[0]["table_name"]
        else:
            table_name = self.name + ".embeddings_" + str(uuid.uuid4())[:6]
            embeddings_size = len(
                self._get_embeddings(
                    conn=conn, model_name=model_name, parameters=model_parameters
                )
            )
            # chunk --> chunk_id
            create_statement = (
                "CREATE TABLE IF NOT EXISTS %s ( \
                                id          serial8 PRIMARY KEY,\
                                created_at  timestamptz NOT NULL DEFAULT now(),\
                                chunk_id    int8 NOT NULL REFERENCES %s\
                                          ON DELETE CASCADE\
                                          ON UPDATE CASCADE\
                                          DEFERRABLE INITIALLY DEFERRED,\
                                embedding   vector(%d) NOT NULL \
                                );"
                % (table_name, self.chunks_table, embeddings_size)
            )
            run_create_or_insert_statement(conn, create_statement)

            # Insert this name in transforms table
            insert_statement = (
                "INSERT INTO %s (table_name, task, model_id, splitter_id) VALUES (%s, 'embedding', %d, %d)"
                % (
                    self.transforms_table,
                    sql.Literal(table_name).as_string(conn),
                    model_id,
                    splitter_id,
                )
            )
            run_create_or_insert_statement(conn, insert_statement)

            index_statement = (
                "CREATE INDEX CONCURRENTLY IF NOT EXISTS created_at_index ON %s (created_at);"
                % table_name
            )
            run_create_or_insert_statement(conn, index_statement, autocommit=True)

            index_statement = (
                "CREATE INDEX CONCURRENTLY IF NOT EXISTS chunk_id_index ON %s (chunk_id);"
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
        insert_select_statement = """
            INSERT INTO {embeddings_table} (chunk_id, embedding)
            SELECT id, pgml.embed(text => chunk, transformer => {model}, kwargs => {model_params})
            FROM {chunks_table}
            WHERE splitter_id = {splitter_id}
                AND id NOT IN (SELECT chunk_id FROM {embeddings_table});
            """.format(
            embeddings_table=embeddings_table,
            model=sql.Literal(model).as_string(conn),
            model_params=sql.Literal(json.dumps(model_params)).as_string(conn),
            chunks_table=self.chunks_table,
            splitter_id=splitter_id,
        )

        rprint("Generating embeddings using %s ... " % model)
        run_create_or_insert_statement(conn, insert_select_statement)

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

        if model_id in self._cache_model_names.keys():
            model = self._cache_model_names[model_id]
        else:
            select_statement = "SELECT name FROM %s WHERE id = %d;" % (
                self.models_table,
                model_id,
            )
            results = run_select_statement(conn, select_statement)

            model = results[0]["name"]
            self._cache_model_names[model_id] = model

        embeddings_table = ""
        if model_id in self._cache_embeddings_table_names.keys():
            if splitter_id in self._cache_embeddings_table_names[model_id].keys():
                embeddings_table = self._cache_embeddings_table_names[model_id][
                    splitter_id
                ]

        if not embeddings_table:
            embeddings_table_statement = (
                "SELECT table_name FROM %s WHERE model_id = %d AND splitter_id = %d"
                % (self.transforms_table, model_id, splitter_id)
            )

            embedding_table_results = run_select_statement(
                conn, embeddings_table_statement
            )
            embeddings_table = embedding_table_results[0]["table_name"]
            self._cache_embeddings_table_names[model_id] = {
                splitter_id: embeddings_table
            }

        if not embeddings_table:
            rprint(
                "Embeddings for model id %d and splitter id %d do not exist.\nPlease run collection.generate_embeddings(model_id = %d, splitter_id = %d)"
                % (model_id, splitter_id, model_id, splitter_id)
            )
            return []

        cte_select_statement = """
        WITH query_cte AS (
            SELECT pgml.embed(transformer => {model}, text => '{query_text}', kwargs => {model_params}) AS query_embedding
        ),
        cte AS (
            SELECT chunk_id, 1 - ({embeddings_table}.embedding <=> query_cte.query_embedding::float8[]::vector) AS score
            FROM {embeddings_table}
            CROSS JOIN query_cte
            ORDER BY score DESC
            LIMIT {top_k}
        )
        SELECT cte.score, chunks.chunk, documents.metadata
        FROM cte
        INNER JOIN {chunks_table} chunks ON chunks.id = cte.chunk_id
        INNER JOIN {documents_table} documents ON documents.id = chunks.document_id;
        """.format(
            model=sql.Literal(model).as_string(conn),
            query_text=query,
            model_params=sql.Literal(json.dumps(query_parameters)).as_string(conn),
            embeddings_table=embeddings_table,
            top_k=top_k,
            chunks_table=self.chunks_table,
            documents_table=self.documents_table,
        )

        search_results = run_select_statement(conn, cte_select_statement)
        self.pool.putconn(conn)

        return search_results
