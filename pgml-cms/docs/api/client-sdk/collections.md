---
description: >-
  Organizational building blocks of the SDK. Manage all documents and related
  chunks, embeddings, tsvectors, and pipelines.
---

# Collections

Collections are the organizational building blocks of the SDK. They manage all documents and related chunks, embeddings, tsvectors, and pipelines.

## Creating Collections

By default, collections will read and write to the database specified by `PGML_DATABASE_URL` environment variable.

### **Default `PGML_DATABASE_URL`**

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const collection = pgml.newCollection("test_collection")
```
{% endtab %}

{% tab title="Python" %}
```python
collection = Collection("test_collection")
```
{% endtab %}

{% tab title="Rust" %}
```rust
let mut collection = Collection::new("test_collection", None)?;
```
{% endtab %}

{% tab title="C" %}
```c
CollectionC * collection = pgml_collectionc_new("test_collection", NULL);
```
{% endtab %}
{% endtabs %}

### Custom `PGML_DATABASE_URL`

Create a Collection that reads from a different database than that set by the environment variable `PGML_DATABASE_URL`.

{% tabs %}
{% tab title="Javascript" %}
```javascript
const collection = pgml.newCollection("test_collection", CUSTOM_DATABASE_URL)
```
{% endtab %}

{% tab title="Python" %}
```python
collection = Collection("test_collection", CUSTOM_DATABASE_URL)
```
{% endtab %}

{% tab title="Rust" %}
```rust
let mut collection = Collection::new("test_collection", Some(CUSTOM_DATABASE_URL))?;
```
{% endtab %}

{% tab title="C" %}
```c
CollectionC * collection = pgml_collectionc_new("test_collection", CUSTOM_DATABASE_URL);
```
{% endtab %}
{% endtabs %}

## Upserting Documents

Documents are dictionaries with one required key: `id`. All other keys/value pairs are stored and can be chunked, embedded, broken into tsvectors, and searched over as specified by a `Pipeline`.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const documents = [
  {
    id: "document_one",
    title: "Document One",
    text: "document one contents...",
    random_key: "here is some random data",
  },
  {
    id: "document_two",
    title: "Document Two",
    text: "document two contents...",
    random_key: "here is some random data",
  },
];
await collection.upsert_documents(documents);
```
{% endtab %}

{% tab title="Python" %}
```python
documents = [
    {
        "id": "document_one",
        "title": "Document One",
        "text": "Here are the contents of Document 1",
        "random_key": "here is some random data",
    },
    {
        "id": "document_two",
        "title": "Document Two",
        "text": "Here are the contents of Document 2",
        "random_key": "here is some random data",
    },
]
await collection.upsert_documents(documents)
```
{% endtab %}

{% tab title="Rust" %}
```rust
let documents: Vec<pgml::types::Json> = vec![
    serde_json::json!({
        "id": "document_one",
        "title": "Document One",
        "text": "Here are the contents of Document 1",
        "random_key": "here is some random data",
    })
    .into(),
    serde_json::json!({
        "id": "document_two",
        "title": "Document Two",
        "text": "Here are the contents of Document 2",
        "random_key": "here is some random data",
    })
    .into(),
];
collection.upsert_documents(documents, None).await?;
```
{% endtab %}

{% tab title="C" %}
```c
char * documents[2] = {
  "{\"id\": \"document_one\", \"title\": \"Document One\", \"text\": \"Here are the contents of Document 1\", \"random_key\": \"here is some random data\"}",
  "{\"id\": \"document_two\", \"title\": \"Document Two\", \"text\": \"Here are the contents of Document 2\", \"random_key\": \"here is some random data\"}"
};
pgml_collectionc_upsert_documents(collection, documents, 2, NULL);
```
{% endtab %}
{% endtabs %}

