---
image: .gitbook/assets/blog_image_generating_llm_embeddings.png
features: true
description: >-
  How to use the pgml.embed(...) function to generate embeddings with free and
  open source models in your own database.
---

# Generating LLM embeddings with open source models in PostgresML

<div align="left">

<figure><img src=".gitbook/assets/montana.jpg" alt="Author" width="125"><figcaption></figcaption></figure>

</div>

Montana Low

April 21, 2023

PostgresML makes it easy to generate embeddings from text in your database using a large selection of state-of-the-art models with one simple call to **`pgml.embed`**`(model_name, text)`. Prove the results in this series to your own satisfaction, for free, by signing up for a GPU accelerated database.

This article is the first in a multipart series that will show you how to build a post-modern semantic search and recommendation engine, including personalization, using open source models.

1. Generating LLM Embeddings with HuggingFace models
2. Tuning vector recall with pgvector
3. Personalizing embedding results with application data
4. Optimizing semantic results with an XGBoost ranking model - coming soon!

## Introduction

In recent years, embeddings have become an increasingly popular technique in machine learning and data analysis. They are essentially vector representations of data points that capture their underlying characteristics or features. In most programming environments, vectors can be efficiently represented as native array datatypes. They can be used for a wide range of applications, from natural language processing to image recognition and recommendation systems.

They can also turn natural language into quantitative features for downstream machine learning models and applications.

<figure><img src=".gitbook/assets/image (27).png" alt=""><figcaption><p>Embeddings show us the relationships between rows in the database.</p></figcaption></figure>

A popular use case driving the adoption of "vector databases" is doing similarity search on embeddings, often referred to as "Semantic Search". This is a powerful technique that allows you to find similar items in large datasets by comparing their vectors. For example, you could use it to find similar products in an e-commerce site, similar songs in a music streaming service, or similar documents given a text query.

Postgres is a good candidate for this type of application because it's a general purpose database that can store both the embeddings and the metadata in the same place, and has a rich set of features for querying and analyzing them, including fast vector indexes used for search.

This chapter is the first in a multipart series that will show you how to build a modern semantic search and recommendation engine, including personalization, using PostgresML and open source models. We'll show you how to use the **`pgml.embed`** function to generate embeddings from text in your database using an open source pretrained model. Further chapters will expand on how to implement many of the different use cases for embeddings in Postgres, like similarity search, personalization, recommendations and fine-tuned models.

## It always starts with data

Most general purpose databases are full of all sorts of great data for machine learning use cases. Text data has historically been more difficult to deal with using complex Natural Language Processing techniques, but embeddings created from open source models can effectively turn unstructured text into structured features, perfect for more straightforward implementations.

