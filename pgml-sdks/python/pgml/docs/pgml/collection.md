Module pgml.collection
======================

Variables
---------

    
`log`
:   Collection class to store tables for documents, chunks, models, splitters, and embeddings

Classes
-------

`Collection(pool: psycopg_pool.pool.ConnectionPool, name: str)`
:   The function initializes an object with a connection pool and a name, and creates several tables
    while registering a text splitter and a model.
    
    :param pool: `pool` is an instance of `ConnectionPool` class which manages a pool of database
    connections
    :type pool: ConnectionPool
    :param name: The `name` parameter is a string that represents the name of an object being
    initialized. It is used as an identifier for the object within the code
    :type name: str

    ### Methods

    `generate_chunks(self, splitter_id: int = 1) ‑> None`
    :   This function generates chunks of text from unchunked documents using a specified text splitter.
        
        :param splitter_id: The ID of the splitter to use for generating chunks, defaults to 1
        :type splitter_id: int (optional)

    `generate_embeddings(self, model_id: Optional[int] = 1, splitter_id: Optional[int] = 1) ‑> None`
    :   This function generates embeddings for chunks of text using a specified model and inserts them into
        a database table.
        
        :param model_id: The ID of the model to use for generating embeddings, defaults to 1
        :type model_id: Optional[int] (optional)
        :param splitter_id: The `splitter_id` parameter is an optional integer that specifies the ID of the
        data splitter to use for generating embeddings. If not provided, it defaults to 1, defaults to 1
        :type splitter_id: Optional[int] (optional)

    `get_models(self) ‑> List[Dict[str, Any]]`
    :   The function retrieves a list of dictionaries containing information about models from a database
        table.
        :return: The function `get_models` is returning a list of dictionaries, where each dictionary
        represents a model and contains the following keys: "id", "task", "name", and "parameters". The
        values associated with these keys correspond to the respective fields in the database table
        specified by `self.models_table`.

    `get_text_splitters(self) ‑> List[Dict[str, Any]]`
    :   This function retrieves a list of dictionaries containing information about text splitters from a
        database.
        :return: The function `get_text_splitters` is returning a list of dictionaries, where each
        dictionary contains the `id`, `name`, and `parameters` of a text splitter.

    `register_model(self, task: Optional[str] = 'embedding', model_name: Optional[str] = 'intfloat/e5-small', model_params: Optional[Dict[str, Any]] = {}) ‑> None`
    :   This function registers a model in a database if it does not already exist.
        
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

    `register_text_splitter(self, splitter_name: Optional[str] = 'RecursiveCharacterTextSplitter', splitter_params: Optional[Dict[str, Any]] = {}) ‑> None`
    :   This function registers a text splitter with a given name and parameters in a database table if it
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

    `upsert_documents(self, documents: List[Dict[str, Any]], text_key: Optional[str] = 'text', id_key: Optional[str] = 'id') ‑> None`
    :   The function `upsert_documents` inserts or updates documents in a database table based on their ID,
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

    `vector_search(self, query: str, query_parameters: Optional[Dict[str, Any]] = {}, top_k: int = 5, model_id: int = 1, splitter_id: int = 1) ‑> List[Dict[str, Any]]`
    :   This function performs a vector search on a database using a query and returns the top matching
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