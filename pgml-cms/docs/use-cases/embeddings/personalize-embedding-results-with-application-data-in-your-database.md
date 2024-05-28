# Personalize embedding results with application data in your database

PostgresML makes it easy to generate embeddings using open source models from Huggingface and perform complex queries with vector indexes and application data unlike any other database. The full expressive power of SQL as a query language is available to seamlessly combine semantic, geospatial, and full text search, along with filtering, boosting, aggregation, and ML reranking in low latency use cases. You can do all of this faster, simpler and with higher quality compared to applications built on disjoint APIs like OpenAI + Pinecone. Prove the results in this series to your own satisfaction, for free, by signing up for a GPU accelerated database.

## Introduction

This article is the third in a multipart series that will show you how to build a post-modern semantic search and recommendation engine, including personalization, using open source models. You may want to start with the previous articles in the series if you aren't familiar with PostgresML's capabilities.

1. Generating LLM Embeddings with HuggingFace models
2. Tuning vector recall with pgvector
3. Personalizing embedding results with application data
4. Optimizing semantic results with an XGBoost ranking model - coming soon!


_Embeddings can be combined into personalized perspectives when stored as vectors in the database._

## Personalization

In the era of big data and advanced machine learning algorithms, personalization has become a critical component in many modern technologies. One application of personalization is in search and recommendation systems, where the goal is to provide users with relevant and personalized experiences. Embedding vectors have become a popular tool for achieving this goal, as they can represent items and users in a compact and meaningful way. However, standard embedding vectors have limitations, as they do not take into account the unique preferences and behaviors of individual users. To address this, a promising approach is to use aggregates of user data to personalize embedding vectors. This article will explore the concept of using aggregates to create new embedding vectors and provide a step-by-step guide to implementation.

We'll continue working with the same dataset from the previous articles. 5M+ customer reviews about movies from amazon over a decade. We've already generated embeddings for each review, and aggregated them to build a consensus view of the reviews for each movie. You'll recall that our reviews also include a customer\_id as well.

!!! generic

!!! code\_block

```postgresql
\d pgml.amazon_us_reviews
```

!!!

!!! results

| Column             | Type    | Collation | Nullable | Default |
| ------------------ | ------- | --------- | -------- | ------- |
| marketplace        | text    |           |          |         |
| customer\_id       | text    |           |          |         |
| review\_id         | text    |           |          |         |
| product\_id        | text    |           |          |         |
| product\_parent    | text    |           |          |         |
| product\_title     | text    |           |          |         |
| product\_category  | text    |           |          |         |
| star\_rating       | integer |           |          |         |
| helpful\_votes     | integer |           |          |         |
| total\_votes       | integer |           |          |         |
| vine               | bigint  |           |          |         |
| verified\_purchase | bigint  |           |          |         |
| review\_headline   | text    |           |          |         |
| review\_body       | text    |           |          |         |
| review\_date       | text    |           |          |         |

!!!

!!!

## Creating embeddings for customers

In the previous article, we saw that we could aggregate all the review embeddings to create a consensus view of each movie. Now we can take that a step further, and aggregate all the movie embeddings that each customer has reviewed, to create an embedding for every customer in terms of the movies they've reviewed. We're not going to worry about if they liked the movie or not just yet based on their star rating. Simply the fact that they've chosen to review a movie indicates they chose to purchase the DVD, and reveals something about their preferences. It's always easy to create more tables and indexes related to other tables in our database.

!!! generic

!!! code\_block time="458838.918 ms (07:38.839)"

```postgresql
CREATE TABLE customers AS
SELECT
  customer_id AS id,
  count(*) AS total_reviews,
  avg(star_rating) AS star_rating_avg,
  pgml.sum(movies.review_embedding_e5_large)::vector(1024) AS movie_embedding_e5_large
FROM pgml.amazon_us_reviews
JOIN movies
  ON movies.id = amazon_us_reviews.product_id
GROUP BY customer_id;
```

!!!

!!! results

SELECT 2075970

!!!

!!!

We've just created a table aggregating our 5M+ reviews into 2M+ customers, with mostly vanilla SQL. The query includes a JOIN between the `pgml.amazon_us_reviews` we started with, and the `movies` table we created to hold the movie embeddings. We're using `pgml.sum()` again, this time to sum up all the movies a customer has reviewed, to create an embedding for the customer. We will want to be able to quickly recall a customers embedding by their ID whenever they visit the site, so we'll create a standard Postgres index on their ID. This isn't just a vector database, it's a full AI application database.

!!! generic

