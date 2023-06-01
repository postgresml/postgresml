Module pgml.database
====================

Classes
-------

`Database(conninfo: str, min_connections: Optional[int] = 1)`
:   This function initializes a connection pool and creates a table in a PostgreSQL database if it does
    not already exist.
    
    :param conninfo: A string containing the connection information for the PostgreSQL database, such
    as the host, port, database name, username, and password
    :type conninfo: str
    :param min_connections: The minimum number of connections that should be maintained in the
    connection pool at all times. If there are no available connections in the pool when a new
    connection is requested, a new connection will be created up to the maximum size of the pool,
    defaults to 1
    :type min_connections: Optional[int] (optional)

    ### Methods

    `archive_collection(self, name: str) ‑> None`
    :   This function deletes a PostgreSQL schema if it exists.
        
        :param name: The name of the collection (or schema) to be deleted
        :type name: str

    `create_or_get_collection(self, name: str) ‑> pgml.collection.Collection`
    :   This function creates a new collection in a PostgreSQL database if it does not already exist and
        returns a Collection object.
        
        :param name: The name of the collection to be created
        :type name: str
        :return: A Collection object is being returned.