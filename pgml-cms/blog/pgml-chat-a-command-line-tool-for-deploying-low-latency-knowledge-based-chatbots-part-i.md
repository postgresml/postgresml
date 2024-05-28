---
description: >-
  pgml-chat: A command-line tool for deploying low-latency knowledge-based
  chatbots
---

# pgml-chat: A command-line tool for deploying low-latency knowledge-based chatbots

<div align="left">

<figure><img src=".gitbook/assets/santi.jpg" alt="Author" width="158"><figcaption></figcaption></figure>

</div>

Santi Adavani

August 17, 2023

## Introduction

Chatbots powered by large language models like GPT-4 seem amazingly smart at first. They can have conversations on almost any topic. But chatbots have a huge blindspot - no long-term memory. Ask them about current events from last week or topics related to your specific business, and they just draw a blank.

To be truly useful for real applications, chatbots need fast access to knowledge - almost like human memory. Without quick recall, conversations become frustratingly slow and limited. It's like chatting with someone suffering from short-term memory loss.

Open source tools like LangChain and LlamaIndex are trying to help by giving chatbots more context and knowledge to work with. But behind the scenes, these tools end up gluing together many complex components into a patchwork. This adds lots of infrastructure overhead, ongoing maintenance needs, and results in slow response times that hurt chatbot performance.

Under the hood, these tools need to connect:

* A document storage system like MongoDB to house all the knowledge
* External machine learning service like Hugging Face or OpenAI to generate semantic embeddings
* A specialized vector database like Pinecone to index those embeddings for quick search

Managing and querying across so many moving parts introduces latency at each step. It's like passing ingredients from one sous chef to another in a busy kitchen. This assembled patchwork of services struggles to inject knowledge at the millisecond speeds required for smooth natural conversations.

We need a better foundational solution tailored specifically for chatbots - one that tightly integrates knowledge ingestion, analysis and retrieval under one roof. This consolidated architecture would provide the low latency knowledge lookups that chatbots desperately need.

In this blog series, we will explore PostgresML to do just that. In the first part, we will talk about deploying a chatbot using `pgml-chat` command line tool built on top of PostgresML. We will compare PostgresML query performance with a combination of Hugging Face and Pinecone. In the second part, we will show how `pgml-chat` works under the hood and focus on achieving low-latencies.

## Steps to build a chatbot on your own data

Similar to building and deploying machine learning models, building a chatbot involves steps that are both offline and online. The offline steps are compute-intensive and need to be done periodically when the data changes or the chatbot performance has deteriorated. The online steps are fast and need to be done in real-time. Below, we describe the steps in detail.

### 1. Building the Knowledge Base

This offline setup lays the foundation for your chatbot's intelligence. It involves:

1. Gathering domain documents like articles, reports, and websites to teach your chatbot about the topics it will encounter.
2. Splitting these documents into smaller chunks using different splitter algorithms. This keeps each chunk within the context size limits of AI models. In addition, it allows for chunking strategies that are tailored to the file type (e.g. PDFs, HTML, .py etc.).
3. Generating semantic embeddings for each chunk using deep learning models like SentenceTransformers. The embeddings capture conceptual meaning.
4. Indexing the chunk embeddings for efficient similarity search during conversations.

This knowledge base setup powers the contextual understanding for your chatbot. It's compute-intensive but only needs to be peridocially updated as your domain knowledge evolves.

### 2. Connecting to Conversational AI

With its knowledge base in place, now the chatbot links to models that allow natural conversations:

1. Based on users' questions, querying the indexed chunks to rapidly pull the most relevant passages.
2. Passing those passages to a model like GPT-3 to generate conversational responses.
3. Orchestrating the query, retrieval and generation flow to enable real-time chat.

### 3. Evaluating and Fine-tuning the chatbot

The chatbot needs to be evaluated and fine-tuned before it can be deployed to the real world. This involves:

1. Experimenting with different prompts and selecting the one that generates the best responses for a suite of questions.
2. Evaluating the chatbot's performance on a test set of questions by comparing the chatbot's responses to the ground truth responses.
3. If the performance is not satisfactory then we need to go to step 1 and generate embeddings using a different model. This is because the embeddings are the foundation of the chatbot's intelligence to get the most relevant passage from the knowledge base.

### 4. Connecting to the Real World

Finally, the chatbot needs to be deployed to the real world. This involves:

