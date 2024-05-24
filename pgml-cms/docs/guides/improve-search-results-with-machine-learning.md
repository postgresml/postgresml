# Improve Search Results with Machine Learning

PostgresML makes it easy to use machine learning with your database and to scale workloads horizontally in our cloud. One of the most common use cases is to improve search results. In this article, we'll show you how to build a search engine from the ground up, that leverages multiple types of natural language processing (NLP) and machine learning (ML) models to improve search results, including vector search and personalization with embeddings.

## Keyword Search

One important takeaway from this article is that search engines are built in multiple layers from simple to complex and use iterative refinement of results along the way. We'll explore what that composition and iterative refinement looks like using standard SQL and the additional functions provided by PostgresML. Our foundational layer is the traditional form of search, keyword search. This is the type of search you're probably most familiar with. You type a few words into a search box, and get back a list of results that contain those words.

### Queries

Our search application will start with a **documents** table. Our documents have a title and a body, as well as a unique id for our application to reference when updating or deleting existing documents. We create our table with the standard SQL `CREATE TABLE` syntax.

!!! generic

!!! code\_block time="10.493 ms"

```postgresql
CREATE TABLE documents (
  id BIGSERIAL PRIMARY KEY,
  title TEXT,
  body TEXT
);
```

!!!

!!!

We can add new documents to our _text corpus_ with the standard SQL `INSERT` statement. Postgres will automatically take care of generating the unique ids, so we'll add a few **documents** with just a **title** and **body** to get started.

!!! generic

!!! code\_block time="3.417 ms"

```postgresql
INSERT INTO documents (title, body) VALUES 
  ('This is a title', 'This is the body of the first document.'),
  ('This is another title', 'This is the body of the second document.'),
  ('This is the third title', 'This is the body of the third document.')
;
```

!!!

!!!

As you can see, it takes a few milliseconds to insert new documents into our table. Postgres is pretty fast out of the box. We'll also cover scaling and tuning in more depth later on for production workloads.

Now that we have some documents, we can immediately start using built in keyword search functionality. Keyword queries allow us to find documents that contain the words in our queries, but not necessarily in the order we typed them. Standard variations on a root word, like pluralization, or past tense, should also match our queries. This is accomplished by "stemming" the words in our queries and documents. Postgres provides 2 important functions that implement these grammatical cleanup rules on queries and documents.

* `to_tsvector(config, text)` will turn plain text into a `tsvector` that can also be indexed for faster recall.
* `to_tsquery(config, text)` will turn a plain text query into a boolean rule (and, or, not, phrase) `tsquery` that can match `@@` against a `tsvector`.

You can configure the grammatical rules in many advanced ways, but we'll use the built-in `english` config for our examples. Here's how we can use the match `@@` operator with these functions to find documents that contain the word "second" in the **body**.

!!! generic

!!! code\_block time="0.651 ms"

```postgresql
SELECT * 
FROM documents
WHERE to_tsvector('english', body) @@ to_tsquery('english', 'second');
```

!!!

!!! results

| id | title                 | body                                     |
| -- | --------------------- | ---------------------------------------- |
| 2  | This is another title | This is the body of the second document. |

!!!

!!!