!!! code\_block time="2709.506 ms (00:02.710)"

```postgresql
CREATE INDEX customers_id_idx ON customers (id);
```

!!!

!!! results

```
CREATE INDEX
```

!!!

!!!

Now we can incorporate a customer embedding to personalize the results whenever they search. Normally, we'd have the `customers.id` in our application already because they'd be searching and browsing our site, but we don't have an actual application or customers for this article, so we'll have to find one for our example. Let's find a customer that loves the movie Empire Strikes Back. No Star Wars made our original list, so we have a good opportunity to improve our previous results with personalization.

## Finding a customer to personalize results for

Now that we have customer embeddings around movies they've reviewed, we can incorporate those to personalize the results whenever they search. Normally, we'd have the `customers.id` handy in our application because they'd be searching and browsing our app, but we don't have an actual application or customers for this article, so we'll have to find one for our example. Let's find a customer that loves the movie "Empire Strikes Back". No "Star Wars" made our original list of "Best 1980's scifi movie", so we have a good opportunity to improve our previous results with personalization.

We can find a customer that our embeddings model feels is close to the sentiment "I love all Star Wars, but Empire Strikes Back is particularly amazing". Keep in mind, we didn't want to take the time to build a vector index for queries against the customers table, so this is going to be slower than it could be, but that's fine because it's just a one-off exploration, not some frequently executed query in our application. We can still do vector searches, just without the speed boost an index provides.

!!! generic

!!! code\_block time="9098.883 ms (00:09.099)"

```postgresql
WITH request AS (
  SELECT pgml.embed(
    'Alibaba-NLP/gte-base-en-v1.5',
    'query: I love all Star Wars, but Empire Strikes Back is particularly amazing'
  )::vector(1024) AS embedding
)

SELECT
  id,
  total_reviews,
  star_rating_avg,
  1 - (
    movie_embedding_e5_large <=> (SELECT embedding FROM request)
  ) AS cosine_similarity
FROM customers
ORDER BY cosine_similarity DESC
LIMIT 1;
```

!!!

!!! results

| id       | total\_reviews | star\_rating\_avg  | cosine\_similarity |
| -------- | -------------- | ------------------ | ------------------ |
| 44366773 | 1              | 2.0000000000000000 | 0.8831349398621555 |

!!!

!!!

!!! note

Searching without indexes is slower (9s), but creating a vector index can take a very long time (remember indexing all the reviews took more than an hour). For frequently executed application queries, we always want to make sure we have at least 1 index available to improve speed. Anyway, it turns out we have a customer with a very similar embedding to our desired personalization. Semantic search is wonderfully powerful. Once you've generated embeddings, you can find all the things that are similar to other things, even if they don't share any of the same words. Whether this customer has actually ever even seen Star Wars, the model thinks their embedding is pretty close to a review like that...

!!!

It turns out we have a customer with a very similar embedding to our desired personalization. Semantic search is wonderfully powerful. Once you've generated embeddings, you can find all the things that are similar to other things, even if they don't share any of the same words. Whether this customer has actually ever even seen Star Wars, the model thinks their embedding is pretty close to a review like that... They seem a little picky though with 2-star rating average. I'm curious what the 1 review they've actually written looks like:

!!! generic

!!! code\_block time="25156.945 ms (00:25.157)"

```postgresql
SELECT product_title, star_rating, review_body
FROM pgml.amazon_us_reviews
WHERE customer_id = '44366773';
```

!!!

!!! results

| product\_title                                                     | star\_rating | review\_body                                                                  |
| ------------------------------------------------------------------ | ------------ | ----------------------------------------------------------------------------- |
| Star Wars, Episode V: The Empire Strikes Back (Widescreen Edition) | 2            | The item was listed as new. The box was opened and had damage to the outside. |

!!!

!!!

This is odd at first glance. The review doesn't mention anything thing about Star Wars, and the sentiment is actually negative, even the `star_rating` is bad. How did they end up with an embedding so close to our desired sentiment of "I love all Star Wars, but Empire Strikes Back is particularly amazing"? Remember we didn't generate embeddings from their review text directly. We generated customer embeddings from the movies they had bothered to review. This customer has only ever reviewed 1 movie, and that happens to be the movie closest to our sentiment. Exactly what we were going for!

If someone only ever bothered to write 1 review, and they are upset about the physical DVD, it's likely they are a big fan of the movie, and they are upset about the physical DVD because they wanted to keep it for a long time. This is a great example of how stacking and relating embeddings carefully can generate insights at a scale that is otherwise impossible, revealing the signal in the noise.

