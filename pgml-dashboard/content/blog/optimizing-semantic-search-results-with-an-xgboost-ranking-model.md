---
author: Montana Low
description: How to personalize results from a vector database generated with open source HuggingFace models using pgvector and PostgresML.
image: https://postgresml.org/static/images/blog/models_1.jpg
image_alt: Embeddings can be combined into personalized perspectives when stored as vectors in the database.
---

# Optimizing semantic search results with an XGBoost model in your database

<div class="d-flex align-items-center mb-4">
  <img width="54px" height="54px" src="/dashboard/static/images/team/montana.jpg" style="border-radius: 50%;" alt="Author" />
  <div class="ps-3 d-flex justify-content-center flex-column">
    <p class="m-0">Montana Low</p>
    <p class="m-0">May 3, 2023</p>
  </div>
</div>

PostgresML makes it easy to generate embeddings using open source models from Huggingface and perform complex queries with vector indexes and application data unlike any other database. The full expressive power of SQL as a query language is available to seamlessly combine semantic, geospatial, and full text search, along with filtering, boosting, aggregation, and ML reranking in low latency use cases. You can do all of this faster, simpler and with higher quality compared to applications built on disjoint APIs like OpenAI | Pinecone. Prove the results in this series to your own satisfaction, for free, by [signing up](<%- crate::utils::config::signup_url() %>) for a GPU accelerated database.

## Introduction

This article is the fourth in a multipart series that will show you how to build a post-modern semantic search and recommendation engine, including personalization, using open source models. You may want to start with the previous articles in the series if you aren't familiar with PostgresML's capabilities.

1) [Generating LLM Embeddings with HuggingFace models](/blog/generating-llm-embeddings-with-open-source-models-in-postgresml)
2) [Tuning vector recall with pgvector](/blog/tuning-vector-recall-while-generating-query-embeddings-in-the-database)
3) [Personalizing embedding results with application data](/blog/personalize-embedding-vector-search-results-with-huggingface-and-pgvector)
4) [Optimizing semantic search results with an XGBoost model](/blog/optimizing-semantic-search-results-with-an-xgboost-model)

<img src="/dashboard/static/images/blog/models_1.webp" alt="Models allow us to predict the future." />
<center><p><i>Models can be trained on application data, to reach an objective.</i></p></center>

## Custom Ranking Models

In the previous article, we showed how to personalize results from a vector database generated with open source HuggingFace models using pgvector and PostgresML. In the end though, we need to combine multiple scores together, semantic relevance (cosine similarity of the request embedding), personalization (cosine similarity of the customer embedding) and the movies average star rating into a single final score. This is a common technique used in production search engines, and is called reranking. I made up some numbers to scale the personalization score so that it didn't completely dominate the relevance score, but often times, making up weights like that for one query, makes other queries worse. Balancing, and finding the optimal weights for multiple scores is a hard problem, and is best solved with a machine learning model using real world user data as the final arbiter.

A Machine Learning model is just a computer program or mathematical function that takes inputs, and produces an output. Generally speaking, PostgresML can train two types of classical Machine Learning models, "regression" or "classification". These are closely related, but the difference it that the outputs for classification models produce discrete outputs, like booleans, or enums, and the outputs for regression models are continuous, i.e. floating point numbers. In our movie ranking example, we could train a classification model that would try to predict our movie score as 1 of 5 different star classes, where each star level is discrete, but it would lump all 4-star movies together, and all 5-star movies together, which wouldn't allow us to show subtle between say a 4.1 star and 4.8 star movie when ranking search results. Regression models predict a floating point number, aka a continuous variable, and since star ratings can be thought of on a continuous scale rather than discrete classes with no order relating each other, we'll use a regression model to predict the final score for our search results.

In our case, the inputs we have available are the same as the inputs to our final score (user and movie data), and the output we want is a prediction of how much this user will like this movie on a scale of 0-5. There are many different algorithm's available to train models. The simplest algorithm, would be to always predict the middle value of 2.5 stars. I mean, that's a terrible model, but it's pretty simple, we didn't even have to look at any data at all0. Slightly better would be to find the average star rating of all movies, and just predict that every time. Still simple, but it doesn't differentiate between movies take into consideration any inputs. A step further might predict the average star rating for each movie... At least we'd take the movie id as an input now, and predict differe

