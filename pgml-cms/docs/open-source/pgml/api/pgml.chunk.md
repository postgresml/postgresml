---
description: Split some text into chunks using the specified splitter.
---

# pgml.chunk()

Chunks are pieces of documents split using some specified splitter. This is typically done before embedding.

## API

```postgresql
pgml.chunk(
    splitter TEXT,    -- splitter name
    text TEXT,        -- text to embed
    kwargs JSON       -- optional arguments (see below)
)
```

## Examples

```postgresql
SELECT pgml.chunk('recursive_character', 'test');
```

```postgresql
SELECT pgml.chunk('recursive_character', 'test', '{"chunk_size": 1000, "chunk_overlap": 40}'::jsonb);
```

```postgresql
SELECT pgml.chunk('markdown', '# Some test');
```

Note that the input text for those splitters is so small it isn't splitting it at all, a real world example would look more like:

```postgresql
SELECT pgml.chunk('recursive_character', content) FROM documents;
```

Where `documents` is some table that has a `text` column called `content`

## Supported Splitters

We support the following splitters:

* `recursive_character`
* `latex`
* `markdown`
* `ntlk`
* `python`
* `spacy`

For more information on splitters see[ LangChain's docs ](https://python.langchain.com/docs/modules/data\_connection/document\_transformers/)