Documents can be replaced by upserting documents with the same `id`.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const documents = [
  {
    id: "document_one",
    title: "Document One New Title",
    text: "Here is some new text for document one",
    random_key: "here is some new random data",
  },
  {
    id: "document_two",
    title: "Document Two New Title",
    text: "Here is some new text for document two",
    random_key: "here is some new random data",
  },
];
await collection.upsert_documents(documents);
```
{% endtab %}

{% tab title="Python" %}
```python
documents = [
    {
        "id": "document_one",
        "title": "Document One",
        "text": "Here is some new text for document one",
        "random_key": "here is some random data",
    },
    {
        "id": "document_two",
        "title": "Document Two",
        "text": "Here is some new text for document two",
        "random_key": "here is some random data",
    },
]
await collection.upsert_documents(documents)
```
{% endtab %}

{% tab title="Rust" %}
```rust
let documents: Vec<pgml::types::Json> = vec![
    serde_json::json!({
        "id": "document_one",
        "title": "Document One",
        "text": "Here is some new text for document one",
        "random_key": "here is some random data",
    })
    .into(),
    serde_json::json!({
        "id": "document_two",
        "title": "Document Two",
        "text": "Here is some new text for document two",
        "random_key": "here is some random data",
    })
    .into(),
];
collection.upsert_documents(documents, None).await?;
```
{% endtab %}

{% tab title="C" %}
```c
char * documents[2] = {
  "{\"id\": \"document_one\", \"title\": \"Document One\", \"text\": \"Here is some new text for document one\", \"random_key\": \"here is some random data\"}",
  "{\"id\": \"document_two\", \"title\": \"Document Two\", \"text\": \"Here is some new text for document two\", \"random_key\": \"here is some random data\"}"
};
pgml_collectionc_upsert_documents(collection, documents, 2, NULL);
```
{% endtab %}
{% endtabs %}

Documents can be merged by setting the `merge` option. On conflict, new document keys will override old document keys.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const documents = [
  {
    id: "document_one",
    new_key: "this will be a new key in document one",
    random_key: "this will replace old random_key"
  },
  {
    id: "document_two",
    new_key: "this will bew a new key in document two",
    random_key: "this will replace old random_key"
  },
];
await collection.upsert_documents(documents, {
  merge: true
});
```
{% endtab %}

{% tab title="Python" %}
```python
documents = [
    {
        "id": "document_one",
        "new_key": "this will be a new key in document one",
        "random_key": "this will replace old random_key",
    },
    {
        "id": "document_two",
        "new_key": "this will be a new key in document two",
        "random_key": "this will replace old random_key",
    },
]
await collection.upsert_documents(documents, {"merge": True})
```
{% endtab %}

{% tab title="Rust" %}
```rust
let documents: Vec<pgml::types::Json> = vec![
    serde_json::json!({
        "id": "document_one",
        "new_key": "this will be a new key in document one",
        "random_key": "this will replace old random_key"
    })
    .into(),
    serde_json::json!({
        "id": "document_two",
        "new_key": "this will be a new key in document two",
        "random_key": "this will replace old random_key"
    })
    .into(),
];
collection
    .upsert_documents(documents, Some(serde_json::json!({"merge": true}).into()))
    .await?;
```
{% endtab %}

{% tab title="C" %}
```c
char * documents[2] = {
  "{\"id\": \"document_one\", \"new_key\": \"this will be a new key in document one\", \"random_key\": \"this will replace old random_key\"}",
  "{\"id\": \"document_two\", \"new_key\": \"this will be a new key in document two\", \"random_key\": \"this will replace old random_key\"}"
};
pgml_collectionc_upsert_documents(collection, documents, 2, "{\"merge\": true}");
```
{% endtab %}
{% endtabs %}

## Getting Documents

Documents can be retrieved using the `get_documents` method on the collection object.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const documents = await collection.get_documents({limit: 100 })
```
{% endtab %}

{% tab title="Python" %}
```python
documents = await collection.get_documents({ "limit": 100 })
```
{% endtab %}

{% tab title="Rust" %}
```rust
let documents = collection
    .get_documents(Some(serde_json::json!({"limit": 100}).into()))
    .await?;
```
{% endtab %}

{% tab title="C" %}
```c
unsigned long r_size = 0;
char** documents = pgml_collectionc_get_documents(collection, "{\"limit\": 100}", &r_size);
```
{% endtab %}
{% endtabs %}

### Paginating Documents

The SDK supports limit-offset pagination and keyset pagination.

#### Limit-Offset Pagination

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const documents = await collection.get_documents({ limit: 100, offset: 10 })
```
{% endtab %}