Models are training on historical data, like our table of movie reviews with star rankings. The simplest model we could build, would always predict the average star rating of all movies, which we can "learn" from the data, but this model doesn't take any inputs into consideration about a particular movie or customer. Fast, not very good, but not the .



, The model is trained on historical data, where we know the correct answer, the final score that the customer gave the movie. The model learns to predict the correct answer, by minimizing the error between the predicted score, and the actual score. Once the model is trained, we can use it to predict the final score for new movies, and new customers, that it has never seen before. This is called inference, and is the same process that we used to generate the embeddings in the first place.



The inputs to our
the type of models we're interested in building require example input data that produced some recorded outcome. For instance, the outcome of a user selecting and then watching a movie was them creating a `star_rating` for the review. This type of learning is referred to as Supervised Learning, because the customer is acting as a supervisor for the model, and "labelling" their own metadata | the movies metadata = star rating, effectively giving it the correct answer for millions of examples. A good model will be able to generalize from those examples, to pairs of customers and movies that it has never seen before, and predict the star rating that the customer would give the movie.

### Creating a View of the Training Data
PostgresML includes dozens of different algorithms that can be effective at learning from examples, and making predictions. Linear Regression is a relatively fast and mathematically straightforward algorithm, that we can use as our first model to establish a baseline for latency and quality. The first step is to create a `VIEW` of our example data for the model.

```postgresql
CREATE VIEW reviews_for_model AS
SELECT
  star_rating::FLOAT4,
  (1 - (customers.movie_embedding_e5_large <=> movies.review_embedding_e5_large) )::FLOAT4 AS cosine_similarity,
  movies.total_reviews::FLOAT4 AS movie_total_reviews,
  movies.star_rating_avg::FLOAT4 AS movie_star_rating_avg,
  customers.total_reviews::FLOAT4 AS customer_total_reviews,
  customers.star_rating_avg::FLOAT4 AS customer_star_rating_avg
FROM pgml.amazon_us_reviews
JOIN customers ON customers.id = amazon_us_reviews.customer_id
JOIN movies ON movies.id = amazon_us_reviews.product_id
WHERE star_rating IS NOT NULL
LIMIT 10
;
```
!!! results "46.855 ms"
```
CREATE VIEW
```
!!!

We're gathering our outcome along with the input features across 3 tables into a single view. Let's take a look at a few example rows:

```postgresql
SELECT *
FROM reviews_for_model
LIMIT 2;
```

!!! results "54.842 ms"

| star_rating | cosine_similarity  | movie_total_reviews | movie_star_rating_avg | customer_total_reviews | customer_star_rating_avg |
|-------------|--------------------|---------------------|-----------------------|------------------------|--------------------------|
| 4           | 0.9934197225949364 | 425                 | 4.6635294117647059    | 13                     | 4.5384615384615385       |
| 5           | 0.9997079926962424 | 425                 | 4.6635294117647059    | 2                      | 5.0000000000000000       |

!!!

### Training a Model
And now we can train a model. We're starting with linear regression, since it's fairly fast and straightforward.

```postgresql
SELECT * FROM pgml.train(
  project_name => 'our reviews model',
  task => 'regression',
  relation_name => 'reviews_for_model',
  y_column_name => 'star_rating',
  algorithm => 'linear'
);
```