1. Identifying the interface that the users will interact with. This can be Slack, Discord, Teams or your own custom chat platform. Once identified get the API keys for the interface.
2. Hosting a chatbot service that can serve multiple users.
3. Integrating the chatbot service with the interface so that it can receive and respond to messages.

## pgml-chat

`pgml-chat` is a command line tool that allows you to do the following:

* Build a knowledge base that involves:
  * Ingesting documents into the database
  * Chunking documents and storing these chunks in the database
  * Generating embeddings and storing them in the database
  * Indexing embeddings for fast query
* Experimenting with prompts that can be passed to chat completion models like OpenAI's GPT-3 or GPT-4 or Meta's Llama2 models
* Experimenting with embeddings models that can be used to generate embeddings for the knowledge base
* Provides a chat interface at command line to evaluate your setup
* Runs Slack or Discord chat services so that your users can interact with your chatbot.

### Getting Started

Before you begin, make sure you have the following:

* PostgresML Database: Sign up for a free [GPU-powered database](https://postgresml.org/signup)
* Python version >=3.8
* OpenAI API key

1. Create a virtual environment and install `pgml-chat` using `pip`:

!!! code\_block

```bash
pip install pgml-chat
```

!!!

`pgml-chat` will be installed in your virtual environment's PATH.

2. Download `.env.template` file from PostgresML Github repository and make a copy.

!!! code\_block

```bash
wget https://raw.githubusercontent.com/postgresml/postgresml/master/pgml-apps/pgml-chat/.env.template
cp .env.template .env
```

!!!

3. Update environment variables with your OpenAI API key and PostgresML database credentials.

!!! code\_block

```bash
OPENAI_API_KEY=<OPENAI_API_KEY>
DATABASE_URL=<POSTGRES_DATABASE_URL starts with postgres://>
MODEL=Alibaba-NLP/gte-base-en-v1.5
SYSTEM_PROMPT=<> # System prompt used for OpenAI chat completion
BASE_PROMPT=<> # Base prompt used for OpenAI chat completion for each turn
SLACK_BOT_TOKEN=<SLACK_BOT_TOKEN> # Slack bot token to run Slack chat service
SLACK_APP_TOKEN=<SLACK_APP_TOKEN> # Slack app token to run Slack chat service
DISCORD_BOT_TOKEN=<DISCORD_BOT_TOKEN> # Discord bot token to run Discord chat service
```

!!!

### Usage

You can get help on the command line interface by running:

!!! code\_block

```bash
(pgml-bot-builder-py3.9) pgml-chat % pgml-chat --help
usage: pgml-chat [-h] --collection_name COLLECTION_NAME [--root_dir ROOT_DIR] [--stage {ingest,chat}] [--chat_interface {cli, slack, discord}]

PostgresML Chatbot Builder

optional arguments:
  -h, --help            show this help message and exit
  --collection_name COLLECTION_NAME
                        Name of the collection (schema) to store the data in PostgresML database (default: None)
  --root_dir ROOT_DIR   Input folder to scan for markdown files. Required for ingest stage. Not required for chat stage (default: None)
  --stage {ingest,chat}
                        Stage to run (default: chat)
  --chat_interface {cli, slack, discord}
                        Chat interface to use (default: cli)
```

!!!

### 1. Building the Knowledge Base

In this step, we ingest documents, chunk documents, generate embeddings and index these embeddings for fast query.

!!! code\_block

```bash
LOG_LEVEL=DEBUG pgml-chat --root_dir <directory> --collection_name <collection_name> --stage ingest
```

!!!

You will see the following output:

!!! code\_block

```bash
[15:39:12] DEBUG    [15:39:12] - Using selector: KqueueSelector 
           INFO     [15:39:12] - Starting pgml_chatbot           
           INFO     [15:39:12] - Scanning <root directory> for markdown files
[15:39:13] INFO     [15:39:13] - Found 85 markdown files 
Extracting text from markdown ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ 100% 0:00:00
           INFO     [15:39:13] - Upserting documents into database                                      
[15:39:32] INFO     [15:39:32] - Generating chunks       
[15:39:33] INFO     [15:39:33] - Starting chunk count: 0            
[15:39:35] INFO     [15:39:35] - Ending chunk count: 576                                                  
           INFO     [15:39:35] - Total documents: 85 Total chunks: 576                                                                                            
           INFO     [15:39:35] - Generating embeddings           
[15:39:36] INFO     [15:39:36] - Splitter ID: 2                                                                
[15:40:47] INFO     [15:40:47] - Embeddings generated in 71.073 seconds                
```

!!!

**Root directory** is where you have all your documentation that you would like the chatbot to be aware of.

!!! note

In the current version, we only support markdown files. We will be adding support for other file types soon.

!!!

**Collection name** is the name of the schema in the PostgresML database where the data will be stored. If the schema does not exist, it will be created.

**LOG\_LEVEL** will set the log level for the application. The default is `ERROR`. You can set it to `DEBUG` to see more detailed logs.

### 2. Connecting to Conversational AI

Here we will show how to experiment with prompts for the chat completion model to generate responses. We will use OpenAI `gpt-3.5-turbo` for chat completion. You need an [OpenAI API key](https://platform.openai.com/account/api-keys) to run this step.

You can provide the bot with a name and style of response using `SYSTEM_PROMPT` and `BASE_PROMPT` environment variables. The bot will then generate a response based on the user's question, context from vector search and the prompt. For the bot we built for PostgresML, we used the following system prompt. You can change the name of the bot, location and the name of the topics it will answer questions about.

!!! code\_block

```bash
SYSTEM_PROMPT="You are an assistant to answer questions about an open source software named PostgresML. Your name is PgBot. You are based out of San Francisco, California."
```

!!!

We used the following base prompt for the bot. Note that the prompt is a formatted string with placeholders for the `{context}` and the `{question}`. The chat service will replace these placeholders with the context and the question before passing it to the chat completion model. You can tune this prompt to get the best responses for your chatbot. In addition, you can update the email address and the support link to your own.

!!! code\_block

```bash
BASE_PROMPT="Given relevant parts of a document and a question, create a final answer.\ 
                Include a SQL query in the answer wherever possible. \
                Use the following portion of a long document to see if any of the text is relevant to answer the question.\
                \nReturn any relevant text verbatim.\n{context}\nQuestion: {question}\n \
                If the context is empty then ask for clarification and suggest user to send an email to team@postgresml.org or join PostgresML [Discord](https://discord.gg/DmyJP3qJ7U)."
```

!!!

### 3. Evaluating and Fine-tuning chatbot

Here we will show how to evaluate the chatbot's performance using the `cli` chat interface. This step will help you experiment with different prompts without spinning up a chat service. You can increase the log level to ERROR to suppress the logs from pgml-chat and OpenAI chat completion service.

!!! code\_block

```bash
LOG_LEVEL=ERROR pgml-chat --collection_name <collection_name> --stage chat --chat_interface cli
```

!!!

You should be able to interact with the bot as shown below. Control-C to exit.

!!! code\_block

```bash
User (Ctrl-C to exit): Who are you?
PgBot: I am PgBot, an AI assistant here to answer your questions about PostgresML, an open source software. How can I assist you today?
User (Ctrl-C to exit): What is PostgresML?
Found relevant documentation.... 
PgBot: PostgresML is an open source software that allows you to unlock the full potential of your data and drive more sophisticated insights and decision-making processes. It provides a dashboard with analytical views of the training data and 
model performance, as well as integrated notebooks for rapid iteration. PostgresML is primarily written in Rust using Rocket as a lightweight web framework and SQLx to interact with the database.

If you have any further questions or need more information, please feel free to send an email to team@postgresml.org or join the PostgresML Discord community at https://discord.gg/DmyJP3qJ7U.
```

!!!

To test with a new prompt, stop the chatbot using Control-C and update the `SYSTEM_PROMPT` and `BASE_PROMPT` environment variables. Then run the chatbot again.

If the responses are not acceptible, then increase the LOG\_LEVEL to check for the context that is being sent to chat completion. If the context is not satisfactory then you need to go back to step 1 and generate embeddings using a different model. This is because the embeddings are the foundation of the chatbot's intelligence to get the most relevant passage from the knowledge base.

You can change the embeddings model using the environment variable `MODEL` in `.env` file. Some models like `hknulp/instructor-xl` also take an instruction to generate embeddings. You can change the instruction using the environment variable `MODEL_PARAMS`. You can also change the instruction for query embeddings using the environment variable `QUERY_PARAMS`.

### 4. Connecting to the Real World

Once you are comfortable with the chatbot's performance it is ready for connecting to the real world. Here we will show how to run the chatbot as a Slack or Discord service. You need to create a Slack or Discord app and get the bot token and app token to run the chat service. Under the hood we use [`slack-bolt`](https://slack.dev/bolt-python/concepts) and [`discord.py`](https://discordpy.readthedocs.io/en/stable/) libraries to run the chat services.

#### Slack

You need SLACK\_BOT\_TOKEN and SLACK\_APP\_TOKEN to run the chatbot on Slack. You can get these tokens by creating a Slack app. Follow the instructions [here](https://slack.dev/bolt-python/tutorial/getting-started) to create a Slack app.Include the following environment variables in your .env file:

!!! code\_block

```bash
SLACK_BOT_TOKEN=<SLACK_BOT_TOKEN>
SLACK_APP_TOKEN=<SLACK_APP_TOKEN>
```

!!!

In this step, we start chatting with the chatbot on Slack. You can increase the log level to ERROR to suppress the logs.

```bash
LOG_LEVEL=ERROR pgml-chat --collection_name <collection_name> --stage chat --chat_interface slack
```

If you have set up the Slack app correctly, you should see the following output:

```
⚡️ Bolt app is running!
```

Once the slack app is running, you can interact with the chatbot on Slack as shown below. In the example here, name of the bot is `PgBot`. This app responds only to direct messages to the bot.

<figure><img src=".gitbook/assets/image (5).png" alt=""><figcaption></figcaption></figure>

#### Discord

You need DISCORD\_BOT\_TOKEN to run the chatbot on Discord. You can get this token by creating a Discord app. Follow the instructions [here](https://discordpy.readthedocs.io/en/stable/discord.html) to create a Discord app. Include the following environment variables in your .env file:

```bash
DISCORD_BOT_TOKEN=<DISCORD_BOT_TOKEN>
```

In this step, we start chatting with the chatbot on Discord. You can increase the log level to ERROR to suppress the logs.

```bash
pgml-chat --collection_name <collection_name> --stage chat --chat_interface discord
```

If you have set up the Discord app correctly, you should see the following output:

```bash
2023-08-02 16:09:57 INFO     discord.client logging in using static token
```

Once the discord app is running, you can interact with the chatbot on Discord as shown below. In the example here, name of the bot is `pgchat`. This app responds only to direct messages to the bot.

<figure><img src=".gitbook/assets/image (6).png" alt=""><figcaption></figcaption></figure>

### PostgresML vs. Hugging Face + Pinecone

To evaluate query latency, we performed an experiment with 10,000 Wikipedia documents from the SQuAD dataset. Embeddings were generated using the Alibaba-NLP/gte-base-en-v1.5 model.

For PostgresML, we used a GPU-powered serverless database running on NVIDIA A10G GPUs with client in us-west-2 region. For HuggingFace, we used their inference API endpoint running on NVIDIA A10G GPUs in us-east-1 region and a client in the same us-east-1 region. Pinecone was used as the vector search index for HuggingFace embeddings.

By keeping the document dataset, model, and hardware constant, we aimed to evaluate the performance of the two systems independently. Care was taken to eliminate network latency as a factor - HuggingFace endpoint and client were co-located in us-east-1, while PostgresML database and client were co-located in us-west-2.

Our experiments found that PostgresML outperformed HuggingFace + Pinecone in query latency by \~4x. Mean latency was 59ms for PostgresML and 233ms for HuggingFace + Pinecone. Query latency was averaged across 100 queries to account for any outliers. This \~4x improvement in mean latency can be attributed to PostgresML's tight integration of embedding generation, indexing, and querying within the database running on NVIDIA A10G GPUs.

For applications like chatbots that require low latency access to knowledge, PostgresML provides superior performance over combining multiple services. The serverless architecture also provides predictable pricing and scales seamlessly with usage.

<figure><img src=".gitbook/assets/image (7).png" alt=""><figcaption></figcaption></figure>

## Conclusions

In this post, we announced PostgresML Chatbot Builder - an open source tool that makes it easy to build knowledge based chatbots. We discussed the effort required to integrate various components like ingestion, embedding generation, indexing etc. and how PostgresML Chatbot Builder automates this end-to-end workflow.

We also presented some initial benchmark results comparing PostgresML and HuggingFace + Pinecone for query latency using the SQuAD dataset. PostgresML provided up to \~4x lower latency thanks to its tight integration and optimizations.

Stay tuned for part 2 of this benchmarking blog post where we will present more comprehensive results evaluating performance for generating embeddings with different models and batch sizes. We will also share additional query latency benchmarks with more document collections.
