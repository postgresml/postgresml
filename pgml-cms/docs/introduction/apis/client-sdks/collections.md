---
description: Organizational building blocks of the SDK. Manage all documents and related chunks, embeddings, tsvectors, and pipelines.
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
{% endtabs %}

Documents  can be merged by setting the `merge` option. On conflict, new document keys will override old document keys.

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
{% endtabs %}