!!! results "85416.566 ms (01:25.417)"
```
INFO:  Snapshotting table "reviews_for_model", this may take a little while...
INFO:  Dataset { num_features: 5, num_labels: 1, num_distinct_labels: 0, num_rows: 5134517, num_train_rows: 3850888, num_test_rows: 1283629 }
INFO:  Column "star_rating": Statistics { min: 1.0, max: 5.0, max_abs: 5.0, mean: 4.3076715, median: 5.0, mode: 5.0, variance: 1.3873447, std_dev: 1.177856, missing: 0, distinct: 5, histogram: [248745, 0, 0, 0, 0, 158934, 0, 0, 0, 0, 290411, 0, 0, 0, 0, 613476, 0, 0, 0, 2539322], ventiles: [1.0, 2.0, 3.0, 4.0, 4.0, 4.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0], categories: None }
INFO:  Column "cosine_similarity": Statistics { min: 0.73038024, max: 1.0, max_abs: 1.0, mean: 0.98407245, median: 0.9864355, mode: 1.0, variance: 0.00076778734, std_dev: 0.027708976, missing: 0, distinct: 1065916, histogram: [139, 55, 179, 653, 1344, 2122, 3961, 8381, 11891, 15454, 17234, 21213, 24762, 38839, 67734, 125466, 247090, 508321, 836051, 1919999], ventiles: [0.9291469, 0.94938564, 0.95920646, 0.9656065, 0.97034097, 0.97417694, 0.9775266, 0.9805849, 0.98350716, 0.9864354, 0.98951995, 0.9930062, 0.99676734, 0.99948853, 1.0, 1.0, 1.0, 1.0, 1.0], categories: None }
INFO:  Column "movie_total_reviews": Statistics { min: 1.0, max: 4969.0, max_abs: 4969.0, mean: 226.21008, median: 84.0, mode: 1.0, variance: 231645.1, std_dev: 481.29523, missing: 0, distinct: 834, histogram: [2973284, 462646, 170076, 81199, 56737, 33804, 14253, 14832, 6293, 4729, 0, 0, 2989, 3414, 3641, 0, 4207, 8848, 0, 9936], ventiles: [3.0, 7.0, 12.0, 18.0, 25.0, 34.0, 44.0, 55.0, 69.0, 84.0, 101.0, 124.0, 150.0, 184.0, 226.0, 283.0, 370.0, 523.0, 884.0], categories: None }
INFO:  Column "movie_star_rating_avg": Statistics { min: 1.0, max: 5.0, max_abs: 5.0, mean: 4.430256, median: 4.4761906, mode: 5.0, variance: 0.34566483, std_dev: 0.58793265, missing: 0, distinct: 9058, histogram: [12889, 1385, 6882, 3758, 3904, 15136, 12148, 16419, 24421, 23666, 71070, 84890, 126533, 155995, 212073, 387150, 511706, 769109, 951284, 460470], ventiles: [3.2, 3.5789473, 3.8135593, 3.9956522, 4.090909, 4.1969695, 4.277202, 4.352941, 4.4166665, 4.4761906, 4.5234375, 4.571429, 4.6164384, 4.6568627, 4.6944447, 4.734375, 4.773006, 4.818182, 4.9], categories: None }
INFO:  Column "customer_total_reviews": Statistics { min: 1.0, max: 3588.0, max_abs: 3588.0, mean: 63.472603, median: 4.0, mode: 1.0, variance: 67485.94, std_dev: 259.78055, missing: 0, distinct: 561, histogram: [3602754, 93036, 42129, 26392, 17871, 16154, 9864, 8125, 5465, 9093, 0, 1632, 1711, 1819, 7795, 2065, 2273, 0, 0, 2710], ventiles: [1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 5.0, 7.0, 9.0, 13.0, 19.0, 29.0, 48.0, 93.0, 268.0], categories: None }
INFO:  Column "customer_star_rating_avg": Statistics { min: 1.0, max: 5.0, max_abs: 5.0, mean: 4.3082585, median: 4.6666665, mode: 5.0, variance: 0.8520067, std_dev: 0.92304206, missing: 0, distinct: 4911, histogram: [109606, 2313, 6148, 4254, 3472, 57468, 16056, 24706, 30530, 23478, 158010, 78288, 126053, 144905, 126600, 417290, 232601, 307764, 253474, 1727872], ventiles: [2.3333333, 3.0, 3.5, 3.7777777, 4.0, 4.0, 4.2, 4.375, 4.5, 4.6666665, 4.7887325, 4.95, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0], categories: None }
INFO:  Training Model { id: 1, task: regression, algorithm: linear, runtime: rust }
INFO:  Hyperparameter searches: 1, cross validation folds: 1
INFO:  Hyperparams: {}
INFO:  Metrics: {"r2": 0.64389575, "mean_absolute_error": 0.4502707, "mean_squared_error": 0.50657624, "fit_time": 0.23825137, "score_time": 0.015739812}
INFO:  Deploying model id: 1
```