Now we can write our personalized SQL query. It's nearly the same as our query from the previous article, but we're going to include an additional CTE to fetch the customers embedding by id, and then tweak our `final_score`. Here comes personalized query results, using that customer 44366773's embedding. Instead of the generic popularity boost we've been using, we'll calculate the cosine similarity of the customer embedding to all the movies in the results, and use that as a boost. This will push movies that are similar to the customer's embedding to the top of the results.

## Personalizing search results

Now we can write our personalized SQL query. It's nearly the same as our query from the previous article, but we're going to include an additional CTE to fetch the customers embedding by id, and then tweak our `final_score`. Instead of the generic popularity boost we've been using, we'll calculate the cosine similarity of the customer embedding to all the movies in the results, and use that as a boost. This will push movies that are similar to the customer's embedding to the top of the results. Here comes personalized query results, using that customer 44366773's embedding:

!!! generic

!!! code\_block time="127.639 ms (00:00.128)"

```postgresql
-- create a request embedding on the fly
WITH request AS (
  SELECT pgml.embed(
    'Alibaba-NLP/gte-base-en-v1.5',
    'query: Best 1980''s scifi movie'
  )::vector(1024) AS embedding
),

-- retrieve the customers embedding by id
customer AS (
  SELECT movie_embedding_e5_large AS embedding
  FROM customers
  WHERE id = '44366773'
),

-- vector similarity search for movies and calculate a customer_cosine_similarity at the same time
first_pass AS (
  SELECT
    title,
    total_reviews,
    star_rating_avg,
    1 - (
      review_embedding_e5_large <=> (SELECT embedding FROM request)
    ) AS request_cosine_similarity,
    (1 - (
      review_embedding_e5_large <=> (SELECT embedding FROM customer)
    ) - 0.9) * 10 AS  customer_cosine_similarity,
    star_rating_avg / 5 AS star_rating_score
  FROM movies
  WHERE total_reviews > 10
  ORDER BY review_embedding_e5_large <=> (SELECT embedding FROM request)
  LIMIT 1000
)

-- grab the top 10 results, re-ranked using a combination of request similarity and customer similarity
SELECT
  title,
  total_reviews,
  round(star_rating_avg, 2) as star_rating_avg,
  star_rating_score,
  request_cosine_similarity,
  customer_cosine_similarity,
  request_cosine_similarity + customer_cosine_similarity + star_rating_score AS final_score
FROM first_pass
ORDER BY final_score DESC
LIMIT 10;
```

!!!

!!! results

