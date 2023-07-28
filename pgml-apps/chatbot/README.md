# PostgresML Chatbot Builder
A command line tool to build and deploy a **_knowledge based_** chatbot using PostgresML and OpenAI API.

There are two stages in building a knowledge based chatbot:
- Build a knowledge base by ingesting documents, chunking documents, generating embeddings and indexing these embeddings for fast query
- Generate responses to user queries by retrieving relevant documents and generating responses using OpenAI API

This tool automates the above two stages and provides a command line interface to build and deploy a knowledge based chatbot.

## Prerequisites
Before you begin, make sure you have the following:

- PostgresML Database: You can spin up a database using [Docker](https://github.com/postgresml/postgresml#installation) or [sign-up](https://postgresml.org/signup) for a free GPU-powered database. 
- Python version >=3.8
- OpenAI API key
- Python 3.8+
- Poetry

## Getting started
1. Clone this repository, start a poetry shell and install dependencies
```bash
git clone https://github.com/postgresml/postgresml
cd postgresml/pgml-apps/chatbot
poetry shell
poetry install
```

2. Update environment variables in `.env` file
```bash
cp .env.template .env
```

Update environment variables with your OpenAI API key and PostgresML database credentials.
```bash
OPENAI_API_KEY=<OPENAI_API_KEY>
DATABASE_URL=<POSTGRES_DATABASE_URL starts with postgres://>
MODEL=hkunlp/instructor-xl
MODEL_PARAMS={"instruction": "Represent the Wikipedia document for retrieval: "}
QUERY_PARAMS={"instruction": "Represent the Wikipedia question for retrieving supporting documents: "}
SYSTEM_PROMPT="You are an assistant to answer questions about an open source software named PostgresML. Your name is PgBot. You are based out of San Francisco, California."
BASE_PROMPT="Given relevant parts of a document and a question, create a final answer.\ 
                Include a SQL query in the answer wherever possible. \
                Use the following portion of a long document to see if any of the text is relevant to answer the question.\
                \nReturn any relevant text verbatim.\n{context}\nQuestion: {question}\n \
                If the context is empty then ask for clarification and suggest user to send an email to team@postgresml.org or join PostgresML [Discord](https://discord.gg/DmyJP3qJ7U)."
```

## Usage
You can get help on the command line interface by running:

```bash
(pgml-bot-builder-py3.9) chatbot % python pgml_chatbot/main.py --help
usage: main.py [-h] [--root_dir ROOT_DIR] [--collection_name COLLECTION_NAME] [--stage {ingest,chat}]

Process some integers.

optional arguments:
  -h, --help            show this help message and exit
  --root_dir ROOT_DIR   Input folder to scan for markdown files
  --collection_name COLLECTION_NAME
                        Name of the collection to store the data in
  --stage {ingest,chat}
                        Stage to run
```
### Ingest
In this step, we ingest documents, chunk documents, generate embeddings and index these embeddings for fast query.

```bash
python pgml_chatbot/main.py --root_dir <directory> --collection_name <collection_name> --stage ingest
```

### Chat
In this step, we start chatting with the chatbot at the command line. You can increase the log level to ERROR to suppress the logs.
    
```bash
LOG_LEVEL=ERROR python pgml_chatbot/main.py --root_dir <directory> --collection_name <collection_name> --stage chat
```

You should be able to interact with the bot as shown below:
```bash
User (Ctrl-C to exit): Who are you?
PgBot: I am PgBot, an AI assistant here to answer your questions about PostgresML, an open source software. How can I assist you today?
User (Ctrl-C to exit): What is PostgresML?
Found relevant documentation.... 
PgBot: PostgresML is an open source software that allows you to unlock the full potential of your data and drive more sophisticated insights and decision-making processes. It provides a dashboard with analytical views of the training data and 
model performance, as well as integrated notebooks for rapid iteration. PostgresML is primarily written in Rust using Rocket as a lightweight web framework and SQLx to interact with the database.

If you have any further questions or need more information, please feel free to send an email to team@postgresml.org or join the PostgresML Discord community at https://discord.gg/DmyJP3qJ7U.
```

## Options
You can control the behavior of the chatbot by setting the following environment variables:
- `SYSTEM_PROMPT`: This is the prompt that is used to initialize the chatbot. You can customize this prompt to change the behavior of the chatbot. For example, you can change the name of the chatbot or the location of the chatbot.
- `BASE_PROMPT`: This is the prompt that is used to generate responses to user queries. You can customize this prompt to change the behavior of the chatbot. 
- `MODEL`: This is the open source embedding model used to generate embeddings for the documents. You can change this to use a different model.