| project           | task       | algorithm | deployed |
|-------------------|------------|-----------|----------|
| our reviews model | regression | linear    | t        |

!!!

PostgresML just did a fair bit of work in a couple of minutes. We'll go through the steps in detail below, but here's a quick summary:
1) It scanned our 5134517, and split it into training and testing data
2) It did a quick analysis of each column in the data, to calculate some statistics we can view later
3) It trained a linear regression model on the training data
4) It evaluated the model on the testing data, and recorded the key metrics. In this case, the R2 score was 0.64, which is not bad for a first pass
5) Since the model passed evaluation, it was deployed for use

Regression models use R<sup>2</sup> as a measure of how well the model fits the data. The value ranges from 0 to 1, with 1 being a perfect fit. The value of 0.64 means that the model explains 64% of the variance in the data. You could input This is a good start, but we can do better.

### Inspect the models predictions

We can run a quick check on the model with our training data:

```sql
SELECT
  star_rating,
  pgml.predict(
    project_name => 'our reviews model',
    features => ARRAY[
      cosine_similarity,
      movie_total_reviews,
      movie_star_rating_avg,
      customer_total_reviews,
      customer_star_rating_avg
    ]
  ) AS prediction
FROM reviews_for_model
LIMIT 10;
```

!!! results "39.498 ms"

| star_rating | predict   |
|-------------|-----------|
| 5           | 4.8204975 |
| 5           | 5.1297455 |
| 5           | 5.0331154 |
| 5           | 4.466692  |
| 5           | 5.062803  |
| 5           | 5.1485577 |
| 1           | 3.3430705 |
| 5           | 5.055003  |
| 4           | 2.2641056 |
| 5           | 4.512218  |

!!!

This simple model has learned that we have a lot of 5-star ratings. If you scroll up to the original output, the analysis measured the star_rating has a mean of 4.3. The simplest model we could make, would be to just guess the average of 4.3 every time, or the mode of 5 every time. This model is doing a little better than that. It did lower its guesses for the 2 non 5 star examples we check, but not much. We'll skip 30 years of research and development, and jump straight to a more advanced algorithm.

### XGBoost

XGBoost is a popular algorithm for tabular data. It's a tree-based algorithm, which means it's a little more complex than linear regression, but it can learn more complex patterns in the data. We'll train an XGBoost model on the same training data, and see if it can do better.

```sql
SELECT * FROM pgml.train(
  project_name => 'our reviews model',
  task => 'regression',
  relation_name => 'reviews_for_model',
  y_column_name => 'star_rating',
  algorithm => 'xgboost'
);
```

!!! results "98830.704 ms (01:38.831)"