Postgres provides the complete reference [documentation](https://www.postgresql.org/docs/current/datatype-textsearch.html) on these functions.

### Indexing

Postgres treats everything in the standard SQL `WHERE` clause as a filter. By default, it makes this keyword search work by scanning the entire table, converting each document body to a `tsvector`, and then comparing the `tsquery` to the `tsvector`. This is called a "sequential scan". It's fine for small tables, but for production use cases at scale, we'll need a more efficient solution.

The first step is to store the `tsvector` in the table, so we don't have to generate it during each search. We can do this by adding a new `GENERATED` column to our table, that will automatically stay up to date. We also want to search both the **title** and **body**, so we'll concatenate `||` the fields we want to include in our search, separated by a simple space `' '`.

!!! generic

!!! code\_block time="17.883 ms"

```postgresql
ALTER TABLE documents
ADD COLUMN title_and_body_text tsvector
GENERATED ALWAYS AS (to_tsvector('english', title || ' ' || body )) STORED;
```

!!!

!!!

One nice aspect of generated columns is that they will backfill the data for existing rows. They can also be indexed, just like any other column. We can add a Generalized Inverted Index (GIN) on this new column that will pre-compute the lists of all documents that contain each keyword. This will allow us to skip the sequential scan, and instead use the index to find the exact list of documents that satisfy any given `tsquery`.

!!! generic

!!! code\_block time="5.145 ms"

```postgresql
CREATE INDEX documents_title_and_body_text_index 
ON documents 
USING GIN (title_and_body_text);
```

!!!

!!!

And now, we'll demonstrate a slightly more complex `tsquery`, that requires both the keywords **another** and **second** to match `@@` the **title** or **body** of the document, which will automatically use our index on **title\_and\_body\_text**.

!!! generic

!!! code\_block time="3.673 ms"

```postgresql
SELECT * 
FROM documents
WHERE title_and_body_text @@ to_tsquery('english', 'another & second');
```

!!!

!!! results

| id | title                 | body                                     | title\_and\_body\_text                                |
| -- | --------------------- | ---------------------------------------- | ----------------------------------------------------- |
| 2  | This is another title | This is the body of the second document. | 'anoth':3 'bodi':8 'document':12 'second':11 'titl':4 |

!!!

!!!

We can see our new `tsvector` column in the results now as well, since we used `SELECT *`. You'll notice that the `tsvector` contains the stemmed words from both the **title** and **body**, along with their position. The position information allows Postgres to support _phrase_ matches as well as single keywords. You'll also notice that _stopwords_, like "the", "is", and "of" have been removed. This is a common optimization for keyword search, since these words are so common, they don't add much value to the search results.

### Ranking

Ranking is a critical component of search, and it's also where Machine Learning becomes critical for great results. Our users will expect us to sort our results with the most relevant at the top. A simple arithmetic relevance score is provided `ts_rank`. It computes the Term Frequency (TF) of each keyword in the query that matches the document. For example, if the document has 2 keyword matches out of 5 words total, it's `ts_rank` will be `2 / 5 = 0.4`. The more matches and the fewer total words, the higher the score and the more relevant the document.

With multiple query terms OR `|` together, the `ts_rank` will add the numerators and denominators to account for both. For example, if the document has 2 keyword matches out of 5 words total for the first query term, and 1 keyword match out of 5 words total for the second query term, it's ts\_rank will be `(2 + 1) / (5 + 5) = 0.3`. The full `ts_rank` function has many additional options and configurations that you can read about in the [documentation](https://www.postgresql.org/docs/current/textsearch-controls.html#TEXTSEARCH-RANKING), but this should give you the basic idea.

!!! generic

!!! code\_block time="0.561 ms"

```postgresql
SELECT ts_rank(title_and_body_text, to_tsquery('english', 'second | title')), *   
FROM documents 
ORDER BY ts_rank DESC;
```

!!!

!!! results

| ts\_rank    | id | title                   | body                                     | title\_and\_body\_text                                |
| ----------- | -- | ----------------------- | ---------------------------------------- | ----------------------------------------------------- |
| 0.06079271  | 2  | This is another title   | This is the body of the second document. | 'anoth':3 'bodi':8 'document':12 'second':11 'titl':4 |
| 0.030396355 | 1  | This is a title         | This is the body of the first document.  | 'bodi':8 'document':12 'first':11 'titl':4            |
| 0.030396355 | 3  | This is the third title | This is the body of the third document.  | 'bodi':9 'document':13 'third':4,12 'titl':5          |

!!!

!!!

Our document that matches 2 of the keywords has twice the score of the documents that match just one of the keywords. It's important to call out, that this query has no `WHERE` clause. It will rank and return every document in a potentially large table, even when the `ts_rank` is 0, i.e. not a match at all. We'll generally want to add both a basic match `@@` filter that can leverage an index, and a `LIMIT` to make sure we're not returning completely irrelevant documents or too many results per page.

### Boosting

A quick improvement we could make to our search query would be to differentiate relevance of the title and body. It's intuitive that a keyword match in the title is more relevant than a keyword match in the body. We can implement a simple boosting function by multiplying the title rank 2x, and adding it to the body rank. This will _boost_ title matches up the rankings in our final results list. This can be done by creating a simple arithmetic formula in the `ORDER BY` clause.

!!! generic

!!! code\_block time="0.561 ms"

```postgresql
SELECT 
  ts_rank(title, to_tsquery('english', 'second | title')) AS title_rank,
  ts_rank(body, to_tsquery('english', 'second | title')) AS body_rank,
  *   
FROM documents 
ORDER BY (2 * title_rank) + body_rank DESC;
```

!!!

!!!

Wait a second... is a title match 2x or 10x, or maybe log(π / tsrank2) more relevant than a body match? Since document length penalizes ts\_rank more in the longer body content, maybe we should be boosting body matches instead? You might try a few equations against some test queries, but it's hard to know what the value that works best across all queries is. Optimizing functions like this is one area Machine Learning can help.

## Learning to Rank

So far we've only considered simple statistical measures of relevance like `ts_rank`s TF/IDF, but people have a much more sophisticated idea of relevance. Luckily, they'll tell you exactly what they think is relevant by clicking on it. We can use this feedback to train a model that learns the optimal weights of **title\_rank** vs **body\_rank** for our boosting function. We'll redefine relevance as the probability that a user will click on a search result, given our inputs like **title\_rank** and **body\_rank**.

This is considered a Supervised Learning problem, because we have a labeled dataset of user clicks that we can use to train our model. The inputs to our function are called _features_ of the data for the machine learning model, and the output is often referred to as the _label_.

### Training Data

First things first, we need to record some user clicks on our search results. We'll create a new table to store our training data, which are the observed inputs and output of our new relevance function. In a real system, we'd probably have separate tables to record **sessions**, **searches**, **results**, **clicks** and other events, but for simplicity in this example, we'll just record the exact information we need to train our model in a single table. Everytime we perform a search, we'll record the `ts_rank` for the both the **title** and **body**, and whether the user **clicked** on the result.

!!! generic

!!! code\_block time="0.561 ms"

```postgresql
CREATE TABLE search_result_clicks (
  title_rank REAL,
  body_rank REAL,
  clicked BOOLEAN
);
```

!!!

!!!

One of the hardest parts of machine learning is gathering the data from disparate sources and turning it into features like this. There are often teams of data engineers involved in maintaining endless pipelines from one feature store or data warehouse and then back again. We don't need that complexity in PostgresML and can just insert the ML features directly into the database.

I've made up 4 example searches, across our 3 documents, and recorded the `ts_rank` for the **title** and **body**, and whether the user **clicked** on the result. I've cherry-picked some intuitive results, where the user always clicked on the top ranked document, that has the highest combined title and body ranks. We'll insert this data into our new table.

!!! generic

!!! code\_block time="2.161 ms"

```postgresql
INSERT INTO search_result_clicks 
  (title_rank, body_rank, clicked) 
VALUES
-- search 1
  (0.5, 0.5, true),
  (0.3, 0.2, false),
  (0.1, 0.0, false),
-- search 2
  (0.0, 0.5, true),
  (0.0, 0.2, false),
  (0.0, 0.0, false),
-- search 3
  (0.2, 0.5, true),
  (0.1, 0.2, false),
  (0.0, 0.0, false),
-- search 4
  (0.4, 0.5, true),
  (0.4, 0.2, false),
  (0.4, 0.0, false)
;
```

!!!

!!!

In a real application, we'd record the results of millions of searches results with the ts\_ranks and clicks from our users, but even this small amount of data is enough to train a model with PostgresML. Bootstrapping or back-filling data is also possible with several techniques. You could build the app, and have your admins or employees use it to generate training data before a public release.

### Training a Model to rank search results

We'll train a model for our "Search Ranking" project using the `pgml.train` function, which takes several arguments. The `project_name` is a handle we can use to refer to the model later when we're ranking results, and the `task` is the type of model we want to train. In this case, we want to train a model to predict the probability of a user clicking on a search result, given the `title_rank` and `body_rank` of the result. This is a regression problem, because we're predicting a continuous value between 0 and 1. We could also train a classification model to make a boolean prediction whether a user will click on a result, but we'll save that for another example.

Here goes some machine learning:

!!! generic

!!! code\_block time="6.867 ms"

```postgresql
SELECT * FROM pgml.train(
  project_name => 'Search Ranking',
  task => 'regression',
  relation_name => 'search_result_clicks',
  y_column_name => 'clicked'
);
```

!!!

!!! results

| project        | task       | algorithm | deployed |
| -------------- | ---------- | --------- | -------- |
| Search Ranking | regression | linear    | t        |

!!!

!!!

SQL statements generally begin with `SELECT` to read something, but in this case we're really just interested in reading the result of the training function. The `pgml.train` function takes a few arguments, but the most important are the `relation_name` and `y_column_name`. The `relation_name` is the table we just created with our training data, and the `y_column_name` is the column we want to predict. In this case, we want to predict whether a user will click on a search result, given the **title\_rank** and **body\_rank**. There are two common machine learning **tasks** for making predictions like this. Classification makes a discrete or categorical prediction like `true` or `false`. Regression makes a floating point prediction, akin to the probability that a user will click on a search result. In this case, we want to rank search results from most likely to least likely, so we'll use the `regression` task. The project is just a name for the model we're training, and we'll use it later to make predictions.

Training a model in PostgresML is actually a multiple step pipeline that gets executed to implement best practices. There are options to control the pipeline, but by default, the following steps are executed:

1. The training data is split into a training set and a test set
2. The model is trained on the training set
3. The model is evaluated on the test set
4. The model is saved into `pgml.models` along with the evaluation metrics
5. The model is deployed if it's better than the currently deployed model

!!! tip

The `pgml.train` function will return a table with some information about the training process. It will show several columns of data about the model that was trained, including the accuracy of the model on the training data. You may see calls to `pgml.train` that use `SELECT * FROM pgml.train(...)` instead of `SELECT pgml.train(...)`. Both invocations of the function are equivalent, but calling the function in `FROM` as if it were a table gives a slightly more readable table formatted result output.

!!!

PostgresML automatically deploys a model for online predictions after training, if the **key metric** is a better than the currently deployed model. We'll train many models over time for this project, and you can read more about deployments later.

### Making Predictions

Once a model is trained, you can use `pgml.predict` to use it on new inputs. `pgml.predict` is a function that takes our project name, along with an array of features to predict on. In this case, our features are th `title_rank` and `body_rank`. We can use the `pgml.predict` function to make predictions on the training data, but in a real application, we'd want to make predictions on new data that the model hasn't seen before. Let's do a quick sanity check, and see what the model predicts for all the values of our training data.

!!! generic

!!! code\_block time="3.119 ms"

```postgresql
SELECT 
  clicked, 
  pgml.predict('Search Ranking', array[title_rank, body_rank]) 
FROM search_result_clicks;
```

!!!

!!! results

| clicked | predict     |
| ------- | ----------- |
| t       | 0.88005996  |
| f       | 0.2533733   |
| f       | -0.1604198  |
| t       | 0.910045    |
| f       | 0.27136433  |
| f       | -0.15442279 |
| t       | 0.898051    |
| f       | 0.26536733  |
| f       | -0.15442279 |
| t       | 0.886057    |
| f       | 0.24737626  |
| f       | -0.17841086 |

!!!

!!!

!!! note

If you're watching your database logs, you'll notice the first time a model is used there is a "Model cache miss". PostgresML automatically caches models in memory for faster predictions, and the cache is invalidated when a new model is deployed. The cache is also invalidated when the database is restarted or a connection is closed.

!!!

The model is predicting values close to 1 when there was a click, and values closer to 0 when there wasn't a click. This is a good sign that the model is learning something useful. We can also use the `pgml.predict` function to make predictions on new data, and this is where things actually get interesting in online search results with PostgresML.

### Ranking Search Results with Machine Learning

Search results are often computed in multiple steps of recall and (re)ranking. Each step can apply more sophisticated (and expensive) models on more and more features, before pruning less relevant results for the next step. We're going to expand our original keyword search query to include a machine learning model that will re-rank the results. We'll use the `pgml.predict` function to make predictions on the title and body rank of each result, and then we'll use the predictions to re-rank the results.

It's nice to organize the query into logical steps, and we can use **Common Table Expressions** (CTEs) to do this. CTEs are like temporary tables that only exist for the duration of the query. We'll start by defining a CTE that will rank all the documents in our table by the ts\_rank for title and body text. We define a CTE using the `WITH` keyword, and then we can use the CTE as if it were a table in the rest of the query. We'll name our CTE **first\_pass\_ranked\_documents**. Having the full power of SQL gives us a lot of power to flex in this step.

1. We can efficiently recall matching documents using the keyword index `WHERE title_and_body_text @@ to_tsquery('english', 'second | title'))`
2. We can generate multiple ts\_rank scores for each row the documents using the `ts_rank` function as if they were columns in the table
3. We can order the results by the `title_and_body_rank` and limit the results to the top 100 to avoid wasting time in the next step applying an ML model to less relevant results
4. We'll use this new table in a second query to apply the ML model to the title and body rank of each document and re-rank the results with a second `ORDER BY` clause

!!! generic

!!! code\_block time="2.118 ms"

```postgresql
WITH first_pass_ranked_documents AS (
  SELECT
    -- Compute the ts_rank for the title and body text of each document 
    ts_rank(title_and_body_text, to_tsquery('english', 'second | title')) AS title_and_body_rank,       
    ts_rank(to_tsvector('english', title), to_tsquery('english', 'second | title')) AS title_rank, 
    ts_rank(to_tsvector('english', body), to_tsquery('english', 'second | title')) AS body_rank,
    * 
  FROM documents 
  WHERE title_and_body_text @@ to_tsquery('english', 'second | title')
  ORDER BY title_and_body_rank DESC
  LIMIT 100
)
SELECT
    -- Use the ML model to predict the probability that a user will click on the result
    pgml.predict('Search Ranking', array[title_rank, body_rank]) AS ml_rank,
    *
FROM first_pass_ranked_documents
ORDER BY ml_rank DESC
LIMIT 10;
```

!!!

!!! results

| ml\_rank    | title\_and\_body\_rank | title\_rank | body\_rank  | id | title                   | body                                     | title\_and\_body\_text                                |
| ----------- | ---------------------- | ----------- | ----------- | -- | ----------------------- | ---------------------------------------- | ----------------------------------------------------- |
| -0.09153378 | 0.06079271             | 0.030396355 | 0.030396355 | 2  | This is another title   | This is the body of the second document. | 'anoth':3 'bodi':8 'document':12 'second':11 'titl':4 |
| -0.15624566 | 0.030396355            | 0.030396355 | 0           | 1  | This is a title         | This is the body of the first document.  | 'bodi':8 'document':12 'first':11 'titl':4            |
| -0.15624566 | 0.030396355            | 0.030396355 | 0           | 3  | This is the third title | This is the body of the third document.  | 'bodi':9 'document':13 'third':4,12 'titl':5          |

!!!

!!!

You'll notice that calculating the `ml_rank` adds virtually no additional time to the query. The `ml_rank` is not exactly "well calibrated", since I just made up 4 for searches worth of `search_result_clicks` data, but it's a good example of how we can use machine learning to re-rank search results extremely efficiently, without having to write much code or deploy any new microservices.

You can also be selective about which fields you return to the application for greater efficiency over the network, or return everything for logging and debugging modes. After all, this is all just standard SQL, with a few extra function calls involved to make predictions.

## Next steps with Machine Learning

With composable CTEs and a mature Postgres ecosystem, you can continue to extend your search engine capabilities in many ways.

### Add more features

You can bring a lot more data into the ML model as **features**, or input columns, to improve the quality of the predictions. Many documents have a notion of "popularity" or "quality" metrics, like the `average_star_rating` from customer reviews or `number_of_views` for a video. Another common set of features would be the global Click Through Rate (CTR) and global Conversion Rate (CVR). You should probably track all **sessions**, **searches**, **results**, **clicks** and **conversions** in tables, and compute global stats for how appealing each product is when it appears in search results, along multiple dimensions. Not only should you track the average stats for a document across all searches globally, you can track the stats for every document for each search query it appears in, i.e. the CTR for the "apples" document is different for the "apple" keyword search vs the "fruit" keyword search. So you could use both the global CTR and the keyword specific CTR as features in the model. You might also want to track short term vs long term stats, or things like "freshness".

Postgres offers `MATERIALIZED VIEWS` that can be periodically refreshed to compute and cache these stats table efficiently from the normalized tracking tables your application would write the structured event data into. This prevents write amplification from occurring when a single event causes updates to dozens of related statistics.

### Use more sophisticated ML Algorithms

PostgresML offers more than 50 algorithms. Modern gradient boosted tree based models like XGBoost, LightGBM and CatBoost provide state-of-the-art results for ranking problems like this. They are also relatively fast and efficient. PostgresML makes it simple to just pass an additional `algorithm` parameter to the `pgml.train` function to use a different algorithm. All the resulting models will be tracked in your project, and the best one automatically deployed. You can also pass a specific **model\_id** to `pgml.predict` instead of a **project\_name** to use a specific model. This makes it easy to compare the results of different algorithms statistically. You can also compare the results of different algorithms at the application level in AB tests for business metrics, not just statistical measures like r2.

### Train regularly

You can also retrain the model with new data whenever new data is available which will naturally improve your model over time as the data set grows larger and has more examples including edge cases and outliers. It's important to note you should only need to retrain when there has been a "statistically meaningful" change in the total dataset, not on every single new search or result. Training once a day or once a week is probably sufficient to avoid "concept drift".

An additional benefit of regular training is that you will have faster detection of any breakage in the data pipeline. If the data pipeline breaks, for whatever reason, like the application team drops an important column they didn't realize was in use for training by the model, it'd be much better to see that error show up within 24 hours, and lose 1 day of training data, than to wait until the next time a Data Scientist decides to work on the model, and realize that the data has been lost for the last year, making it impossible to continue using in the next version, potentially leaving you with a model that can never be retrained and never beaten by new versions, until the entire project is revisited from the ground up. That sort of thing happens all the time in other more complicated distributed systems, and it's a huge waste of time and money.

### Vector Search w/ LLM embeddings

PostgresML not only incorporates the latest vector search, including state-of-the\_art HNSW recall provided by pgvector, but it can generate the embeddings _inside the database with no network overhead_ using the latest pre-trained LLMs downloaded from Huggingface. This is big enough to be its own topic, so we've outlined it in a series on how to generate LLM Embeddings with HuggingFace models.

### Personalization & Recommendations

There are a few ways to implement personalization for search results. PostgresML supports both collaborative or content based filtering for personalization and recommendation systems. We've outlined one approach to personalizing embedding results with application data for further reading, but you can implement many different approaches using all the building blocks provided by PostgresML.

### Multi-Modal Search

You may want to offer search results over multiple document types. For example a professional social networking site may return results from **People**, **Companies**, **JobPostings**, etc. You can have features specific to each document type, and PostgresML will handle the `NULL` inputs where documents don't have data for specific feature. This will allow you to build one model that ranks all types of "documents" together to optimize a single global objective.

### Tie it all together in a single query

You can tier multiple models and ranking algorithms together in a single query. For example, you could recall candidates with both vector search and keyword search, join their global document level CTR & CVR and other stats, join more stats for how each document has converted on this exact query, join more personalized stats or vectors from the user history or current session, and input all those features into a tree based model to re-rank the results. Pulling all those features together from multiple feature stores in a microservice architecture and joining at the application layer would be prohibitively slow at scale, but with PostgresML you can do it all in a single query with indexed joins in a few milliseconds on the database, layering CTEs as necessary to keep the query maintainable.

### Make it fast

When you have a dozen joins across many tables in a single query, it's important to make sure the query is fast. We typically target sub 100ms for end to end search latency on large production scale datasets, including LLM embedding generation, vector search, and personalization reranking. You can use standard SQL `EXPLAIN ANALYZE` to see what parts of the query take the cost the most time or memory. Postgres offers many index types (BTREE, GIST, GIN, IVFFLAT, HNSW) which can efficiently deal with billion row datasets of numeric, text, keyword, JSON, vector or even geospatial data.

### Make it scale

Modern machines are available in most clouds with hundreds of cores, which will scale to tens of thousands of queries per second. More advanced techniques like partitioning and sharding can be used to scale beyond billion row datasets or to millions of queries per second. Postgres has tried and true replication patterns that we expose with a simple slider to scale out to as many machines as necessary in our cloud hosted platform, but since PostgresML is open source, you can run it however you're comfortable scaling your Postgres workloads in house as well.

## Conclusion

You can use PostgresML to build a state-of-the-art search engine with cutting edge capabilities on top of your application and domain data. It's easy to get started with our fully hosted platform that provides additional features like horizontal scalability and GPU acceleration for the most intensive workloads at scale. The efficiency inherent to our shared memory implementation without network calls means PostgresML is also more reliable and cheaper to operate than alternatives, and the integrated machine learning algorithms mean you can fully leverage all of your application data. PostgresML is also open source, and we welcome contributions from the community, especially when it comes to the rapidly evolve ML landscape with the latest improvements we're seeing from foundation model capabilities.
