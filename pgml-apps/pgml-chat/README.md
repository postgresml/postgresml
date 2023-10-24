# PostgresML Chatbot Builder
A command line tool to build and deploy a **_knowledge based_** chatbot using PostgresML and OpenAI API.

There are two stages in building a knowledge based chatbot:
- Build a knowledge base by ingesting documents, chunking documents, generating embeddings and indexing these embeddings for fast query
- Generate responses to user queries by retrieving relevant documents and generating responses using OpenAI API

This tool automates the above two stages and provides a command line interface to build and deploy a knowledge based chatbot.

# Prerequisites
Before you begin, make sure you have the following:

- PostgresML Database: Sign up for a free [GPU-powered database](https://postgresml.org/signup)
- Python version >=3.8
- OpenAI API key


# Getting started
1. Create a virtual environment and install `pgml-chat` using `pip`:
```bash
pip install pgml-chat
```

`pgml-chat` will be installed in your PATH.

2. Download `.env.template` file from PostgresML Github repository.

```bash
wget https://raw.githubusercontent.com/postgresml/postgresml/master/pgml-apps/pgml-chat/.env.template
```
3. Copy the template file to `.env`

4. Update environment variables with your OpenAI API key and PostgresML database credentials.
```bash
OPENAI_API_KEY=<OPENAI_API_KEY>
DATABASE_URL=<POSTGRES_DATABASE_URL starts with postgres://>
```

# Usage
You can get help on the command line interface by running:

```bash
(pgml-bot-builder-py3.9) pgml-chat % pgml-chat % pgml-chat --help
usage: pgml-chat [-h] --collection_name COLLECTION_NAME [--root_dir ROOT_DIR] [--stage {ingest,chat}] [--chat_interface {cli,slack,discord}]
                 [--chat_history CHAT_HISTORY] [--bot_name BOT_NAME] [--bot_language BOT_LANGUAGE] [--bot_topic BOT_TOPIC]
                 [--bot_topic_primary_language BOT_TOPIC_PRIMARY_LANGUAGE] [--bot_persona BOT_PERSONA]

PostgresML Chatbot Builder

optional arguments:
  -h, --help            show this help message and exit
  --collection_name COLLECTION_NAME
                        Name of the collection (schema) to store the data in PostgresML database (default: None)
  --root_dir ROOT_DIR   Input folder to scan for markdown files. Required for ingest stage. Not required for chat stage (default: None)
  --stage {ingest,chat}
                        Stage to run (default: chat)
  --chat_interface {cli,slack,discord}
                        Chat interface to use (default: cli)
  --chat_history CHAT_HISTORY
                        Number of messages from history used for generating response (default: 1)
  --bot_name BOT_NAME   Name of the bot (default: PgBot)
  --bot_language BOT_LANGUAGE
                        Language of the bot (default: English)
  --bot_topic BOT_TOPIC
                        Topic of the bot (default: PostgresML)
  --bot_topic_primary_language BOT_TOPIC_PRIMARY_LANGUAGE
                        Primary programming language of the topic (default: )
  --bot_persona BOT_PERSONA
                        Persona of the bot (default: Engineer)
```
## Ingest
In this step, we ingest documents, chunk documents, generate embeddings and index these embeddings for fast query.

```bash
LOG_LEVEL=DEBUG pgml-chat --root_dir <directory> --collection_name <collection_name> --stage ingest
```

You will see output logging the pipelines progress.

## Chat
You can interact with the bot using the command line interface or Slack. 

### Command Line Interface
In this step, we start chatting with the chatbot at the command line. You can increase the log level to ERROR to suppress the logs. CLI is the default chat interface.
    
```bash
LOG_LEVEL=ERROR pgml-chat --collection_name <collection_name> --stage chat --chat_interface cli
```

You should be able to interact with the bot as shown below. Control-C to exit.
```bash
User (Ctrl-C to exit): Who are you?
PgBot: I am PgBot, an AI assistant here to answer your questions about PostgresML, an open source software. How can I assist you today?
User (Ctrl-C to exit): What is PostgresML?
Found relevant documentation.... 
PgBot: PostgresML is an open source software that allows you to unlock the full potential of your data and drive more sophisticated insights and decision-making processes. It provides a dashboard with analytical views of the training data and 
model performance, as well as integrated notebooks for rapid iteration. PostgresML is primarily written in Rust using Rocket as a lightweight web framework and SQLx to interact with the database.

If you have any further questions or need more information, please feel free to send an email to team@postgresml.org or join the PostgresML Discord community at https://discord.gg/DmyJP3qJ7U.
```


### Slack

**Setup**
You need SLACK_BOT_TOKEN and SLACK_APP_TOKEN to run the chatbot on Slack. You can get these tokens by creating a Slack app. Follow the instructions [here](https://slack.dev/bolt-python/tutorial/getting-started) to create a Slack app.Include the following environment variables in your .env file:

```bash
SLACK_BOT_TOKEN=<SLACK_BOT_TOKEN>
SLACK_APP_TOKEN=<SLACK_APP_TOKEN>
```
In this step, we start chatting with the chatbot on Slack. You can increase the log level to ERROR to suppress the logs. 
```bash
LOG_LEVEL=ERROR pgml-chat --collection_name <collection_name> --stage chat --chat_interface slack
```
If you have set up the Slack app correctly, you should see the following output:

```
⚡️ Bolt app is running!
```

Once the slack app is running, you can interact with the chatbot on Slack as shown below. In the example here, name of the bot is `PgBot`. This app responds only to direct messages to the bot.

![Slack Chatbot](./images/slack_screenshot.png)


### Discord

**Setup**
You need DISCORD_BOT_TOKEN to run the chatbot on Discord. You can get this token by creating a Discord app. Follow the instructions [here](https://discordpy.readthedocs.io/en/stable/discord.html) to create a Discord app. Include the following environment variables in your .env file:

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

![Discord Chatbot](./images/discord_screenshot.png)

# Developer Guide

1. Clone this repository, start a poetry shell and install dependencies

```bash
git clone https://github.com/postgresml/postgresml
cd postgresml/pgml-apps/pgml-chat
poetry shell
poetry install
pip install .
```

2. Create a .env file in the root directory of the project and add all the environment variables discussed in [Getting Started](#getting-started) section.
3. All the logic is in `pgml_chat/main.py`
4. Check the [roadmap](#roadmap) for features that you would like to work on.
5. If you are looking for features that are not included here, please open an issue and we will add it to the roadmap.



# Roadmap
- ~~Use a collection for chat history that can be retrieved and used to generate responses.~~
- Support for file formats like rst, html, pdf, docx, etc.
- Support for open source models in addition to OpenAI for chat completion.
- Support for multi-turn converstaions using converstaion buffer. 