```
INFO:  Snapshotting table "reviews_for_model", this may take a little while...
INFO:  Dataset { num_features: 5, num_labels: 1, num_distinct_labels: 0, num_rows: 5134517, num_train_rows: 3850888, num_test_rows: 1283629 }
INFO:  Column "star_rating": Statistics { min: 1.0, max: 5.0, max_abs: 5.0, mean: 4.30768, median: 5.0, mode: 5.0, variance: 1.3873348, std_dev: 1.1778518, missing: 0, distinct: 5, histogram: [248741, 0, 0, 0, 0, 158931, 0, 0, 0, 0, 290417, 0, 0, 0, 0, 613455, 0, 0, 0, 2539344], ventiles: [1.0, 2.0, 3.0, 4.0, 4.0, 4.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0], categories: None }
INFO:  Column "cosine_similarity": Statistics { min: 0.73038024, max: 1.0, max_abs: 1.0, mean: 0.98407227, median: 0.98643565, mode: 1.0, variance: 0.0007678081, std_dev: 0.02770935, missing: 0, distinct: 1065927, histogram: [139, 55, 179, 653, 1344, 2122, 3960, 8382, 11893, 15455, 17235, 21212, 24764, 38840, 67740, 125468, 247086, 508314, 836036, 1920011], ventiles: [0.92914546, 0.9493847, 0.9592061, 0.9656064, 0.97034085, 0.97417694, 0.9775268, 0.98058504, 0.9835075, 0.98643565, 0.98952013, 0.99300617, 0.9967673, 0.99948853, 1.0, 1.0, 1.0, 1.0, 1.0], categories: None }
INFO:  Column "movie_total_reviews": Statistics { min: 1.0, max: 4969.0, max_abs: 4969.0, mean: 226.21071, median: 84.0, mode: 1.0, variance: 231646.2, std_dev: 481.2964, missing: 0, distinct: 834, histogram: [2973282, 462640, 170079, 81203, 56738, 33804, 14253, 14832, 6293, 4729, 0, 0, 2989, 3414, 3641, 0, 4207, 8848, 0, 9936], ventiles: [3.0, 7.0, 12.0, 18.0, 25.0, 34.0, 44.0, 55.0, 69.0, 84.0, 101.0, 124.0, 150.0, 184.0, 226.0, 283.0, 370.0, 523.0, 884.0], categories: None }
INFO:  Column "movie_star_rating_avg": Statistics { min: 1.0, max: 5.0, max_abs: 5.0, mean: 4.430269, median: 4.4761906, mode: 5.0, variance: 0.34565005, std_dev: 0.5879201, missing: 0, distinct: 9058, histogram: [12888, 1385, 6882, 3756, 3903, 15133, 12146, 16423, 24417, 23664, 71072, 84889, 126526, 155994, 212070, 387127, 511706, 769112, 951295, 460500], ventiles: [3.2, 3.5789473, 3.8135593, 3.9956522, 4.090909, 4.1969695, 4.277228, 4.352941, 4.4166665, 4.4761906, 4.5234375, 4.571429, 4.6164384, 4.6568627, 4.6944447, 4.73444, 4.773006, 4.818182, 4.9], categories: None }
INFO:  Column "customer_total_reviews": Statistics { min: 1.0, max: 3588.0, max_abs: 3588.0, mean: 63.47199, median: 4.0, mode: 1.0, variance: 67485.87, std_dev: 259.78043, missing: 0, distinct: 561, histogram: [3602758, 93032, 42129, 26392, 17871, 16154, 9864, 8125, 5465, 9093, 0, 1632, 1711, 1819, 7795, 2065, 2273, 0, 0, 2710], ventiles: [1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 5.0, 7.0, 9.0, 13.0, 19.0, 29.0, 48.0, 93.0, 268.0], categories: None }
INFO:  Column "customer_star_rating_avg": Statistics { min: 1.0, max: 5.0, max_abs: 5.0, mean: 4.3082776, median: 4.6666665, mode: 5.0, variance: 0.85199296, std_dev: 0.92303467, missing: 0, distinct: 4911, histogram: [109606, 2313, 6148, 4253, 3472, 57466, 16055, 24703, 30528, 23476, 158009, 78291, 126051, 144898, 126584, 417284, 232599, 307763, 253483, 1727906], ventiles: [2.3333333, 3.0, 3.5, 3.7777777, 4.0, 4.0, 4.2, 4.375, 4.5, 4.6666665, 4.7887325, 4.95, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0], categories: None }
INFO:  Training Model { id: 3, task: regression, algorithm: xgboost, runtime: rust }
INFO:  Hyperparameter searches: 1, cross validation folds: 1
INFO:  Hyperparams: {}
INFO:  Metrics: {"r2": 0.6684715, "mean_absolute_error": 0.43539175, "mean_squared_error": 0.47162533, "fit_time": 13.076226, "score_time": 0.10688886}
INFO:  Deploying model id: 3
```

| project           | task       | algorithm | deployed |
|-------------------|------------|-----------|----------|
| our reviews model | regression | xgboost   | true     |

!!!

Our second model had a slightly better r2 value, so it was automatically deployed as the new winner. We can spot check some results with the same query as before:

