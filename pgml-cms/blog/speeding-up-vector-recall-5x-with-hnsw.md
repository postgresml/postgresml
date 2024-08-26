---
description: >-
  HNSW indexing is the latest upgrade in vector recall performance. In this post
  we announce our updated SDK that utilizes HNSW indexing to give world class
  performance in vector search.
tags: [engineering]
featured: false
image: ".gitbook/assets/blog_image_hnsw.png"
---

# Speeding up vector recall 5x with HNSW

<div align="left">

<figure><img src=".gitbook/assets/silas.jpg" alt="Author" width="128"><figcaption></figcaption></figure>

</div>

Silas Marvin

October 2, 2023

PostgresML makes it easy to use machine learning with your database and to scale workloads horizontally in our cloud. Our SDK makes it even easier.

<figure><img src=".gitbook/assets/image (3).png" alt=""><figcaption><p>HNSW (hierarchical navigable small worlds) is an indexing method that greatly improves vector recall</p></figcaption></figure>

## Introducing HNSW

Underneath the hood our SDK utilizes [pgvector](https://github.com/pgvector/pgvector) to store, index, and recall vectors. Up until this point our SDK used IVFFlat indexing to divide vectors into lists, search a subset of those lists, and return the closest vector matches.

While the IVFFlat indexing method is fast, it is not as fast as HNSW. Thanks to the latest update of [pgvector](https://github.com/pgvector/pgvector) our SDK now utilizes HNSW indexing, creating multi-layer graphs instead of lists and removing the required training step IVFFlat imposed.

The results are not disappointing.

## Comparing HNSW and IVFFlat

In one of our previous posts: Tuning vector recall while generating query embeddings in the database we were working on a dataset with over 5 million Amazon Movie Reviews, and after embedding the reviews, performed semantic similarity search to get the closest 5 reviews.

Let's run that query again:

!!! generic

!!! code\_block time="89.118 ms"

```postgresql
WITH request AS (
  SELECT pgml.embed(
    'Alibaba-NLP/gte-base-en-v1.5',
    'query: Best 1980''s scifi movie'
  )::vector(1024) AS embedding
)

SELECT
  id,
  1 - (
    review_embedding_e5_large <=> (
      SELECT embedding FROM request
    )
  ) AS cosine_similarity
FROM pgml.amazon_us_reviews
ORDER BY review_embedding_e5_large <=> (SELECT embedding FROM request)
LIMIT 5;
```

!!!

!!! results

| review\_body                                     | product\_title                                                | star\_rating | total\_votes | cosine\_similarity |
| ------------------------------------------------ | ------------------------------------------------------------- | ------------ | ------------ | ------------------ |
| best 80s SciFi movie ever                        | The Adventures of Buckaroo Banzai Across the Eighth Dimension | 5            | 1            | 0.9495371273162286 |
| the best of 80s sci fi horror!                   | The Blob                                                      | 5            | 2            | 0.9097434758143605 |
| Three of the best sci-fi movies of the seventies | Sci-Fi: Triple Feature (BD) \[Blu-ray]                        | 5            | 0            | 0.9008723412875651 |
| best sci fi movie ever                           | The Day the Earth Stood Still (Special Edition) \[Blu-ray]    | 5            | 2            | 0.8943620968858654 |
| Great Science Fiction movie                      | Bloodsport / Timecop (Action Double Feature) \[Blu-ray]       | 5            | 0            | 0.894282454374093  |

!!!

!!!

This query utilized IVFFlat indexing and queried through over 5 million rows in 89.118ms. Pretty fast!

Let's drop our IVFFlat index and create an HNSW index.

!!! code\_block time="10255099.233 ms (02:50:55.099)"

```postgresql
DROP INDEX index_amazon_us_reviews_on_review_embedding_e5_large;
CREATE INDEX CONCURRENTLY ON pgml.amazon_us_reviews USING hnsw (review_embedding_e5_large vector_cosine_ops);
```

!!!

Now let's try the query again utilizing the new HNSW index we created.

!!! generic

!!! code\_block time="17.465 ms"

```postgresql
WITH request AS (
  SELECT pgml.embed(
    'Alibaba-NLP/gte-base-en-v1.5',
    'query: Best 1980''s scifi movie'
  )::vector(1024) AS embedding
)

SELECT
  id,
  1 - (
    review_embedding_e5_large <=> (
      SELECT embedding FROM request
    )
  ) AS cosine_similarity
FROM pgml.amazon_us_reviews
ORDER BY review_embedding_e5_large <=> (SELECT embedding FROM request)
LIMIT 5;
```

!!!

!!! results

| review\_body                   | product\_title                                                | star\_rating | total\_votes | cosine\_similarity |
| ------------------------------ | ------------------------------------------------------------- | ------------ | ------------ | ------------------ |
| best 80s SciFi movie ever      | The Adventures of Buckaroo Banzai Across the Eighth Dimension | 5            | 1            | 0.9495371273162286 |
| the best of 80s sci fi horror! | The Blob                                                      | 5            | 2            | 0.9097434758143605 |
| One of the Better 80's Sci-Fi  | Krull (Special Edition)                                       | 3            | 5            | 0.9093884940741694 |
| Good 1980s movie               | Can't Buy Me Love                                             | 4            | 0            | 0.9090294438721961 |
| great 80's movie               | How I Got Into College                                        | 5            | 0            | 0.9016508795301296 |

!!!

!!!

Not only are the results better (the `cosine_similarity` is higher overall), but HNSW is over 5x faster, reducing our search and embedding time to 17.465ms.

This is a massive upgrade to the recall speed utilized by our SDK and greatly improves overall performance.

For a deeper dive into HNSW checkout [Jonathan Katz's excellent article on HNSW in pgvector](https://jkatz05.com/post/postgres/pgvector-hnsw-performance/).