| title                                                                | total\_reviews | star\_rating\_avg | star\_rating\_score    | request\_cosine\_similarity | customer\_cosine\_similarity | final\_score       |
| -------------------------------------------------------------------- | -------------- | ----------------- | ---------------------- | --------------------------- | ---------------------------- | ------------------ |
| Star Wars, Episode V: The Empire Strikes Back (Widescreen Edition)   | 78             | 4.44              | 0.88717948717948718000 | 0.8295302273865711          | 0.9999999999999998           | 2.716709714566058  |
| Star Wars, Episode IV: A New Hope (Widescreen Edition)               | 80             | 4.36              | 0.87250000000000000000 | 0.8339361274771777          | 0.9336656923446551           | 2.640101819821833  |
| Forbidden Planet (Two-Disc 50th Anniversary Edition)                 | 255            | 4.82              | 0.96392156862745098000 | 0.8577616472530644          | 0.6676592605840725           | 2.489342476464588  |
| The Day the Earth Stood Still                                        | 589            | 4.76              | 0.95212224108658744000 | 0.8555529952535671          | 0.6733939449212423           | 2.4810691812613967 |
| Forbidden Planet \[Blu-ray]                                          | 223            | 4.79              | 0.95874439461883408000 | 0.8479982398847651          | 0.6536320269646467           | 2.4603746614682462 |
| John Carter (Four-Disc Combo: Blu-ray 3D/Blu-ray/DVD + Digital Copy) | 559            | 4.65              | 0.93059033989266548000 | 0.8338600628541288          | 0.6700415876545052           | 2.4344919904012996 |
| The Terminator                                                       | 430            | 4.59              | 0.91813953488372094000 | 0.8428833221752442          | 0.6638043064287047           | 2.4248271634876697 |
| The Day the Earth Stood Still (Two-Disc Special Edition)             | 37             | 4.57              | 0.91351351351351352000 | 0.8419118958433142          | 0.6636373066510914           | 2.419062716007919  |
| The Thing from Another World                                         | 501            | 4.71              | 0.94291417165668662000 | 0.8511107698234265          | 0.6231913893834695           | 2.4172163308635826 |
| The War of the Worlds (Special Collector's Edition)                  | 171            | 4.67              | 0.93333333333333334000 | 0.8460163011246516          | 0.6371641286728591           | 2.416513763130844  |

!!!

!!!

Bingo. Now we're boosting movies by `(customer_cosine_similarity - 0.9) * 10`, and we've kept our previous boost for movies with a high average star rating. Not only does Episode V top the list as expected, Episode IV is a close second. This query has gotten fairly complex! But the results are perfect for me, I mean our hypothetical customer who is searching for "Best 1980's scifi movie" but has already revealed to us with their one movie review that they think like the comment "I love all Star Wars, but Empire Strikes Back is particularly amazing". I promise I'm not just doing all of this to find a new movie to watch tonight.

You can compare this to our non-personalized results from the previous article for reference Forbidden Planet used to be the top result, but now it's #3.

!!! code\_block time="124.119 ms"

!!! results

| title                                                | total\_reviews | star\_rating\_avg |       final\_score |    star\_rating\_score | cosine\_similarity |
| ---------------------------------------------------- | -------------: | ----------------: | -----------------: | ---------------------: | -----------------: |
| Forbidden Planet (Two-Disc 50th Anniversary Edition) |            255 |              4.82 | 1.8216832158805154 | 0.96392156862745098000 | 0.8577616472530644 |
| Back to the Future                                   |             31 |              4.94 |   1.82090702765472 | 0.98709677419354838000 | 0.8338102534611714 |
| Warning Sign                                         |             17 |              4.82 | 1.8136734057737756 | 0.96470588235294118000 | 0.8489675234208343 |
| Plan 9 From Outer Space/Robot Monster                |             13 |              4.92 | 1.8126103400815046 | 0.98461538461538462000 | 0.8279949554661198 |
| Blade Runner: The Final Cut (BD) \[Blu-ray]          |             11 |              4.82 | 1.8120690455673043 | 0.96363636363636364000 | 0.8484326819309408 |
| The Day the Earth Stood Still                        |            589 |              4.76 | 1.8076752363401547 | 0.95212224108658744000 | 0.8555529952535671 |
| Forbidden Planet \[Blu-ray]                          |            223 |              4.79 | 1.8067426345035993 | 0.95874439461883408000 | 0.8479982398847651 |
| Aliens (Special Edition)                             |             25 |              4.76 |  1.803194119705901 | 0.95200000000000000000 |  0.851194119705901 |
| Night of the Comet                                   |             22 |              4.82 |  1.802469182369724 | 0.96363636363636364000 | 0.8388328187333605 |
| Forbidden Planet                                     |             19 |              4.68 |  1.795573710000297 | 0.93684210526315790000 | 0.8587316047371392 |

!!!

!!!

Big improvement! We're doing a lot now to achieve filtering, boosting, and personalized re-ranking, but you'll notice that this extra work only takes a couple more milliseconds in PostgresML. Remember in the previous article when took over 100ms to just retrieve 5 embedding vectors in no particular order. All this embedding magic is pretty much free when it's done inside the database. Imagine how slow a service would be if it had to load 1000 embedding vectors (not 5) like our similarity search is doing, and then passing those to some HTTP API where some ML black box lives, and then fetching a different customer embedding from a different database, and then trying to combine that with the thousand results from the first query... This is why machine learning microservices break down at scale, and it's what makes PostgresML one step ahead of less mature vector databases.

## What's next?

We've got personalized results now, but `(... - 0.9) * 10` is a bit of a hack I used to scale the personalization score to have a larger impact on the final score. Hacks and heuristics are frequently injected like this when a Product Manager tells an engineer to "just make it work", but oh no! Back To The Future is now nowhere to be found on my personalized list. We can do better! Those magic numbers are intended to optimize something our Product Manager is going for as a business metric. There's a way out of infinite customer complaints and one off hacks like this, and it's called machine learning.

Finding the optimal set of magic numbers that "just make it work" is what modern machine learning is all about from one point of view. In the next article, we'll look at building a real personalized ranking model using XGBoost on top of our personalized embeddings, that predicts how our customer will rate a movie on our 5-star review scale. Then we can rank results based on a much more sophisticated model's predicted star rating score instead of just using cosine similarity and made up numbers. With all the savings we're accruing in terms of latency and infrastructure simplicity, our ability to layer additional models, refinements and techniques will put us another step ahead of the alternatives.