```
SELECT
  star_rating,
  pgml.predict(
    project_name => 'our reviews model',
    features => ARRAY[
      cosine_similarity,
      movie_total_reviews,
      movie_star_rating_avg,
      customer_total_reviews,
      customer_star_rating_avg
    ]
  ) AS prediction
FROM reviews_for_model
LIMIT 10;
```

!!! results "169.680 ms"

| star_rating | prediction |
|-------------|------------|
| 5           | 4.8721976  |
| 5           | 4.47331    |
| 4           | 4.221939   |
| 5           | 4.521522   |
| 5           | 4.872866   |
| 5           | 4.8721976  |
| 5           | 4.1635613  |
| 4           | 3.9177465  |
| 5           | 4.872866   |
| 5           | 4.872866   |

!!!

By default, xgboost will use 10 trees. We can increase this by passing in a hyperparameter. It'll take longer, but often more trees can help tease out some more complex relationships in the data. Let's try 100 trees:

```sql
SELECT * FROM pgml.train(
  project_name => 'our reviews model',
  task => 'regression',
  relation_name => 'reviews_for_model',
  y_column_name => 'star_rating',
  algorithm => 'xgboost',
  hyperparams => '{
    "n_estimators": 100
  }'
);
```

!!! results "1.5 min"

```
INFO:  Snapshotting table "reviews_for_model", this may take a little while...
INFO:  Dataset { num_features: 5, num_labels: 1, num_distinct_labels: 0, num_rows: 5134517, num_train_rows: 3850888, num_test_rows: 1283629 }
INFO:  Column "star_rating": Statistics { min: 1.0, max: 5.0, max_abs: 5.0, mean: 4.307681, median: 5.0, mode: 5.0, variance: 1.3873324, std_dev: 1.1778507, missing: 0, distinct: 5, histogram: [248740, 0, 0, 0, 0, 158931, 0, 0, 0, 0, 290418, 0, 0, 0, 0, 613454, 0, 0, 0, 2539345], ventiles: [1.0, 2.0, 3.0, 4.0, 4.0, 4.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0], categories: None }
INFO:  Column "cosine_similarity": Statistics { min: 0.73038024, max: 1.0, max_abs: 1.0, mean: 0.98407227, median: 0.98643565, mode: 1.0, variance: 0.0007678081, std_dev: 0.02770935, missing: 0, distinct: 1065927, histogram: [139, 55, 179, 653, 1344, 2122, 3960, 8382, 11893, 15455, 17235, 21212, 24764, 38840, 67740, 125468, 247086, 508314, 836036, 1920011], ventiles: [0.92914546, 0.9493847, 0.9592061, 0.9656064, 0.97034085, 0.97417694, 0.9775268, 0.98058504, 0.9835075, 0.98643565, 0.98952013, 0.9930061, 0.9967673, 0.99948853, 1.0, 1.0, 1.0, 1.0, 1.0], categories: None }
INFO:  Column "movie_total_reviews": Statistics { min: 1.0, max: 4969.0, max_abs: 4969.0, mean: 226.21071, median: 84.0, mode: 1.0, variance: 231646.2, std_dev: 481.2964, missing: 0, distinct: 834, histogram: [2973282, 462640, 170079, 81203, 56738, 33804, 14253, 14832, 6293, 4729, 0, 0, 2989, 3414, 3641, 0, 4207, 8848, 0, 9936], ventiles: [3.0, 7.0, 12.0, 18.0, 25.0, 34.0, 44.0, 55.0, 69.0, 84.0, 101.0, 124.0, 150.0, 184.0, 226.0, 283.0, 370.0, 523.0, 884.0], categories: None }
INFO:  Column "movie_star_rating_avg": Statistics { min: 1.0, max: 5.0, max_abs: 5.0, mean: 4.4302673, median: 4.4761906, mode: 5.0, variance: 0.34565157, std_dev: 0.5879214, missing: 0, distinct: 9058, histogram: [12888, 1385, 6882, 3756, 3903, 15134, 12146, 16423, 24417, 23664, 71072, 84889, 126526, 155994, 212070, 387126, 511706, 769111, 951295, 460501], ventiles: [3.2, 3.5789473, 3.8135593, 3.9956522, 4.090909, 4.1969695, 4.277228, 4.352941, 4.4166665, 4.4761906, 4.5234375, 4.571429, 4.6164384, 4.6568627, 4.6944447, 4.73444, 4.773006, 4.818182, 4.9], categories: None }
INFO:  Column "customer_total_reviews": Statistics { min: 1.0, max: 3588.0, max_abs: 3588.0, mean: 63.471996, median: 4.0, mode: 1.0, variance: 67485.87, std_dev: 259.78043, missing: 0, distinct: 561, histogram: [3602758, 93032, 42129, 26392, 17871, 16154, 9864, 8125, 5465, 9093, 0, 1632, 1711, 1819, 7795, 2065, 2273, 0, 0, 2710], ventiles: [1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 5.0, 7.0, 9.0, 13.0, 19.0, 29.0, 48.0, 93.0, 268.0], categories: None }
INFO:  Column "customer_star_rating_avg": Statistics { min: 1.0, max: 5.0, max_abs: 5.0, mean: 4.3082776, median: 4.6666665, mode: 5.0, variance: 0.8519933, std_dev: 0.92303485, missing: 0, distinct: 4911, histogram: [109606, 2313, 6148, 4253, 3472, 57466, 16055, 24703, 30528, 23476, 158010, 78291, 126050, 144898, 126584, 417283, 232599, 307763, 253484, 1727906], ventiles: [2.3333333, 3.0, 3.5, 3.7777777, 4.0, 4.0, 4.2, 4.375, 4.5, 4.6666665, 4.7887325, 4.95, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0], categories: None }
INFO:  Training Model { id: 4, task: regression, algorithm: xgboost, runtime: rust }
INFO:  Hyperparameter searches: 1, cross validation folds: 1
INFO:  Hyperparams: {
  "n_estimators": 100
}
INFO:  Metrics: {"r2": 0.6796674, "mean_absolute_error": 0.3631905, "mean_squared_error": 0.45570046, "fit_time": 111.8426, "score_time": 0.34201664}
INFO:  Deploying model id: 4
```
| project           | task       | algorithm | deployed |
|-------------------|------------|-----------|----------|
| our reviews model | regression | xgboost   | t        |

