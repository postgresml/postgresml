# Documentation for FastAPI Example

This document provides an overview of the available API endpoints `/ingest` and `/search`, how to use them, and how to run the server.

## Run the Server

To run the server, use the following command:

```bash
poetry run python3 examples/fastapi-server/app/main.py
```


## API Endpoints

### `/ingest` Endpoint

This endpoint allows you to ingest documents for processing. It expects a POST request with a JSON body that contains the details of the documents to be processed.

#### Request

- **URL:** `http://0.0.0.0:8888/ingest`
- **Method:** `POST`
- **Content-Type:** `application/json`
- **Body:**

```json
{
    "collection_name": "<name_of_collection>",
    "document_path": "<path_to_document>"
}
```

#### Example

You can use the following `curl` command as an example:

```bash
curl --location 'http://0.0.0.0:8888/ingest' \
--header 'Content-Type: application/json' \
--data '{
    "collection_name": "test_collection",
    "document_path": "~/path/to/pdf"
}'
```

### `/search` Endpoint

This endpoint allows you to search within a given collection. It expects a POST request with a JSON body that contains the details of the search request.


#### Request

- **URL:** `http://0.0.0.0:8888/search`
- **Method:** `POST`
- **Content-Type:** `application/json`
- **Body:**

```json
{
    "collection_name": "<name_of_collection>",
    "question": "<search_query>",
    "k": "<number_of_results>",
    "metadata_filter": "<metadata_filter>"
}
```

Note: The `k` and `metadata_filter` fields are optional. The `k` field is used to limit the number of search results, and the metadata_filter field is used to add additional filters on the metadata of the documents.

### Example Request
You can use the following `curl` command as an example:

```bash
curl --location 'http://0.0.0.0:8888/search' \
--header 'Content-Type: application/json' \
--data '{
    "collection_name": "testing",
    "question": "What people did he met?"
}'
```