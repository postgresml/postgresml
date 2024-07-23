import asyncio
from pgml import (
    Collection,
    Model,
    Splitter,
    Pipeline,
    migrate,
    init_logger,
    Builtins,
    OpenSourceAI,
)
import logging
from rich.logging import RichHandler
from rich.progress import track
from rich import print
import os
from dotenv import load_dotenv
import glob
import argparse
from time import time
from openai import OpenAI
import signal
from uuid import uuid4
import pendulum

import ast
from slack_bolt.async_app import AsyncApp
from slack_bolt.adapter.socket_mode.aiohttp import AsyncSocketModeHandler
import requests

import discord


def handler(signum, frame):
    print("Exiting...")
    exit(0)


signal.signal(signal.SIGINT, handler)

parser = argparse.ArgumentParser(
    description="PostgresML Chatbot Builder",
    formatter_class=argparse.ArgumentDefaultsHelpFormatter,
)
parser.add_argument(
    "--collection_name",
    dest="collection_name",
    type=str,
    help="Name of the collection (schema) to store the data in PostgresML database",
    required=True,
)
parser.add_argument(
    "--root_dir",
    dest="root_dir",
    type=str,
    help="Input folder to scan for markdown files. Required for ingest stage. Not required for chat stage",
)
parser.add_argument(
    "--stage",
    dest="stage",
    choices=["ingest", "chat"],
    type=str,
    default="chat",
    help="Stage to run",
)
parser.add_argument(
    "--chat_interface",
    dest="chat_interface",
    choices=["cli", "slack", "discord"],
    type=str,
    default="cli",
    help="Chat interface to use",
)

parser.add_argument(
    "--chat_history",
    dest="chat_history",
    type=int,
    default=0,
    help="Number of messages from history used for generating response",
)

parser.add_argument(
    "--bot_name",
    dest="bot_name",
    type=str,
    default="PgBot",
    help="Name of the bot",
)

parser.add_argument(
    "--bot_language",
    dest="bot_language",
    type=str,
    default="English",
    help="Language of the bot",
)

parser.add_argument(
    "--bot_topic",
    dest="bot_topic",
    type=str,
    default="PostgresML",
    help="Topic of the bot",
)
parser.add_argument(
    "--bot_topic_primary_language",
    dest="bot_topic_primary_language",
    type=str,
    default="SQL",
    help="Primary programming language of the topic",
)

parser.add_argument(
    "--bot_persona",
    dest="bot_persona",
    type=str,
    default="Engineer",
    help="Persona of the bot",
)

parser.add_argument(
    "--chat_completion_model",
    dest="chat_completion_model",
    type=str,
    default="meta-llama/Meta-LLama-3.1-8B-Instruct",
)

parser.add_argument(
    "--max_tokens",
    dest="max_tokens",
    type=int,
    default=256,
    help="Maximum number of tokens to generate",
)

parser.add_argument(
    "--temperature",
    dest="temperature",
    type=float,
    default=0.7,
    help="Temperature for generating response",
)

parser.add_argument(
    "--top_p",
    dest="top_p",
    type=float,
    default=0.9,
    help="Top p for generating response",
)
parser.add_argument(
    "--vector_recall_limit",
    dest="vector_recall_limit",
    type=int,
    default=1,
    help="Maximum number of documents to retrieve from vector recall",
)

args = parser.parse_args()

FORMAT = "%(message)s"
logging.basicConfig(
    level=os.environ.get("LOG_LEVEL", "ERROR"),
    format="%(asctime)s - %(message)s",
    datefmt="[%X]",
    handlers=[RichHandler()],
)
log = logging.getLogger("rich")

# Load .env file
load_dotenv(".env")


# The code is using the `argparse` module to parse command line arguments.
chat_history_collection_name = args.collection_name + "_chat_history"
collection = Collection(args.collection_name)
chat_collection = Collection(chat_history_collection_name)
stage = args.stage
chat_interface = args.chat_interface
chat_history = args.chat_history

# Get all bot related environment variables
bot_name = args.bot_name
bot_language = args.bot_language
bot_persona = args.bot_persona
bot_topic = args.bot_topic
bot_topic_primary_language = args.bot_topic_primary_language

# The above code is retrieving environment variables and assigning their values to various variables.
database_url = os.environ.get("DATABASE_URL")
splitter_name = os.environ.get("SPLITTER", "recursive_character")
splitter_params = os.environ.get(
    "SPLITTER_PARAMS", {"chunk_size": 1500, "chunk_overlap": 40}
)

splitter = Splitter(splitter_name, splitter_params)
model_name = "Alibaba-NLP/gte-base-en-v1.5"
model_params = {}

model = Model(model_name, "pgml", model_params)
pipeline = Pipeline(args.collection_name + "_pipeline", model, splitter)
chat_history_pipeline = Pipeline(
    chat_history_collection_name + "_pipeline", model, splitter
)

