---
author: Silas Marvin
description: HNSW indexing is the latest upgrade in vector recall performance. In this post we announce our updated SDK that utilizes HNSW indexing to give world class performance in vector search.
image: https://postgresml.org/dashboard/static/images/blog/elephant_sky.jpg
image_alt: PostgresML is a composition engine that provides advanced AI capabilities.
---

# Announcing HNSW Support in Our SDK

<div class="d-flex align-items-center mb-4">
  <img width="54px" height="54px" src="/dashboard/static/images/team/silas.jpg" style="border-radius: 50%;" alt="Author" />
  <div class="ps-3 d-flex justify-content-center flex-column">
    <p class="m-0">Silas Marvin</p>
    <p class="m-0">September 6, 2023</p>
  </div>
</div>

PostgresML makes it easy to use machine learning with your database and to scale workloads horizontally in our cloud. Our SDK makes it even easier.

## Introducing HNSW

Underneath the hood our SDK utilizes [pgvector](https://github.com/pgvector/pgvector) to store, index, and recall vectors. Up until this point our SDk used IVFFlat indexing to divide vectors into lists, search a subset of those lists, and return the closest vector matches.

While this is fast, it is not as fast as HNSW. Thanks to the latest update of [pgvector](https://github.com/pgvector/pgvector) our SDK now utilizes HNSW indexing, creating multi-layer graphs instead of lists and removing the required training step IVFFlat imposed.

The results are not disappointing.

## Comparing HNSW and IVFFlat

In one of our previous posts: [Tuning vector recall while generating query embeddings in the database](/blog/tuning-vector-recall-while-generating-query-embeddings-in-the-database) we were working on a datasets with over 5 million Amazon Movie Reviews, and after embedding the reviews, performed semantic similarity search to get the closest 5 reviews.

Here is the sql query we ran:

!!! generic

!!! code_block time="152.037 ms"

```postgresql
WITH request AS (
  SELECT pgml.embed(
    'intfloat/e5-large',
    'query: Best 1980''s scifi movie'
  )::vector(1024) AS embedding
)

SELECT
  review_body,
  product_title,
  star_rating,
  total_votes,
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

| review_body                                         | product_title                                                 | star_rating | total_votes | cosine_similarity  |
| --------------------------------------------------- | ------------------------------------------------------------- | ----------- | ----------- | ------------------ |
| best 80s SciFi movie ever                           | The Adventures of Buckaroo Banzai Across the Eighth Dimension | 5           | 1           | 0.956207707312679  |
| One of the best 80's sci-fi movies, beyond a doubt! | Close Encounters of the Third Kind [Blu-ray]                  | 5           | 1           | 0.9298004258989776 |
| One of the Better 80's Sci-Fi,                      | Krull (Special Edition)                                       | 3           | 5           | 0.9126601222760491 |
| the best of 80s sci fi horror!                      | The Blob                                                      | 5           | 2           | 0.9095577631102708 |
| Three of the best sci-fi movies of the seventies    | Sci-Fi: Triple Feature (BD) [Blu-ray]                         | 5           | 0           | 0.9024044582495285 |

!!!

!!!

This query utilized IVFFlat indexing and queried through over 5 million rows in 152 milliseconds. Pretty fast!

Let drop our IVFFlat index and create an HNSW index.

!!! generic

!!! code_block time="53236.884 ms (00:53.237)"

```postgresql
DROP INDEX index_amazon_us_reviews_on_review_embedding_e5_large;
CREATE INDEX CONCURRENTLY ON pgml.amazon_us_reviews USING hnsw (review_embedding_e5_large vector_cosine_ops);
```

!!!

!!! results

|CREATE INDEX|
|------------|

!!!

!!!

