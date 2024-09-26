---
description: The Python module that seamlessly integrates PostgresML and Django ORM
featured: true
tags: [product]
image: ".gitbook/assets/django-pgml_blog-image.png"
---

# Announcing postgresml-django

<div align="left">

<figure><img src=".gitbook/assets/silas.jpg" alt="Author" width="100"><figcaption></figcaption></figure>

</div>

Silas Marvin

September 10, 2024

We're excited to announce the release of [postgresml-django](https://github.com/postgresml/postgresml-django), a Python module that bridges the gap between PostgresML and Django ORM. This powerful tool enables automatic in-database embedding of Django models, simplifying the process of creating and searching vector embeddings for your text data.

With postgresml-django, you can:
- Automatically generate in-database embeddings for specified fields in your Django models
- Perform vector similarity searches directly in your database
- Seamlessly integrate advanced machine learning capabilities into your Django projects

Whether you're building a recommendation system, a semantic search engine, or any application requiring text similarity comparisons, postgresml-django streamlines your workflow and enhances your Django projects with the power of PostgresML.

## Quick start

Here's a simple example of how to use postgresml-django with a Django model:

```python
from django.db import models
from postgresml_django import VectorField, Embed

class Document(Embed):
    text = models.TextField()
    text_embedding = VectorField(
        field_to_embed="text",
        dimensions=384,
        transformer="intfloat/e5-small-v2"
    )

# Searching
results = Document.vector_search("text_embedding", "query to search against")
```

In this example, we define a `Document` model with a `text` field and a `text_embedding` VectorField. The VectorField automatically generates embeddings for the `text` field using the specified transformer. The `vector_search` method allows for easy similarity searches based on these embeddings.

## Why we are excited about this

There are ton of reasons we are excited for this release but they can all be summarized by two main points:

1. Simplicity: postgresml-django integrates advanced machine learning capabilities into Django projects with just a few lines of code, making it accessible to developers of all skill levels.
2. Performance: By leveraging PostgresML to perform vector operations directly in the database, it significantly improves speed and efficiency, especially when dealing with large datasets.

By bridging Django ORM and PostgresML, we're opening up new possibilities for building intelligent, data-driven applications with ease.

## Recap

postgresml-django marks a significant step forward in making advanced machine learning capabilities accessible to Django developers. We invite you to try it out and experience the power of seamless vector embeddings and similarity searches in your projects.

For more detailed information, installation instructions, and advanced usage examples, check out the [postgresml-django GitHub repository](https://github.com/postgresml/postgresml-django). We're eager to hear your feedback and see the innovative ways you'll use postgresml-django in your applications.

Happy coding!
