# Constructing Pipelines

Pipelines are a powerful feature for processing and preparing documents for efficient search and retrieval. They define a series of transformations applied to your data, enabling operations like text splitting, semantic embedding, and full-text search preparation. This guide will walk you through the process of constructing Pipeline schemas, allowing you to customize how your documents are processed and indexed.

If you are looking for information on how to work with Pipelines and Collections review the [Pipelines API](../api/pipelines).

Pipelines are specified as JSON. If you are working in Python or JavaScript they are objects. For this guide we will be writing everything in Python but it can be easily translated to work with JavaScript, Rust, or C.

For this guide, we'll use a simple document structure as an example. Understanding your document structure is crucial for creating an effective Pipeline, as it determines which fields you'll process:
```python
example_document = {
  "id": "doc_001",  # Unique identifier for the document
  "title": "Introduction to Machine Learning",  # Document title
  "text": "Machine learning is a branch of artificial intelligence..."  # Main content
}
```

Your Pipeline will define how to process these fields.

## Pipeline Structure and Components

Pipelines can apply three different transformations:
- Splitting
- Embedding
- Creating tsvectors

Here is an example Pipeline that will split, embed, and generate tsvectors for the `text` key of documents.

```python
pipeline = Pipeline(
    "v0",
    {
        "text": {
            "splitter": {"model": "recursive_character"},
            "semantic_search": {
                "model": "Alibaba-NLP/gte-base-en-v1.5",
            },
            "full_text_search": {
              "configuration": "english"
            }
        },
    },
)
```

The first argument to the `Pipeline` constructor is the name, the second is the schema.

Let's break the schema down.

First, as specified above, we are specifying the `text` key. This means the transformation object applies only to the `text` key of the document.

The `text` object contains three different keys:
- `splitter`
- `semantic_search`
- `full_text_search`

Let's break each down indiviually.

### Splitter

The `splitter` object takes two parameters:
- `model`
- `parameters`

The `model` is the string name of the model to use for splitting.

The `parameters` is an optional object specifying what parameters to pass to the splitter model.

It is common to adjust the max chunk size and overlap for the `recursive_character` splitter. An example pipeline doing this:
```python
pipeline = Pipeline(
    "v0",
    {
        "text": {
            "splitter": {
              "model": "recursive_character",
              "parameters": {
                "chunk_size": 1500,
                "chunk_overlap": 40
              }
            },
            "semantic_search": {
                "model": "Alibaba-NLP/gte-base-en-v1.5",
            },
            "full_text_search": {
              "configuration": "english"
            }
        },
    },
)
```

### Semantic Search

The `semantic_search` object takes two parameters:
- `model`
- `parameters`

The `model` is the string name of the model to use for embedding.

The `parameters` is an optional object specifying what parameters to pass to the splitter model.

It is common for embedding models to require some kind of prompt when generating embeddings. For example the popular `intfloat/e5-small-v2` requires that embeddings for storage be prefixed with `passage: `. This can be done with the following `Pipeline`:

```python
pipeline = Pipeline(
    "v0",
    {
        "text": {
            "splitter": {"model": "recursive_character"},
            "semantic_search": {
                "model": "intfloat/e5-small-v2",
                "parameters": {
                  "prompt": "passage: "
                }
            },
            "full_text_search": {
              "configuration": "english"
            }
        },
    },
)
```

### Full Text Search

The `full_text_search` object only takes one key: `configuration`. The `configuration` key is passed directly to the [`to_tsvector` function](https://www.postgresql.org/docs/current/textsearch-controls.html).

This will most likely be the language you want to enable full text search for. A common one is `english`.

If you want to perform hybrid search you must supply the `full_text_search` key.

## Transforming Multiple Fields

It is common to perform search over more than one field of a document. We must specify the keys we plan to search over in our Pipeline schema.

```python
pipeline = Pipeline(
    "v0",
    {
        "abstract": {
            "semantic_search": {
                "model": "Alibaba-NLP/gte-base-en-v1.5",
            },
            "full_text_search": {
              "configuration": "english"
            }
        },
        "text": {
            "splitter": {"model": "recursive_character"},
            "semantic_search": {
                "model": "Alibaba-NLP/gte-base-en-v1.5",
            },
            "full_text_search": {
              "configuration": "english"
            }
        },
    },
)
```

The `Pipeline` above generates embeddings and tsvectors for the `abstract` and splits and generates embeddings and tsvectors for the `text`.

We can now perform search over both the `text` and `abstract` key of our documents. See the [guide for vector search](vector-search) for more information on how to do this.