!!!

Once again, we've slightly improved our r2 score, and we're now at 0.68.  We've also reduced our mean absolute error to 0.36, and our mean squared error to 0.46.  We're still not doing great, but we're getting better. Choosing the right algorithm and the right hyperparameters can make a big difference, but a full exploration is beyond the scope of this article. When you're not getting much better results, it's time to look at your data.


### Using embeddings as features

```sql
CREATE OR REPLACE VIEW reviews_with_embeddings_for_model AS
SELECT
  star_rating::FLOAT4,
  (1 - (customers.movie_embedding_e5_large <=> movies.review_embedding_e5_large) )::FLOAT4 AS cosine_similarity,
  movies.total_reviews::FLOAT4 AS movie_total_reviews,
  movies.star_rating_avg::FLOAT4 AS movie_star_rating_avg,
  customers.total_reviews::FLOAT4 AS customer_total_reviews,
  customers.star_rating_avg::FLOAT4 AS customer_star_rating_avg,
  customers.movie_embedding_e5_large::FLOAT4[] AS customer_movie_embedding_e5_large,
  movies.review_embedding_e5_large::FLOAT4[] AS movie_review_embedding_e5_large
FROM pgml.amazon_us_reviews
JOIN customers ON customers.id = amazon_us_reviews.customer_id
JOIN movies ON movies.id = amazon_us_reviews.product_id
WHERE star_rating IS NOT NULL
LIMIT 100;
```

!!!results "52.949 ms"
CREATE VIEW
!!!

And now we'll train a new model using the embeddings as features.

```sql
SELECT * FROM pgml.train(
  project_name => 'our reviews model',
  task => 'regression',
  relation_name => 'reviews_with_embeddings_for_model',
  y_column_name => 'star_rating',
  algorithm => 'xgboost',
  hyperparams => '{
    "n_estimators": 100
  }'
);
```

193GB RAM
