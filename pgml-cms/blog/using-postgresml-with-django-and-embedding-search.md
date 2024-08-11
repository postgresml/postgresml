---
description: >-
  An example application using PostgresML and Django to build embedding based search.
tags: [engineering]
---

# Using PostgresML with Django and embedding search

<div align="left">

<figure><img src=".gitbook/assets/lev.jpg" alt="Author" width="100"><figcaption></figcaption></figure>

</div>

Lev Kokotov

Feb 15, 2024

Building web apps on top of PostgresML allows anyone to integrate advanced machine learning and AI features into their products without much work or needing to understand how it really works. In this blog post, we'll talk about building a classic to-do Django app, with the spicy addition of semantic search powered by embedding models running inside your PostgreSQL database.

### Getting the code

Our example application is on GitHub:[ https://github.com/postgresml/example-django](https://github.com/postgresml/example-django). You can fork it, clone it and run the app locally on your machine, or on any hosting platform of your choice. See the `README` for instructions on how to set it up.

### The basics

PostgresML allows anyone to integrate advanced AI capabilities into their application using only SQL. In this app, we're demonstrating embedding search: the ability to search and rank documents using their semantic meaning.

Advanced search engines like Google use this technique to extract the meaning of search queries and rank the results based on what the user actually _wants_, unlike simple keyword matches which can easily give irrelevant results.

To accomplish this, for each document in our app, we include an embedding column stored as a vector. A vector is just an array of floating point numbers. For each item in our to-do list, we automatically generate the embedding using the PostgresML [`pgml.embed()`](/docs/open-source/pgml/api/pgml.embed) function. This function runs inside the database and doesn't require the Django app to install the model locally.

An embedding model running inside PostgresML is able to extract the meaning of search queries & compare it to the meaning of the documents it stores, just like a human being would if they were able to search millions of documents in just a few milliseconds.

### The app

Our Django application has only one model, the `TodoItem`. It comes with a description, a due date, a completed flag, and the embedding column. The embedding column is using `pgvector`, another great PostgreSQL extension, which provides vector storage and nearest neighbor search. `pgvector` comes with a Django plugin so we had to do very little to get it working out of the box:

```python
embedding = models.GeneratedField(
    expression=EmbedSmallExpression("description"),
    output_field=VectorField(dimensions=768),
    db_persist=True,
)    
```

This little code snippet contains quite a bit of functionality. First, we use a `GeneratedField` which is a database column that's automatically populated with data from the database. The application doesn't need to input anything when a model instance is created. This is a very powerful technique to ensure data durability and accuracy.

Secondly, the generated column is using a `VectorField`. This comes from the `pgvector.django` package and defines a `vector(768)` column: a vector with 768 dimensions.

Lastly, the `expression` argument tells Django how to generate this field inside the database. Since PostgresML doesn't (yet) come with a Django plugin, we had to write the expression class ourselves. Thankfully, Django makes this very easy:

```python
class EmbedSmallExpression(models.Expression):
    output_field = VectorField(null=False, blank=False, dimensions=768)

    def __init__(self, field):
        self.embedding_field = field

    def as_sql(self, compiler, connection, template=None):
        return f"pgml.embed('Alibaba-NLP/gte-base-en-v1.5', {self.embedding_field})", None
```

And that's it! In just a few lines of code, we're generating and storing high quality embeddings automatically in our database. No additional setup is required, and all the AI complexity is taken care of by PostgresML.

#### API

Djago Rest Framework provides the bulk of the implementation. We just added a `ModelViewSet` for the `TodoItem` model, with just one addition: a search endpoint. The search endpoint required us to write a bit of SQL to embed the search query and accept a few filters, but the core of it can be summarized in a single annotation on the query set:

```python
results = TodoItem.objects.annotate(
    similarity=RawSQL(
        "pgml.embed('Alibaba-NLP/gte-base-en-v1.5', %s)::vector(768) &#x3C;=> embedding",
        [query],
    )
).order_by("similarity")
```

This single line of SQL does quite a bit:

1. It embeds the input query using the same model as we used to embed the description column in the model
2. It performs a cosine similarity search on the generated embedding and the embeddings of all other descriptions stored in the database
3. It ranks the result by similarity, returning the results in order of relevance, starting at the most relevant

All of this happens inside PostgresML. Our Django app doesn't need to implement any of this functionality beyond just a bit of raw SQL.

### Creating to-dos

Before going forward, make sure you have the app running either locally or in a cloud provider of your choice. If hosting it somewhere, replace `localhost:8000` with the URL and port of your service.

The simplest way to interact with it is to use cURL or your preferred HTTP client. If running in debug mode locally, the Rest Framework provides a nice web UI which you can access on [http://localhost:8000/api/todo/](http://localhost:8000/api/todo/) using a browser.

To create a to-do item with cURL, you can just run this:

```bash
curl \
    --silent \
    -X POST \
    -d '{"description": "Make a New Year resolution list", "due_date": "2025-01-01"}' \
    -H 'Content-Type: application/json' \
    http://localhost:8000/api/todo/
```

In return, you'll get your to-do item alongside the embedding of the `description` column generated by PostgresML:

```json
{
  "id": 5,
  "description": "Make a New Year resolution",
  "due_date": "2025-01-01",
  "completed": false
  "embedding": "[-2.60886201e-03 -6.66755587e-02 -9.28235054e-02  [...]]"
}
```

The embedding contains 768 floating point numbers; we removed most of them in this blog post to make sure it fits on the page.

You can try creating multiple to-do items for fun and profit. If the description is changed, so will the embedding, demonstrating how the `Alibaba-NLP/gte-base-en-v1.5` model understands the semantic meaning of your text.

### Searching

Once you have a few embeddings and to-dos stored in your database, the fun part of searching can begin. In a typical search example with PostgreSQL, you'd now be using `tsvector` to keyword match your to-dos to the search query with term frequency. That's a good technique, but semantic search is better.

Our search endpoint accepts a query, a completed to-do filter, and a limit. To use it, you can just run this:

```bash
curl \
    --silent \
    -H "Content-Type: application/json" \
    'http://localhost:8000/api/todo/search/?q=resolution&limit=1' | \
     jq ".[0].description"
```

If you've created a bunch of different to-do items, you should get only one search result back, and exactly the one you were expecting:

```json
"Make a New Year resolution"
```

You can increase the `limit` to something larger and you should get more documents, in decreasing order of relevance.

And that's it! In just a few lines of code, we built an advanced semantic search engine, previously only available to large enterprises and teams with dedicated machine learning experts. While it may not stop us from procrastinating our chores, it will definitely help us find the to-dos we really _want_ to do.

The code is available on [GitHub.](https://github.com/postgresml/example-django)

As always, if you have any feedback or thoughts, reach out to us on Discord or by email. We're always happy to talk about the cool things we can build with PostgresML!
