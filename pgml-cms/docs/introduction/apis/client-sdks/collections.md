# Collections

Collections are the organizational building blocks of the SDK. They manage all documents and related chunks, embeddings, tsvectors, and pipelines.

## Creating Collections

By default, collections will read and write to the database specified by `DATABASE_URL` environment variable.

### **Default `DATABASE_URL`**

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

### **Custom DATABASE\_URL**

Create a Collection that reads from a different database than that set by the environment variable `DATABASE_URL`.

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

Documents are dictionaries with two required keys: `id` and `text`. All other keys/value pairs are stored as metadata for the document.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const documents = [
  {
    id: "Document One",
    text: "document one contents...",
    random_key: "this will be metadata for the document",
  },
  {
    id: "Document Two",
    text: "document two contents...",
    random_key: "this will be metadata for the document",
  },
];
await collection.upsert_documents(documents);
```
{% endtab %}

{% tab title="Python" %}
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
{% endtab %}
{% endtabs %}

Document metadata can be replaced by upserting the document without the `text` key.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const documents = [
  {
    id: "Document One",
    random_key: "this will be NEW metadata for the document",
  },
  {
    id: "Document Two",
    random_key: "this will be NEW metadata for the document",
  },
];
await collection.upsert_documents(documents);
```
{% endtab %}

{% tab title="Python" %}
```python
documents = [
    {
        "id": "Document 1",
        "random_key": "this will be NEW metadata for the document"
    },
    {
        "id": "Document 2",
        "random_key": "this will be NEW metadata for the document"
    }
]
collection = Collection("test_collection")
await collection.upsert_documents(documents)
```
{% endtab %}
{% endtabs %}

Document metadata can be merged with new metadata by upserting the document without the `text` key and specifying the merge option.

{% tabs %}
{% tab title="JavaScript" %}
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
await collection.upsert_documents(documents, {
  metdata: {
    merge: true
  }
});
```
{% endtab %}

{% tab title="Python" %}
```python
documents = [
    {
        "id": "Document 1",
        "random_key": "this will be NEW merged metadata for the document"
    },
    {
        "id": "Document 2",
        "random_key": "this will be NEW merged metadata for the document"
    }
]
collection = Collection("test_collection")
await collection.upsert_documents(documents, {
    "metadata": {
        "merge": True
    }
})
```
{% endtab %}
{% endtabs %}

## Getting Documents

Documents can be retrieved using the `get_documents` method on the collection object.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const collection = Collection("test_collection")
const documents = await collection.get_documents({limit: 100 })
```
{% endtab %}

{% tab title="Python" %}
```python
collection = Collection("test_collection")
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
const collection = pgml.newCollection("test_collection")
const documents = await collection.get_documents({ limit: 100, offset: 10 })
```
{% endtab %}

{% tab title="Python" %}
```python
collection = Collection("test_collection")
documents = await collection.get_documents({ "limit": 100, "offset": 10 })
```
{% endtab %}
{% endtabs %}

#### Keyset Pagination

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const collection = Collection("test_collection")
const documents = await collection.get_documents({ limit: 100, last_row_id: 10 })
```
{% endtab %}

{% tab title="Python" %}
```python
collection = Collection("test_collection")
documents = await collection.get_documents({ "limit": 100, "last_row_id": 10 })
```
{% endtab %}
{% endtabs %}

The `last_row_id` can be taken from the `row_id` field in the returned document's dictionary.

### Filtering Documents

Metadata and full text filtering are supported just like they are in vector recall.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const collection = pgml.newCollection("test_collection")
const documents = await collection.get_documents({
  limit: 100,
  offset: 10,
  filter: {
    metadata: {
      id: {
        $eq: 1
      }
    },
    full_text_search: {
      configuration: "english",
      text: "Some full text query"
    }
  }
})
```
{% endtab %}

{% tab title="Python" %}
```python
collection = Collection("test_collection")
documents = await collection.get_documents({
    "limit": 100,
    "offset": 10,
    "filter": {
        "metadata": {
            "id": {
                "$eq": 1
            }
        },
        "full_text_search": {
            "configuration": "english",
            "text": "Some full text query"
        }
    }
})
```
{% endtab %}
{% endtabs %}

### Sorting Documents

Documents can be sorted on any metadata key. Note that this does not currently work well with Keyset based pagination. If paginating and sorting, use Limit-Offset based pagination.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const collection = pgml.newCollection("test_collection")
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
collection = Collection("test_collection")
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

Metadata and full text filtering are supported just like they are in vector recall.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const collection = pgml.newCollection("test_collection")
const documents = await collection.delete_documents({
  metadata: {
    id: {
      $eq: 1
    }
  },
  full_text_search: {
    configuration: "english",
    text: "Some full text query"
  }
})
```
{% endtab %}

{% tab title="Python" %}
```python
documents = await collection.delete_documents({
    "metadata": {
        "id": {
            "$eq": 1
        }
    },
    "full_text_search": {
        "configuration": "english",
        "text": "Some full text query"
    }
})
```
{% endtab %}
{% endtabs %}
