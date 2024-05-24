# Documents

Documents are a type of data that has no predefined schema. Typically stored as JSON, documents can be used to allow users of your application to store, retrieve and work with arbitrary data.

In Postgres, documents are normally stored in regular tables using the `JSONB` data type. `JSONB` supports compression, indexing and various JSON operators that make it useful and performant.

### Storage & retrieval

If you're used to document databases like Mongo or Couch, you can replicate the same format and API in Postgres with just a single table:

```postgresql
CREATE TABLE documents (
    id BIGSERIAL PRIMARY KEY,
    document JSONB
);
```

#### Inserting a document

To insert a document into our table, you can just use a regular insert query:

```postgresql
INSERT INTO documents (
    document
) VALUES ('{"hello": "world", "values": [1, 2, 3, 4]}')
RETURNING id;
```

This query will insert the document `{"hello": "world"}` and return its ID to the application. You can then pass this ID to your users or store it elsewhere for reference.

#### Fetching by ID

To get a document by it's ID, you can just select it from the same table, for example:

```postgresql
SELECT document FROM documents WHERE id = 1;
```

The `id` column is a primary key, which gives it an index automatically. Any fetch by ID will be very quick and can easily retrieve documents from a table storing millions and even billions of documents.

#### Fetching by value

`JSONB` supports many operators to access the data stored in the column:

| Operator | Description                                                | Example                                                                                                          |
| -------- | ---------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| `->`     | Get the value referenced by the key and return it as JSON. | `document->'hello'` will return `"world"` which is a valid JSON string.                                          |
| `->>`    | Get the value referenced by the key and return it as text. | `document->>'hello'` will return `world` which is a PostgreSQL `VARCHAR`.                                        |
| `#>`     | Get a nested value by the key and return it as JSON.       | `document #> {values, 0}` will get the first value in the `values` array and return it as JSON.                  |
| `#>>`    | Get a nested value by the key and return it as text.       | `document #>> {values, 0}` will get the first value in the `values` array and return it as PostgreSQL `VARCHAR`. |
| `@>`     | Checks if the document contains a key/value match.         | `document @> {"hello": "world"}` will return true if the `document` has a key `hello` and a value `world`.       |

For example, if we want to fetch all documents that have a key `hello` and the value of that key `world`, we can do so:

```postgresql
SELECT
    id,
    document->>'values'
FROM documents
WHERE
    document @> '{"hello": "world"}';
```

or if we wanted to fetch the first value inside an array stored in a `values` key, we can:

```postgresql
SELECT
    document #>> '{values, 0}'
FROM documents
WHERE
    document @> '{"hello": "world"}';
```

`JSONB` handles empty, null, or non-existent keys and values without any errors. If the key doesn't exist, a `null` will be returned, just like if we were to access the JSON object from JavaScript.

### Indexing documents

Most key/value databases expect its users to only use primary keys for retrieval. In the real world, things are not always that easy. Postgres makes very few assumptions about how its users interact with JSON data, and allows indexing its top level data structure for fast access:

```postgresql
CREATE INDEX ON documents USING gin(document jsonb_path_ops);
```

When searching the documents for matches, Postgres will now use a much faster GIN index and give us results quickly:

```postgresql
SELECT
    * 
FROM
    documents
WHERE document @> '{"hello": "world"}';
```

We're using the `@>` operator which checks if the `document` column top level JSON structure contains a key `hello` with the value `world`.
