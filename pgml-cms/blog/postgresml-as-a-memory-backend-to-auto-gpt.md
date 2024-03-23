---
description: >-
  Auto-GPT is an open-source autonomous AI tool that can use PostgresML as
  memory backend to store and access data from previous queries or private data.
---

# PostgresML as a memory backend to Auto-GPT

<div align="left">

<figure><img src=".gitbook/assets/santi.jpg" alt="Author" width="158"><figcaption></figcaption></figure>

</div>

Santi Adavani

May 3, 2023

Auto-GPT is an open-source, autonomous AI tool that uses GPT-4 to interact with software and services online. PostgresML is an open-source library that allows you to add machine learning capabilities to your PostgreSQL database.

In this blog post, I will show you how to add PostgresML as a memory backend to AutoGPT. This will allow you to use the power of PostgresML to improve the performance and scalability of AutoGPT.

## What is Auto-GPT?

Auto-GPT is an open-source, autonomous AI tool that uses GPT-4 to interact with software and services online. It was developed by Toran Bruce Richards and released on March 30, 2023.

Auto-GPT can perform a variety of tasks, including:

* Debugging code
* Writing emails
* Conducting market research
* Developing software applications

Auto-GPT is still under development, but it has the potential to be a powerful tool for a variety of tasks. It is still early days, but Auto-GPT is already being used by some businesses and individuals to improve their productivity and efficiency.

## What is PostgresML?

PostgresML is a machine learning extension to PostgreSQL that enables you to perform training and inference on text and tabular data using SQL queries. With PostgresML, you can seamlessly integrate machine learning models into your PostgreSQL database and harness the power of cutting-edge algorithms to process data efficiently.

PostgresML supports a variety of machine learning algorithms, including:

* Natural language processing
* Sentence Embeddings
* Regression
* Classification

## What is a memory backend to Auto-GPT and why is it important?

A memory backend is a way to store and access data that AutoGPT needs to perform its tasks. AutoGPT has both short-term and long-term memory. Short-term memory is used to store information that AutoGPT needs to access quickly, such as the current conversation or the state of a game. Long-term memory is used to store information that AutoGPT needs to access more slowly, such as general knowledge or the rules of a game.

There are a number of different memory backends available for AutoGPT, each with its own advantages and disadvantages. The choice of memory backend depends on the specific needs of the application. Some of the most popular memory backends for AutoGPT are Redis, Pinecone, Milvus, and Weaviate.

## Why add PostgresML as a memory backend to Auto-GPT?

Developing Auto-GPT-powered applications requires a range of APIs from OpenAI as well as a stateful database to store data related to business logic. PostgresML brings AI tasks like sentence embeddings to the database, reducing complexity for app developers, and yielding a host of additional performance, cost and quality advantages. We will use the vector datatype available from the pgvector extension to store (and later index) embeddings efficiently.

## Register the memory backend module with Auto-GPT

Adding PostgresML as a memory backend to Auto-GPT is a relatively simple process. The steps involved are:

1.  Download and install Auto-GPT.

    ```shell
    git clone https://github.com/postgresml/Auto-GPT
    cd Auto-GPT
    git checkout stable-0.2.2
    python3 -m venv venv
    source venv/bin/activate
    pip install -r requirements.txt
    ```
2. Start PostgresML using [Docker](https://github.com/postgresml/postgresml#docker) or [sign up for a free PostgresML account](https://postgresml.org/signup).
3. Install `postgresql` command line utility
   * Ubuntu: `sudo apt install libpq-dev`
   * Centos/Fedora/Cygwin/Babun.: `sudo yum install libpq-devel`
   * Mac: `brew install postgresql`
4. Install `psycopg2` in
   * `pip install psycopg2`
5.  Setting up environment variables

    In your `.env` file set the following if you are using Docker:

    ```shell
    POSTGRESML_HOST=localhost
    POSTGRESML_PORT=5443
    POSTGRESML_USERNAME=postgres
    POSTGRESML_PASSWORD=""
    POSTGRESML_DATABASE=pgml_development
    POSTGRESML_TABLENAME =autogpt_text_embeddings
    ```

    If you are using PostgresML cloud, use the hostname and credentials from the cloud platform.

!!! note

We are using PostgresML fork of Auto-GPT for this tutorial. Our [PR](https://github.com/Significant-Gravitas/Auto-GPT/pull/3274) to add PostgresML as a memory backend to Auto-GPT is currently under review by Auto-GPT team and will be available as an official backend soon!

!!!

## Start Auto-GPT with PostgresML memory backend

Once the `.env` file has all the relevant PostgresML settings you can start autogpt that uses PostgresML backend using the following command:

```shell
python -m autogpt -m postgresml
```

You will see Auto-GPT in action with PostgresML backend as shown below. You should see _Using memory of type: PostgresMLMemory_ in the logs.

<figure><img src=".gitbook/assets/image (23).png" alt=""><figcaption></figcaption></figure>

## Conclusion

In this blog post, I showed you how to add PostgresML as a memory backend to Auto-GPT. Adding PostgresML as a memory backend can significantly accelerate performance and scalability of Auto-GPT. It can enable you to rapidly prototype with Auto-GPT and build AI-powered applications.