chat_completion_model = args.chat_completion_model
max_tokens = args.max_tokens
temperature = args.temperature
vector_recall_limit = args.vector_recall_limit

query_params_instruction = (
    "Represent the %s question for retrieving supporting documents: " % (bot_topic)
)
query_params = {"instruction": query_params_instruction}

default_system_prompt_template = """Use the following pieces of context to answer the question at the end.
If you don't know the answer, just say that you don't know, don't try to make up an answer.
Use three sentences maximum and keep the answer as concise as possible.
Always say "thanks for asking!" at the end of the answer."""

system_prompt_template = os.environ.get("SYSTEM_PROMPT_TEMPLATE", default_system_prompt_template)

system_prompt = system_prompt_template.format(
    topic=bot_topic,
    name=bot_name,
    persona=bot_persona,
    language=bot_language,
    response_programming_language=bot_topic_primary_language,
)

base_prompt = """
{conversation_history}
####
Documents
####
{context}
###
User: {question}
###

Helpful Answer:"""


openai_api_key = os.environ.get("OPENAI_API_KEY","")

system_prompt_document = [
    {
        "text": system_prompt,
        "id": str(uuid4())[:8],
        "interface": chat_interface,
        "role": "system",
        "timestamp": pendulum.now().timestamp(),
    }
]


def get_model_type(chat_completion_model: str):
    model_type = "opensourceai"
    try:
        client = OpenAI(api_key=openai_api_key)
        models = client.models.list()
        for model in models:
            if model.id == chat_completion_model:
                model_type = "openai"
                break
    except Exception as e:
        log.debug(e)

    log.info("Setting model type to " + model_type)

    return model_type


async def upsert_documents(folder: str) -> int:
    log.info("Scanning " + folder + " for markdown files")
    md_files = []
    # root_dir needs a trailing slash (i.e. /root/dir/)
    for filename in glob.iglob(folder + "/**/*.md", recursive=True):
        md_files.append(filename)

    log.info("Found " + str(len(md_files)) + " markdown files")
    documents = []
    for md_file in track(md_files, description="Extracting text from markdown"):
        with open(md_file, "r") as f:
            documents.append({"text": f.read(), "id": md_file})

    log.info("Upserting documents into database")
    await collection.upsert_documents(documents)

    return len(md_files)


async def generate_chat_response(
    user_input,
    system_prompt,
    openai_api_key,
    temperature=temperature,
    max_tokens=max_tokens,
    top_p=0.9,
    user_name="",
):
    messages = []
    messages.append({"role": "system", "content": system_prompt})

    chat_history_messages = await chat_collection.get_documents(
        {
            "limit": chat_history * 2,
            "order_by": {"timestamp": "desc"},
            "filter": {
                "metadata": {
                    "$and": [
                        {
                            "$or": [
                                {"role": {"$eq": "assistant"}},
                                {"role": {"$eq": "user"}},
                            ]
                        },
                        {"interface": {"$eq": chat_interface}},
                        {"user_name": {"$eq": user_name}},
                    ]
                }
            },
        }
    )

    # Reverse the order so that user messages are first

    chat_history_messages.reverse()

    conversation_history = ""
    for entry in chat_history_messages:
        document = entry["document"]
        if document["role"] == "user":
            conversation_history += "User: " + document["text"] + "\n"
        if document["role"] == "assistant":
            conversation_history += "Assistant: " + document["text"] + "\n"

    log.info(conversation_history)

    history_documents = []
    user_message_id = str(uuid4())[:8]
    _document = {
        "text": user_input,
        "id": user_message_id,
        "interface": chat_interface,
        "role": "user",
        "timestamp": pendulum.now().timestamp(),
        "user_name": user_name,
    }
    history_documents.append(_document)


    query = await get_prompt(user_input, conversation_history)

    messages.append({"role": "user", "content": query})

    log.info(messages)

    response = await generate_response(
        messages,
        openai_api_key,
        max_tokens=max_tokens,
        temperature=temperature,
        top_p=top_p,
    )

    _document = {
        "text": response,
        "id": str(uuid4())[:8],
        "parent_message_id": user_message_id,
        "interface": chat_interface,
        "role": "assistant",
        "timestamp": pendulum.now().timestamp(),
        "user_name": user_name,
    }
    history_documents.append(_document)

    await chat_collection.upsert_documents(history_documents)

    return response


