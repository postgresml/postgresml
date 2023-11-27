# Getting Started

## Installation

{% tabs %}
{% tab title="Python " %}
Python > 3.8.1

```bash
pip install pgml
```
{% endtab %}

{% tab title="JavaScript " %}
```
npm i pgml
```
{% endtab %}
{% endtabs %}



## Example

Once the SDK is installed, you an use the following example to get started.

### Create a collection

{% tabs %}
{% tab title="Python" %}
```python
from pgml import Collection, Model, Splitter, Pipeline
import asyncio

async def main():
    # Initialize collection
    collection = Collection("sample_collection")
```
{% endtab %}

{% tab title="JavaScript " %}
```javascript
const pgml = require("pgml");

const main = async () => {
    collection = pgml.newCollection("sample_collection");
```
{% endtab %}
{% endtabs %}

**Explanation:**

* The code imports the pgml module.
* It creates an instance of the Collection class which we will add pipelines and documents onto

### Create a pipeline

Continuing with `main`

{% tabs %}
{% tab title="Python" %}
```python
    # Create a pipeline using the default model and splitter
    model = Model()
    splitter = Splitter()
    pipeline = Pipeline("sample_pipeline", model, splitter)
    await collection.add_pipeline(pipeline)
```
{% endtab %}

{% tab title="JavaScript" %}
```javascript
    model = pgml.newModel();
    splitter = pgml.newSplitter();
    pipeline = pgml.Pipeline("sample_pipeline", model, splitter);
    await collection.add_pipeline(pipeline);
```
{% endtab %}
{% endtabs %}

#### Explanation:

* The code creates an instance of `Model` and `Splitter` using their default arguments.
* Finally, the code constructs a pipeline called `"sample_pipeline"` and add it to the collection we Initialized above. This pipeline automatically generates chunks and embeddings for every upserted document.

### Upsert documents

Continuing with `main`

{% tabs %}
{% tab title="Python" %}
```python
    documents = [
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
{% endtabs %}

**Explanation**

* This code creates and upserts some filler documents.
* As mentioned above, the pipeline added earlier automatically runs and generates chunks and embeddings for each document.

### Query documents

Continuing with `main`

{% tabs %}
{% tab title="Python" %}
```python
    # Query
    query = "Some user query that will match document one first"
    results = await collection.query().vector_recall(query, pipeline).limit(2).fetch_all()
    print(results)
    # Archive collection
    await collection.archive()
```
{% endtab %}

{% tab title="JavaScript" %}
```javascript
const queryResults = await collection
        .query()
        .vector_recall("Some user query that will match document one first", pipeline)
        .limit(2)
        .fetch_all();

    // Convert the results to an array of objects
    const results = queryResults.map((result) => {
      const [similarity, text, metadata] = result;
      return {
        similarity,
        text,
        metadata,
      };
    });
    console.log(results);

    await collection.archive();
```
{% endtab %}
{% endtabs %}

**Explanation:**

* The `query` method is called to perform a vector-based search on the collection. The query string is `Some user query that will match document one first`, and the top 2 results are requested.
* The search results are converted to objects and printed.
* Finally, the `archive` method is called to archive the collection and free up resources in the PostgresML database.

Call `main` function.

{% tabs %}
{% tab title="Python" %}
```python
if __name__ == "__main__":
    asyncio.run(main())
```
{% endtab %}

{% tab title="JavaScript" %}
```javascript
main().then(() => {
  console.log("Done with PostgresML demo");
});
```
{% endtab %}
{% endtabs %}

### **Running the Code**

Open a terminal or command prompt and navigate to the directory where the file is saved.

Execute the following command:

{% tabs %}
{% tab title="Python" %}
```bash
python vector_search.py
```
{% endtab %}

{% tab title="JavaScript" %}
```bash
node vector_search.js
```
{% endtab %}
{% endtabs %}

You should see the search results printed in the terminal. As you can see, our vector search engine did match document one first.

```bash
[
  {
    similarity: 0.8506832955692104,
    text: 'document one contents...',
    metadata: { id: 'Document One' }
  },
  {
    similarity: 0.8066114609244565,
    text: 'document two contents...',
    metadata: { id: 'Document Two' }
  }
]
```