In this example, we'll demonstrate how to generate embeddings for products on an e-commerce site. We'll use a public dataset of millions of product reviews from the [Amazon US Reviews](https://huggingface.co/datasets/amazon\_us\_reviews). It includes the product title, a text review written by a customer and some additional metadata about the product, like category. With just a few pieces of data, we can create a full-featured and personalized product search and recommendation engine, using both generic embeddings and later, additional fine-tuned models trained with PostgresML.

PostgresML includes a convenience function for loading public datasets from [HuggingFace](https://huggingface.co/datasets) directly into your database. To load the DVD subset of the Amazon US Reviews dataset into your database, run the following command:

!!! code\_block

```postgresql
SELECT *
FROM pgml.load_dataset('amazon_us_reviews', 'Video_DVD_v1_00');
```

!!!

It took about 23 minutes to download the 7.1GB raw dataset with 5,069,140 rows into a table within the `pgml` schema (where all PostgresML functionality is name-spaced). Once it's done, you can see the table structure with the following command:

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

Let's take a peek at the first 5 rows of data:

!!! code\_block

```postgresql
SELECT *
FROM pgml.amazon_us_reviews
LIMIT 5;
```

!!! results

| marketplace | customer\_id | review\_id     | product\_id | product\_parent | product\_title                                                                                                      | product\_category | star\_rating | helpful\_votes | total\_votes | vine | verified\_purchase | review\_headline                                    | review\_body                                                                                                                                                                                                      | review\_date |
| ----------- | ------------ | -------------- | ----------- | --------------- | ------------------------------------------------------------------------------------------------------------------- | ----------------- | ------------ | -------------- | ------------ | ---- | ------------------ | --------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------ |
| US          | 27288431     | R33UPQQUZQEM8  | B005T4ND06  | 400024643       | Yoga for Movement Disorders DVD: Rebuilding Strength, Balance, and Flexibility for Parkinson's Disease and Dystonia | Video DVD         | 5            | 3              | 3            | 0    | 1                  | This was a gift for my aunt who has Parkinson's ... | This was a gift for my aunt who has Parkinson's. While I have not previewed it myself, I also have not gotten any complaints. My prior experiences with yoga tell me this should be just what the doctor ordered. | 2015-08-31   |
| US          | 13722556     | R3IKTNQQPD9662 | B004EPZ070  | 685335564       | Something Borrowed                                                                                                  | Video DVD         | 5            | 0              | 0            | 0    | 1                  | Five Stars                                          | Teats my heart out.                                                                                                                                                                                               | 2015-08-31   |
| US          | 20381037     | R3U27V5QMCP27T | B005S9EKCW  | 922008804       | Les Miserables (2012) \[Blu-ray]                                                                                    | Video DVD         | 5            | 1              | 1            | 0    | 1                  | Great movie!                                        | Great movie.                                                                                                                                                                                                      | 2015-08-31   |
| US          | 24852644     | R2TOH2QKNK4IOC | B00FC1ZCB4  | 326560548       | Alien Anthology and Prometheus Bundle \[Blu-ray]                                                                    | Video DVD         | 5            | 0              | 1            | 0    | 1                  | Amazing                                             | My husband was so excited to receive these as a gift! Great picture quality and great value!                                                                                                                      | 2015-08-31   |
| US          | 15556113     | R2XQG5NJ59UFMY | B002ZG98Z0  | 637495038       | Sex and the City 2                                                                                                  | Video DVD         | 5            | 0              | 0            | 0    | 1                  | Five Stars                                          | Love this series.                                                                                                                                                                                                 | 2015-08-31   |

!!!

!!!

## Generating embeddings from natural language text

PostgresML provides a simple interface to generate embeddings from text in your database. You can use the [`pgml.embed`](https://postgresml.org/docs/open-source/pgml/guides/transformers/embeddings) function to generate embeddings for a column of text. The function takes a transformer name and a text value. The transformer will automatically be downloaded and cached on your connection process for reuse. You can see a list of potential good candidate models to generate embeddings on the [Massive Text Embedding Benchmark leaderboard](https://huggingface.co/spaces/mteb/leaderboard).

Since our corpus of documents (movie reviews) are all relatively short and similar in style, we don't need a large model. [`Alibaba-NLP/gte-base-en-v1.5`](https://huggingface.co/Alibaba-NLP/gte-base-en-v1.5) will be a good first attempt. The great thing about PostgresML is you can always regenerate your embeddings later to experiment with different embedding models.

It takes a couple of minutes to download and cache the `Alibaba-NLP/gte-base-en-v1.5` model to generate the first embedding. After that, it's pretty fast.

Note how we prefix the text we want to embed with either `passage:` or `query:` , the e5 model requires us to prefix our data with `passage:` if we're generating embeddings for our corpus and `query:` if we want to find semantically similar content.

```postgresql
SELECT pgml.embed('Alibaba-NLP/gte-base-en-v1.5', 'passage: hi mom');
```

This is a pretty powerful function, because we can pass any arbitrary text to any open source model, and it will generate an embedding for us. We can benchmark how long it takes to generate an embedding for a single review, using client-side timings in Postgres:

```postgresql
\timing on
```

Aside from using this function with strings passed from a client, we can use it on strings already present in our database tables by calling **pgml.embed** on columns. For example, we can generate an embedding for the first review using a pretty simple query:

!!! generic

!!! code_block time="54.820 ms"

```postgresql
SELECT
    review_body,
    pgml.embed('Alibaba-NLP/gte-base-en-v1.5', 'passage: ' || review_body)
FROM pgml.amazon_us_reviews
LIMIT 1;
```

!!!

!!! results

```postgressql
CREATE INDEX
```

!!!

!!!

Time to generate an embedding increases with the length of the input text, and varies widely between different models. If we up our batch size (controlled by `LIMIT`), we can see the average time to compute an embedding on the first 1000 reviews is about 17ms per review:

!!! code\_block time="17955.026 ms"

```postgresql
SELECT
    review_body,
    pgml.embed('Alibaba-NLP/gte-base-en-v1.5', 'passage: ' || review_body) AS embedding
FROM pgml.amazon_us_reviews
LIMIT 1000;
```

!!!

## Comparing different models and hardware performance

This database is using a single GPU with 32GB RAM and 8 vCPUs with 16GB RAM. Running these benchmarks while looking at the database processes with `htop` and `nvidia-smi`, it becomes clear that the bottleneck in this case is actually tokenizing the strings which happens in a single thread on the CPU, not computing the embeddings on the GPU which was only 20% utilized during the query.

We can also do a quick sanity check to make sure we're really getting value out of our GPU by passing the device to our embedding function:

!!! code\_block time="30421.491 ms"

```postgresql
SELECT
    reviqew_body,
    pgml.embed(
        'Alibaba-NLP/gte-base-en-v1.5',
        'passage: ' || review_body,
        '{"device": "cpu"}'
    ) AS embedding
FROM pgml.amazon_us_reviews
LIMIT 1000;
```

!!!

Forcing the embedding function to use `cpu` is almost 2x slower than `cuda` which is the default when GPUs are available.

If you're managing dedicated hardware, there's always a decision to be made about resource utilization. If this is a multi-workload database with other queries using the GPU, it's probably great that we're not completely hogging it with our multi-decade-Amazon-scale data import process, but if this is a machine we've spun up just for this task, we can up the resource utilization to 4 concurrent connections, all running on a subset of the data to more completely utilize our CPU, GPU and RAM.

Another consideration is that GPUs are much more expensive right now than CPUs, and if we're primarily interested in backfilling a dataset like this, high concurrency across many CPU cores might just be the price-competitive winner.

With 4x concurrency and a GPU, it'll take about 6 hours to compute all 5 million embeddings, which will cost $72 on PostgresML Cloud. If we use the CPU instead of the GPU, we'll probably want more cores and higher concurrency to plug through the job faster. A 96 CPU core machine could complete the job in half the time our single GPU would take and at a lower hourly cost as well, for a total cost of $24. It's overall more cost-effective and faster in parallel, but keep in mind if you're interactively generating embeddings for a user facing application, it will add double the latency, 30ms CPU vs 17ms for GPU.

For comparison, it would cost about $299 to use OpenAI's cheapest embedding model to process this dataset. Their API calls average about 300ms, although they have high variability (200-400ms) and greater than 1000ms p99 in our measurements. They also have a default rate limit of 200 tokens per minute which means it would take 1,425 years to process this dataset. You better call ahead.

| Processor | Latency | Cost | Time      |
| --------- | ------- | ---- | --------- |
| CPU       | 30ms    | $24  | 3 hours   |
| GPU       | 17ms    | $72  | 6 hours   |
| OpenAI    | 300ms   | $299 | millennia |

You can also find embedding models that outperform OpenAI's `text-embedding-ada-002` model across many different tests on the [leaderboard](https://huggingface.co/spaces/mteb/leaderboard). It's always best to do your own benchmarking with your data, models, and hardware to find the best fit for your use case.

> _HTTP requests to a different datacenter cost more time and money for lower reliability than co-located compute and storage._

## Instructor embedding models

The current leading model is `hkunlp/instructor-xl`. Instructor models take an additional `instruction` parameter which includes context for the embeddings use case, similar to prompts before text generation tasks.

!!! note

    "Alibaba-NLP/gte-base-en-v1.5" surpassed the quality of instructor-xl, and should be used instead, but we've left this documentation available for existing users 

!!!

Instructions can provide a "classification" or "topic" for the text:

#### Classification

!!! code\_block time="17.912ms"

```postgresql
SELECT pgml.embed(
    transformer => 'hkunlp/instructor-xl',
    text => 'The Federal Reserve on Wednesday raised its benchmark interest rate.',
    kwargs => '{"instruction": "Represent the Financial statement:"}'
);
```

!!!

They can also specify particular use cases for the embedding:

#### Querying

!!! code\_block time="24.263 ms"

```postgresql
SELECT pgml.embed(
    transformer => 'hkunlp/instructor-xl',
    text => 'where is the food stored in a yam plant',
    kwargs => '{
        "instruction": "Represent the Wikipedia question for retrieving supporting documents:"
    }'
);
```

!!!

#### Indexing

!!! code\_block time="30.571 ms"

```postgresql
SELECT pgml.embed(
    transformer => 'hkunlp/instructor-xl',
    text => 'Disparate impact in United States labor law refers to practices in employment, housing, and other areas that adversely affect one group of people of a protected characteristic more than another, even though rules applied by employers or landlords are formally neutral. Although the protected classes vary by statute, most federal civil rights laws protect based on race, color, religion, national origin, and sex as protected traits, and some laws include disability status and other traits as well.',
    kwargs => '{"instruction": "Represent the Wikipedia document for retrieval:"}'
);
```

!!!

#### Clustering

!!! code\_block time="18.986 ms"

```postgresql
SELECT pgml.embed(
    transformer => 'hkunlp/instructor-xl',
    text => 'Dynamical Scalar Degree of Freedom in Horava-Lifshitz Gravity"}',
    kwargs => '{"instruction": "Represent the Medicine sentence for clustering:"}'
);
```

!!!

Performance remains relatively good, even with the most advanced models.

## Generating embeddings for a large dataset

For our use case, we want to generate an embedding for every single review in the dataset. We'll use the `vector` datatype available from the `pgvector` extension to store (and later index) embeddings efficiently. All PostgresML cloud installations include [pgvector](https://github.com/pgvector/pgvector). To enable this extension in your database, you can run:

```postgresql
CREATE EXTENSION vector;
```

Then we can add a `vector` column for our review embeddings, with 384 dimensions (the size of e5-small embeddings):

```postgresql
ALTER TABLE pgml.amazon_us_reviews
ADD COLUMN review_embedding_e5_large vector(1024);
```

It's best practice to keep running queries on a production database relatively short, so rather than trying to update all 5M rows in one multi-hour query, we should write a function to issue the updates in smaller batches. To make iterating over the rows easier and more efficient, we'll add an `id` column with an index to our table:

```postgresql
ALTER TABLE pgml.amazon_us_reviews
ADD COLUMN id SERIAL PRIMARY KEY;
```

Every language/framework/codebase has its own preferred method for backfilling data in a table. The 2 most important considerations are:

1. Keep the number of rows per query small enough that the queries take less than a second
2. More concurrency will get the job done faster, but keep in mind the other workloads on your database

Here's an example of a very simple back-fill job implemented in pure PGSQL, but I'd also love to see example PRs opened with your techniques in your language of choice for tasks like this.

```postgresql
DO $$
BEGIN
    FOR i in 1..(SELECT max(id) FROM pgml.amazon_us_reviews) by 10 LOOP
        BEGIN RAISE NOTICE 'updating % to %', i, i + 10; END;

        UPDATE pgml.amazon_us_reviews
        SET review_embedding_e5_large = pgml.embed(
                'Alibaba-NLP/gte-base-en-v1.5',
                'passage: ' || review_body
            )
        WHERE id BETWEEN i AND i + 10
            AND review_embedding_e5_large IS NULL;

        COMMIT;
    END LOOP;
END;
$$;
```

## What's next?

That's it for now. We've got an Amazon scale table with state-of-the-art machine learning embeddings. As a premature optimization, we'll go ahead and build an index on our new column to make our future vector similarity queries faster. For the full documentation on vector indexes in Postgres see the [pgvector docs](https://github.com/pgvector/pgvector).

!!! code\_block time="4068909.269 ms (01:07:48.909)"

```postgresql
CREATE INDEX CONCURRENTLY index_amazon_us_reviews_on_review_embedding_e5_large
ON pgml.amazon_us_reviews
USING ivfflat (review_embedding_e5_large vector_cosine_ops)
WITH (lists = 2000);
```

!!!

!!! tip

Create indexes `CONCURRENTLY` to avoid locking your table for other queries.

!!!

Building a vector index on a table with this many entries takes a while, so this is a good time to take a coffee break. In the next article we'll look at how to query these embeddings to find the best products and make personalized recommendations for users. We'll also cover updating an index in real time as new data comes in.