{% tab title="Python" %}
```python
documents = await collection.get_documents({ "limit": 100, "offset": 10 })
```
{% endtab %}

{% tab title="Rust" %}
```rust
let documents = collection
    .get_documents(Some(serde_json::json!({"limit": 100, "offset": 10}).into()))
    .await?;
```
{% endtab %}

{% tab title="C" %}
```c
unsigned long r_size = 0;
char** documents = pgml_collectionc_get_documents(collection, "{\"limit\": 100, \"offset\": 10}", &r_size);
```
{% endtab %}
{% endtabs %}

#### Keyset Pagination

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const documents = await collection.get_documents({ limit: 100, last_row_id: 10 })
```
{% endtab %}

{% tab title="Python" %}
```python
documents = await collection.get_documents({ "limit": 100, "last_row_id": 10 })
```
{% endtab %}

{% tab title="Rust" %}
```rust
let documents = collection
    .get_documents(Some(serde_json::json!({"limit": 100, "last_row_id": 10}).into()))
    .await?;
```
{% endtab %}

{% tab title="C" %}
```c
unsigned long r_size = 0;
char** documents = pgml_collectionc_get_documents(collection, "{\"limit\": 100, \"last_row_id\": 10}", &r_size);
```
{% endtab %}
{% endtabs %}

The `last_row_id` can be taken from the `row_id` field in the returned document's dictionary. Keyset pagination does not currently work when specifying the `order_by` key.

### Filtering Documents

Documents can be filtered by passing in the `filter` key.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const documents = await collection.get_documents({
  limit: 10,
  filter: {
    id: {
      $eq: "document_one"
    }
  }
})
```
{% endtab %}

{% tab title="Python" %}
```python
documents = await collection.get_documents(
    {
        "limit": 100,
        "filter": {
            "id": {"$eq": "document_one"},
        },
    }
)
```
{% endtab %}

{% tab title="Rust" %}
```rust
let documents = collection
    .get_documents(Some(
        serde_json::json!({
            "limit": 100,
            "filter": {
                "id": {"$eq": "document_one"},
            }
        })
        .into(),
    ))
    .await?;
```
{% endtab %}

{% tab title="C" %}
```c
unsigned long r_size = 0;
char** documents = pgml_collectionc_get_documents(collection, "{\"limit\": 100, \"filter\": {\"id\": {\"$eq\": \"document_one\"}}}", &r_size);
```
{% endtab %}
{% endtabs %}

### Sorting Documents

Documents can be sorted on any key. Note that this does not currently work well with Keyset based pagination. If paginating and sorting, use Limit-Offset based pagination.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const documents = await collection.get_documents({
  limit: 100,
  offset: 10,
  order_by: {
    id: "desc"
  }
})
```
{% endtab %}

{% tab title="Python" %}
```python
documents = await collection.get_documents({
    "limit": 100,
    "offset": 10,
    "order_by": {
        "id": "desc"
    }
})
```
{% endtab %}

{% tab title="Rust" %}
```rust
let documents = collection
    .get_documents(Some(
        serde_json::json!({
            "limit": 100,
            "offset": 10,
            "order_by": {
                "id": "desc"
            }
        })
        .into(),
    ))
    .await?;
```
{% endtab %}

{% tab title="C" %}
```c
unsigned long r_size = 0;
char** documents = pgml_collectionc_get_documents(collection, "{\"limit\": 100, \"offset\": 10, \"order_by\": {\"id\": \"desc\"}}", &r_size);
```
{% endtab %}
{% endtabs %}

### Deleting Documents

Documents can be deleted with the `delete_documents` method on the collection object.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const documents = await collection.delete_documents({
    id: {
      $eq: 1
    }
})
```
{% endtab %}

{% tab title="Python" %}
```python
documents = await collection.delete_documents(
    {
        "id": {"$eq": 1},
    }
)
```
{% endtab %}

{% tab title="Rust" %}
```rust
let documents = collection
    .delete_documents(
        serde_json::json!({
            "id": {
                "$eq": 1
            }
        })
        .into(),
    )
    .await?;
```
{% endtab %}

{% tab title="C" %}
```c
pgml_collectionc_delete_documents(collection, "{\"id\": { \"$eq\": 1}}");
```
{% endtab %}
{% endtabs %}
