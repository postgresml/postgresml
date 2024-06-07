---
description: Rank documents against a piece of text using the specified ranking model.
---

# pgml.rank()

The `pgml.rank()` function is used to compute a relevance score between documents and some text. This function is primarily used as the last step in a search system where the results returned from the initial search are re-ranked by relevance before being used.

## API

```postgresql
pgml.rank(
    transformer TEXT,  -- transformer name
    query TEXT,        -- text to rank against
    documents TEXT[],  -- documents to rank
    kwargs JSON        -- optional arguments (see below)
)
```

## Example

```postgresql
SELECT pgml.rank('mixedbread-ai/mxbai-rerank-base-v1', 'test', ARRAY['doc1', 'doc2']);
```

```postgresql
SELECT pgml.chunk('mixedbread-ai/mxbai-rerank-base-v1', 'test', ARRAY['doc1', 'doc2'], '{"return_documents": false, "top_k": 10}'::JSONB);
```

## Supported ranking models

We support the following ranking models:

* `mixedbread-ai/mxbai-rerank-base-v1`
