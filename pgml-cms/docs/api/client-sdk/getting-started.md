# Overview

## Installation

{% tabs %}
{% tab title="JavaScript " %}
```bash
npm i pgml
```
{% endtab %}

{% tab title="Python " %}
```bash
pip install pgml
```
{% endtab %}
{% endtabs %}

## Example

Once the SDK is installed, you an use the following example to get started.

### Create a collection

{% tabs %}
{% tab title="JavaScript " %}
```javascript
const pgml = require("pgml");

const main = async () => { // Open the main function
  collection = pgml.newCollection("sample_collection");
```
{% endtab %}

{% tab title="Python" %}
```python
from pgml import Collection, Pipeline
import asyncio

async def main(): # Start of the main function
    collection = Collection("sample_collection")
```
{% endtab %}
{% endtabs %}

**Explanation:**

* The code imports the pgml module.
* It creates an instance of the Collection class which we will add pipelines and documents onto

### Create a pipeline

Continuing with `main`

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const pipeline = pgml.newPipeline("sample_pipeline", {
  text: {
    splitter: { model: "recursive_character" },
    semantic_search: {
      model: "intfloat/e5-small",
    },
  },
});
await collection.add_pipeline(pipeline);
```
{% endtab %}

{% tab title="Python" %}
```python
pipeline = Pipeline(
    "test_pipeline",
    {
        "text": {
            "splitter": { "model": "recursive_character" },
            "semantic_search": {
                "model": "intfloat/e5-small",
            },
        },
    },
)
await collection.add_pipeline(pipeline)
```
{% endtab %}
{% endtabs %}

#### Explanation:

* The code constructs a pipeline called `"sample_pipeline"` and adds it to the collection we Initialized above. This pipeline automatically generates chunks and embeddings for the `text` key for every upserted document.

### Upsert documents

Continuing with `main`

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
await collection.upsert_documents(documents);
```
{% endtab %}

{% tab title="Python" %}
```python
documents = [
    {
        "id": "Document One",
        "text": "document one contents...",
    },
    {
        "id": "Document Two",
        "text": "document two contents...",
    },
]
await collection.upsert_documents(documents)
```
{% endtab %}
{% endtabs %}

**Explanation**

* This code creates and upserts some filler documents.
* As mentioned above, the pipeline added earlier automatically runs and generates chunks and embeddings for each document.

### Query documents

Continuing with `main`

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const results = await collection.vector_search(
  {
    query: {
      fields: {
        text: {
          query: "Something about a document...",
        },
      },
    },
    limit: 2,
  },
  pipeline,
);

console.log(results);

await collection.archive();

} // Close the main function
```
{% endtab %}

{% tab title="Python" %}
```python
results = await collection.vector_search(
    {
        "query": {
            "fields": {
                "text": {
                    "query": "Something about a document...",
                },
            },
        },
        "limit": 2,
    },
    pipeline,
)

print(results)

await collection.archive()

# End of the main function
```
{% endtab %}
{% endtabs %}

**Explanation:**

* The `query` method is called to perform a vector-based search on the collection. The query string is `Something about a document...`, and the top 2 results are requested
* The search results are  printed to the screen
* Finally, the `archive` method is called to archive the collection

Call `main` function.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
main().then(() => {
  console.log("Done with PostgresML demo");
});
```
{% endtab %}

{% tab title="Python" %}
```python
if __name__ == "__main__":
    asyncio.run(main())
```
{% endtab %}
{% endtabs %}

### **Running the Code**

Open a terminal or command prompt and navigate to the directory where the file is saved.

Execute the following command:

{% tabs %}
{% tab title="JavaScript" %}
```bash
node vector_search.js
```
{% endtab %}

{% tab title="Python" %}
```bash
python3 vector_search.py
```
{% endtab %}
{% endtabs %}

You should see the search results printed in the terminal.

```bash
[
    {
        "chunk": "document one contents...",
        "document": {"id": "Document One", "text": "document one contents..."},
        "score": 0.9034339189529419,
    },
    {
        "chunk": "document two contents...",
        "document": {"id": "Document Two", "text": "document two contents..."},
        "score": 0.8983734250068665,
    },
]
```