async def generate_response(
    messages, openai_api_key, temperature=temperature, max_tokens=max_tokens, top_p=0.9
):
    model_type = get_model_type(chat_completion_model)
    if model_type == "openai":
        client = OpenAI(api_key=openai_api_key)
        log.debug("Generating response from OpenAI API: " + str(messages))
        response = client.chat.completions.create(
            model=chat_completion_model,
            messages=messages,
            temperature=temperature,
            max_tokens=max_tokens,
            top_p=top_p,
            frequency_penalty=0,
            presence_penalty=0,
        )
        output = response.choices[0].message.content
    else:
        client = OpenSourceAI(database_url=database_url)
        log.debug("Generating response from OpenSourceAI API: " + str(messages))
        response = client.chat_completions_create(
            model=chat_completion_model,
            messages=messages,
            temperature=temperature,
            max_tokens=max_tokens,
        )
        output = response["choices"][0]["message"]["content"]

    return output


async def ingest_documents(folder: str):
    # Add the pipeline to the collection, does nothing if we have already added it
    await collection.add_pipeline(pipeline)
    await chat_collection.add_pipeline(chat_history_pipeline)
    # This will upsert, chunk, and embed the contents in the folder
    total_docs = await upsert_documents(folder)
    log.info("Total documents: " + str(total_docs))


async def get_prompt(user_input: str = "", conversation_history: str = "") -> str:
    query_input = "In the context of " + bot_topic + ", " + user_input
    vector_results = (
        await collection.query()
        .vector_recall(query_input, pipeline, query_params)
        .limit(vector_recall_limit)
        .fetch_all()
    )
    log.info(vector_results)

    context = ""
    for id, result in enumerate(vector_results):
        if result[0] > 0.6:
            context += "#### \n Document %d: " % (id) + result[1] + "\n"

    if conversation_history:
        conversation_history = "#### \n Conversation History: \n" + conversation_history

    query = base_prompt.format(
        conversation_history=conversation_history,
        context=context,
        question=user_input,
        topic=bot_topic,
        persona=bot_persona,
        language=bot_language,
        response_programming_language=bot_topic_primary_language,
    )

    return query


async def chat_cli():
    user_name = os.environ.get("USER")
    while True:
        try:
            user_input = input("User (Ctrl-C to exit): ")
            response = await generate_chat_response(
                user_input,
                system_prompt,
                openai_api_key,
                max_tokens=max_tokens,
                temperature=temperature,
                top_p=0.9,
                user_name=user_name,
            )
            print("PgBot: " + response)
        except KeyboardInterrupt:
            print("Exiting...")
            break


async def chat_slack():
    if os.environ.get("SLACK_BOT_TOKEN") and os.environ.get("SLACK_APP_TOKEN"):
        app = AsyncApp(token=os.environ.get("SLACK_BOT_TOKEN"))
        response = requests.post(
            "https://slack.com/api/auth.test",
            headers={"Authorization": "Bearer " + os.environ.get("SLACK_BOT_TOKEN")},
        )
        bot_user_id = response.json()["user_id"]

        @app.message(f"<@{bot_user_id}>")
        async def message_hello(message, say):
            print("Message received... ")
            user_input = message["text"]
            user = message["user"]
            response = await generate_chat_response(
                user_input,
                system_prompt,
                openai_api_key,
                max_tokens=max_tokens,
                temperature=temperature,
                user_name=user,
            )

            await say(text=f"<@{user}> {response}")

        socket_handler = AsyncSocketModeHandler(app, os.environ["SLACK_APP_TOKEN"])
        await socket_handler.start_async()
    else:
        log.error(
            "SLACK_BOT_TOKEN and SLACK_APP_TOKEN environment variables are not found. Exiting..."
        )


intents = discord.Intents.default()
intents.message_content = True
client = discord.Client(intents=intents)


@client.event
async def on_ready():
    await chat_collection.upsert_documents(system_prompt_document)
    print(f"We have logged in as {client.user}")


@client.event
async def on_message(message):
    bot_mention = f"<@{client.user.id}>"
    if message.author != client.user and bot_mention in message.content:
        print("Discord response in progress ..")
        user_input = message.content
        response = await generate_chat_response(
            user_input,
            system_prompt,
            openai_api_key,
            max_tokens=max_tokens,
            temperature=temperature,
            user_name=message.author.name,
        )
        await message.channel.send(response)


async def run():
    """
    The `main` function connects to a database, ingests documents from a specified folder, generates
    chunks, and logs the total number of documents and chunks.
    """
    log.info("Starting pgml-chat.... ")
    await chat_collection.upsert_documents(system_prompt_document)
    # await migrate()
    if stage == "ingest":
        root_dir = args.root_dir
        await ingest_documents(root_dir)

    elif stage == "chat":
        if chat_interface == "cli":
            await chat_cli()
        elif chat_interface == "slack":
            await chat_slack()


def main():
    init_logger()
    if (
        stage == "chat"
        and chat_interface == "discord"
        and os.environ.get("DISCORD_BOT_TOKEN")
    ):
        client.run(os.environ["DISCORD_BOT_TOKEN"])
    else:
        asyncio.run(run())


main()
