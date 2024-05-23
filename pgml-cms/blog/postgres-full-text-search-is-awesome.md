---
description: >-
  If you want to improve your search results, don't rely on expensive O(n*m)
  word frequency statistics. Get new sources of data instead.
image: ".gitbook/assets/image (53).png"
---

# Postgres Full Text Search is Awesome!

<div align="left">

<figure><img src=".gitbook/assets/montana.jpg" alt="Author" width="125"><figcaption></figcaption></figure>

</div>

Montana Low

August 31, 2022

Normalized data is a powerful tool leveraged by 10x engineering organizations. If you haven't read [Postgres Full Text Search is Good Enough!](http://rachbelaid.com/postgres-full-text-search-is-good-enough/) you should, unless you're willing to take that statement at face value, without the code samples to prove it. We'll go beyond that claim in this post, but to reiterate the main points, Postgres supports:

* Stemming
* Ranking / Boost
* Multiple languages
* Fuzzy search for misspelling
* Accent support

This is good enough for most of the use cases out there, without introducing any additional concerns to your application. But, if you've ever tried to deliver relevant search results at scale, you'll realize that you need a lot more than these fundamentals. ElasticSearch has all kinds of best in class features, like a modified version of BM25 that is state of the art (developed in the 1970's), which is one of the many features you need beyond the Term Frequency (TF) based ranking that Postgres uses... but, _the ElasticSearch approach is a dead end_ for 2 reasons:

1. Trying to improve search relevance with statistics like TF-IDF and BM25 is like trying to make a flying car. What you want is a helicopter instead.
2. Computing Inverse Document Frequency (IDF) for BM25 brutalizes your search indexing performance, which leads to a [host of follow on issues via distributed computation](https://en.wikipedia.org/wiki/Fallacies\_of\_distributed\_computing), for the originally dubious reason.

<figure><img src=".gitbook/assets/image (53).png" alt=""><figcaption><p>What we were promised</p></figcaption></figure>

Academics have spent decades inventing many algorithms that use orders of magnitude more compute eking out marginally better results that often aren't worth it in practice. Not to generally disparage academia, their work has consistently improved our world, but we need to pay attention to tradeoffs. SQL is another acronym similarly pioneered in the 1970's. One difference between SQL and BM25 is that everyone has heard of the former before reading this blog post, for good reason.

If you actually want to meaningfully improve search results, you generally need to add new data sources. Relevance is much more often revealed by the way other things _**relate**_ to the document, rather than the content of the document itself. Google proved the point 23 years ago. Pagerank doesn't rely on the page content itself as much as it uses metadata from _links to the pages_. We live in a connected world and it's the interplay among things that reveal their relevance, whether that is links for websites, sales for products, shares for social posts... It's the greater context around the document that matters.

> _If you want to improve your search results, don't rely on expensive O(n\*m) word frequency statistics. Get new sources of data instead. It's the relational nature of relevance that underpins why a relational database forms the ideal search engine._

Postgres made the right call to avoid the costs required to compute Inverse Document Frequency in their search indexing, given its meager benefit. Instead, it offers the most feature-complete relational data platform. [Elasticsearch will tell you](https://www.elastic.co/guide/en/elasticsearch/reference/current/joining-queries.html), that you can't join data in a _**naively distributed system**_ at read time, because it is prohibitively expensive. Instead you'll have to join the data eagerly at indexing time, which is even more prohibitively expensive. That's good for their business since you're the one paying for it, and it will scale until you're bankrupt.

What you really should do, is leave the data normalized inside Postgres, which will allow you to join additional, related data at query time. It will take multiple orders of magnitude less compute to index and search a normalized corpus, meaning you'll have a lot longer (potentially forever) before you need to distribute your workload, and then maybe you can do that intelligently instead of naively. Instead of spending your time building and maintaining pipelines to shuffle updates between systems, you can work on new sources of data to really improve relevance.

With PostgresML, you can now skip straight to full on machine learning when you have the related data. You can load your feature store into the same database as your search corpus. Each data source can live in its own independent table, with its own update cadence, rather than having to reindex and denormalize entire documents back to ElasticSearch, or worse, large portions of the entire corpus, when a single thing changes.

With a single SQL query, you can do multiple passes of re-ranking, pruning and personalization to refine a search relevance score.

* basic term relevance
* embedding similarities
* XGBoost or LightGBM inference

These queries can execute in milliseconds on large production-sized corpora with Postgres's multiple indexing strategies. You can do all of this without adding any new infrastructure to your stack.

The following full blown example is for demonstration purposes only of a 3rd generation search engine. You can test it for real in the PostgresML Gym to build up a complete understanding.

```postgresql
WITH query AS (
  -- construct a query context with arguments that would typically be
  -- passed in from the application layer
  SELECT
    -- a keyword query for "my" OR "search" OR "terms"
    tsquery('my | search | terms') AS keywords,
    -- a user_id for personalization later on
    123456 AS user_id
),
first_pass AS (
  SELECT *,
    -- calculate the term frequency of keywords in the document
    ts_rank(documents.full_text, keywords) AS term_frequency
  -- our basic corpus is stored in the documents table
  FROM documents
  -- that match the query keywords defined above
  WHERE documents.full_text @@ query.keywords
  -- ranked by term frequency
  ORDER BY term_frequency DESC
  -- prune to a reasonably large candidate population
  LIMIT 10000
),
second_pass AS (
  SELECT *,
    -- create a second pass score of cosine_similarity across embeddings
    pgml.cosine_similarity(document_embeddings.vector, user_embeddings.vector) AS similarity_score
  FROM first_pass
  -- grab more data from outside the documents
  JOIN document_embeddings ON document_embeddings.document_id = documents.id
  JOIN user_embeddings ON user_embeddings.user_id = query.user_id
  -- of course we be re-ranking
  ORDER BY similarity_score DESC
  -- further prune results to top performers for more expensive ranking
  LIMIT 1000
),
third_pass AS (
  SELECT *,
    -- create a final score using xgboost
    pgml.predict('search relevance model', ARRAY[session_level_features.*]) AS final_score
  FROM second_pass
  JOIN session_level_features ON session_level_features.user_id = query.user_id
)
SELECT *
FROM third_pass
ORDER BY final_score DESC
LIMIT 100;
```

If you'd like to play through an interactive notebook to generate models for search relevance in a Postgres database, try it in the Gym. An exercise for the curious reader, would be to combine all three scores above into a single algebraic function for ranking, and then into a fourth learned model...

Many thanks and ❤️ to all those who are supporting this endeavor. We’d love to hear feedback from the broader ML and Engineering community about applications and other real world scenarios to help prioritize our work.
