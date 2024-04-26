# Serverless databases

A Serverless PostgresML database can be created in less than 5 seconds and provides immediate access to modern GPU acceleration, the entire HuggingFace library of LLMs, and dozens of supervised learning algorithms like XGBoost, LightGBM, Catboost, and everything from Scikit-learn.

Serverless databases start at $0 and have a generous free tier. A free tier user will be able to access the GPUs and 5GB of disk storage for their hobby projects, or to just try PostgresML for the first time, without having to provide a credit card. The free tier has no other limits and can be used to power personal projects without having to worry about being shut down or scaled down.

### Create a Serverless database

To create a Serverless database, make sure you have an account on postgresml.org. If you don't, you can create one now.

Once logged in, select "New Database" from the left menu and choose the Serverless Plan.

<figure><img src="../../.gitbook/assets/image (1).png" alt=""><figcaption><p>Create new database</p></figcaption></figure>

<figure><img src="../../.gitbook/assets/image (2).png" alt=""><figcaption><p>Choose the Serverless plan</p></figcaption></figure>

### Configuring the database

Serverless databases have three (3) configuration options: GPU Cache, Storage, and GPU Concurrency.

<figure><img src="../../.gitbook/assets/image (3).png" alt=""><figcaption><p>The three (3) configuration options for a Serverless database</p></figcaption></figure>

#### GPU Cache

GPU Cache is the amount of GPU memory that will be reserved and guaranteed for your database to use in case you want to use GPU accelerated LLMs. Models like Llama 2, Mistral, and GPT-3 require a GPU to generate text at a reasonable speed, usable in production applications. This setting, if set to the correct amount of GPU RAM required by the such models, will ensure that the model you use remains in the GPU cache for as long as you need it.

If you don't provision any GPU Cache capacity, you can still use GPU acceleration for running LLMs and other models. However, this capacity won't be guaranteed and if we need to evict your model from the cache to serve another request, we may have to do so, and you'll have to wait until that request is complete to use your model again.

#### Storage

Disk storage is used by your database to store data in your tables. This storage metric only applies to PostgreSQL tables. Storage of LLM models used by your database is free. You can scale your storage up at any time, but you can't scale it down without deleting your data. The free tier includes 5GB of storage.

#### GPU Concurrency

GPU Concurrency is the amount of concurrent queries (executed at the same time) that your serverless database can serve. If you're using LLMs, they will be loaded on one or more GPUs, so for the duration of the request, your database will have access to the entire GPU. However, if you need to execute more than one request at a time, which will happen if your application starts getting some more traffic in production, you might need to increase your GPU Concurrency to accommodate that new traffic.

If you don't provision additional GPU Concurrency, requests that can't be served immediately with your current capacity will wait in a queue until your in-flight request completes and a GPU is available to serve them.
