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

Ranking documents is as simple as calling the the function with the documents you want to rank, and text you want to rank against:

```postgresql
SELECT pgml.rank('mixedbread-ai/mxbai-rerank-base-v1', 'test', ARRAY['doc1', 'doc2']);
```

By default the `pgml.rank()` function will return and rank all of the documents. The function can be configured to only return the relevance score and index of the top k documents by setting `return_documents` to `false` and `top_k` to the number of documents you want returned.

```postgresql
SELECT pgml.rank('mixedbread-ai/mxbai-rerank-base-v1', 'test', ARRAY['doc1', 'doc2'], '{"return_documents": false, "top_k": 10}'::JSONB);
```

## Supported ranking models

We currently support cross-encoders for re-ranking. Check out [Sentence Transformer's documentation](https://sbert.net/examples/applications/cross-encoder/README.html) for more information on how cross-encoders work.

By default we provide the following ranking models:

* `mixedbread-ai/mxbai-rerank-base-v1`
