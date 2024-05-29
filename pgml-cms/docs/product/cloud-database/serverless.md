# Serverless databases

A Serverless PostgresML database can be created in less than 5 seconds and provides immediate access to modern GPU acceleration, a predefined set of state-of-the-art large language models that should satisfy most use cases, and dozens of supervised learning algorithms like XGBoost, LightGBM, Catboost, and everything from Scikit-learn.
With a Serverless database, storage and compute resources dynamically adapt to your application's needs, ensuring it can scale down or handle peak loads without overprovisioning.

Serverless databases are billed on a pay-per-use basis and we offer $100 in free credits to get you started!

### Create a Serverless database

To create a Serverless database, make sure you have an account on postgresml.org. If you don't, you can create one now.

Once logged in, select "New Database" from the left menu and choose the Serverless Plan.

<figure><img src="../../.gitbook/assets/image (1).png" alt=""><figcaption><p>Create new database</p></figcaption></figure>

<figure><img src="../../.gitbook/assets/image (2).png" alt=""><figcaption><p>Choose the Serverless plan</p></figcaption></figure>


### Serverless Pricing 
Storage is charged per GB/mo, and all requests by CPU or GPU millisecond of compute required to perform them.

#### Vector & Relational Database
| NAME | PRICING |
| :--- | ---: |
| Tables & Index Storage | $0.20 GB per month |
| Retrieval, Filtering, Ranking & other Queries | $7.50 per hour |
| Embeddings | Included w/ Queries |
| LLMs | Included w/ Queries |
| Fine Tuning | Included w/ Queries |
| Machine Learning | Included w/ Queries |


### Serverless Models

Serverless AI engines come with predefined models and a flexible pricing structure

#### Embedding Models
| NAME | PARAMETERS (M) | MAX INPUT TOKENS | DIMENSIONS | STRENGTHS |
| --- | --- | --- | --- | --- | 
| intfloat/e5-large-v2 | 33.4 | 512 | 384 | High quality, low latency |
| mixedbread-ai/mxbai-embed-large-v1 | 334 | 512 | 1024 | High quality, higher latency |
| Alibaba-NLP/gte-base-en-v1.5 | 137 | 8192 | 768 | Supports up to 8k input tokens, low latency |
| Alibaba-NLP/gte-large-en-v1.5 | 434 | 8192 | 1024 | Supports up to 8k input tokens, higher latency |

#### Instruct Models
| NAME | TOTAL PARAMETERS (M) | ACTIVE PARAMETERS (M) | CONTEXT SIZE | STRENGTHS |
| --- | --- | --- | --- | --- | 
| meta-llama/Meta-Llama-3-70B-Instruct | 70,000 | 70,000 | 8,000 | High quality |
| meta-llama/Meta-Llama-3-8B-Instruct | 8,000 | 8,000 | 8,000 | High quality, low latency |
| microsoft/Phi-3-mini-128k-instruct | 3,820 | 3,820 | 128,000 | Lowest latency |
| mistralai/Mixtral-8x7B-Instruct-v0.1 | 56,000 | 12,900 | 32,768 | MOE high quality |
| mistralai/Mistral-7B-Instruct-v0.2 | 7,000 | 7,000 | 32,768 | High quality, low latency |

#### Summarization Models
| NAME | PARAMETERS (M) | CONTEXT SIZE | STRENGTHS |
| --- | --- | --- | --- |
| google/pegasus-xsum | 568 | 512 | Efficient summarization |
