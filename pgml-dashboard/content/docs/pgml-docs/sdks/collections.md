# Collections

Collections are the organizational building blocks of the SDK. They manage all documents and related chunks, embeddings, tsvectors, and pipelines.

## Creating Collections

By default, collections will read and write to the database specified by `DATABASE_URL`.

### **Default `DATABASE_URL`**

=== "Python"
```python
collection = Collection("test_collection")
```
=== "JavaScript"
```javascript
collection = pgml.newCollection("test_collection")
```
=== 

### **Custom DATABASE\_URL**

Create a Collection that reads from a different database than that set by the environment variable `DATABASE_URL`.

=== "Python"
```python
collection = Collection("test_collection", CUSTOM_DATABASE_URL)
```
=== "JavaScript"
```javascript
collection = pgml.newCollection("test_collection", CUSTOM_DATABASE_URL)
```
=== 


## Upserting Documents

Documents are dictionaries with two required keys: `id` and `text`. All other keys/value pairs are stored as metadata for the document.

**Upsert documents with metadata**

=== "Python"
```python
documents = [
    {
        "id": "Document 1",
        "text": "Here are the contents of Document 1",
        "random_key": "this will be metadata for the document"
    },
    {
        "id": "Document 2",
        "text": "Here are the contents of Document 2",
        "random_key": "this will be metadata for the document"
    }
]
collection = Collection("test_collection")
await collection.upsert_documents(documents)
```
=== "JavaScript"
```javascript
  const documents = [
            {
              id: "Document One",
              text: "document one contents...",
            },
            {
              id: "Document Two",
              text: "document two contents...",
            },
    ];
    await collection.upsert_documents(documents);
```
=== 
