import asyncio
from pgml import Database
import logging
from rich.logging import RichHandler
from rich.progress import track
from rich import print
import os
from dotenv import load_dotenv
import glob
import argparse
from time import time
import openai
import signal

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
collection_name = args.collection_name
stage = args.stage
chat_interface = args.chat_interface

# The above code is retrieving environment variables and assigning their values to various variables.
database_url = os.environ.get("DATABASE_URL")
db = Database(database_url)
splitter = os.environ.get("SPLITTER", "recursive_character")
splitter_params = os.environ.get(
    "SPLITTER_PARAMS", {"chunk_size": 1500, "chunk_overlap": 40}
)
model = os.environ.get("MODEL", "intfloat/e5-small")
model_params = ast.literal_eval(os.environ.get("MODEL_PARAMS", {}))
query_params = ast.literal_eval(os.environ.get("QUERY_PARAMS", {}))
system_prompt = os.environ.get("SYSTEM_PROMPT")
base_prompt = os.environ.get("BASE_PROMPT")
openai_api_key = os.environ.get("OPENAI_API_KEY")


async def upsert_documents(db: Database, collection_name: str, folder: str) -> int:
    log.info("Scanning " + folder + " for markdown files")
    md_files = []
    # root_dir needs a trailing slash (i.e. /root/dir/)
    for filename in glob.iglob(folder + "/**/*.md", recursive=True):
        md_files.append(filename)

    log.info("Found " + str(len(md_files)) + " markdown files")
    documents = []
    for md_file in track(md_files, description="Extracting text from markdown"):
        with open(md_file, "r") as f:
            documents.append({"text": f.read(), "filename": md_file})

    log.info("Upserting documents into database")
    collection = await db.create_or_get_collection(collection_name)
    await collection.upsert_documents(documents)

    return len(md_files)


async def generate_chunks(
    db: Database,
    collection_name: str,
    splitter: str = "recursive_character",
    splitter_params: dict = {"chunk_size": 1500, "chunk_overlap": 40},
) -> int:
    """
    The function `generate_chunks` generates chunks for a given collection in a database and returns the
    count of chunks created.

    :param db: The `db` parameter is an instance of a database connection or client. It is used to
    interact with the database and perform operations such as creating collections, executing queries,
    and fetching results
    :type db: Database
    :param collection_name: The `collection_name` parameter is a string that represents the name of the
    collection in the database. It is used to create or get the collection and perform operations on it
    :type collection_name: str
    :return: The function `generate_chunks` returns an integer, which represents the count of chunks
    generated in the specified collection.
    """
    log.info("Generating chunks")
    collection = await db.create_or_get_collection(collection_name)
    splitter_id = await collection.register_text_splitter(splitter, splitter_params)
    query_string = """SELECT count(*) from {collection_name}.chunks""".format(
        collection_name=collection_name
    )
    results = await db.query(query_string).fetch_all()
    start_chunks = results[0]["count"]
    log.info("Starting chunk count: " + str(start_chunks))
    await collection.generate_chunks(splitter_id)
    results = await db.query(query_string).fetch_all()
    log.info("Ending chunk count: " + str(results[0]["count"]))
    return results[0]["count"] - start_chunks


async def generate_embeddings(
    db: Database,
    collection_name: str,
    splitter: str = "recursive_character",
    splitter_params: dict = {"chunk_size": 1500, "chunk_overlap": 40},
    model: str = "intfloat/e5-small",
    model_params: dict = {},
) -> int:
    """
    The `generate_embeddings` function generates embeddings for text data using a specified model and
    splitter.

    :param db: The `db` parameter is an instance of a database object. It is used to interact with the
    database and perform operations such as creating or getting a collection, registering a text
    splitter, registering a model, and generating embeddings
    :type db: Database
    :param collection_name: The `collection_name` parameter is a string that represents the name of the
    collection in the database where the embeddings will be generated
    :type collection_name: str
    :param splitter: The `splitter` parameter is used to specify the text splitting method to be used
    during the embedding generation process. In this case, the value is set to "recursive_character",
    which suggests that the text will be split into chunks based on recursive character splitting,
    defaults to recursive_character
    :type splitter: str (optional)
    :param splitter_params: The `splitter_params` parameter is a dictionary that contains the parameters
    for the text splitter. In this case, the `splitter_params` dictionary has two keys:
    :type splitter_params: dict
    :param model: The `model` parameter is the name or identifier of the language model that will be
    used to generate the embeddings. In this case, the model is specified as "intfloat/e5-small",
    defaults to intfloat/e5-small
    :type model: str (optional)
    :param model_params: The `model_params` parameter is a dictionary that allows you to specify
    additional parameters for the model. These parameters can be used to customize the behavior of the
    model during the embedding generation process. The specific parameters that can be included in the
    `model_params` dictionary will depend on the specific model you are
    :type model_params: dict
    :return: an integer value of 0.
    """
    log.info("Generating embeddings")
    collection = await db.create_or_get_collection(collection_name)
    splitter_id = await collection.register_text_splitter(splitter, splitter_params)
    model_id = await collection.register_model("embedding", model, model_params)
    log.info("Splitter ID: " + str(splitter_id))
    start = time()
    await collection.generate_embeddings(model_id, splitter_id)
    log.info("Embeddings generated in %0.3f seconds" % (time() - start))

    return 0


async def generate_response(
    messages, openai_api_key, temperature=0.7, max_tokens=256, top_p=0.9
):
    openai.api_key = openai_api_key
    response = openai.ChatCompletion.create(
        model="gpt-3.5-turbo",
        messages=messages,
        temperature=temperature,
        max_tokens=max_tokens,
        top_p=top_p,
        frequency_penalty=0,
        presence_penalty=0,
    )
    return response["choices"][0]["message"]["content"]


async def ingest_documents(
    db: Database,
    collection_name: str,
    folder: str,
    splitter: str,
    splitter_params: dict,
    model: str,
    model_params: dict,
):
    total_docs = await upsert_documents(db, collection_name, folder=folder)
    total_chunks = await generate_chunks(
        db, collection_name, splitter=splitter, splitter_params=splitter_params
    )
    log.info(
        "Total documents: " + str(total_docs) + " Total chunks: " + str(total_chunks)
    )

    await generate_embeddings(
        db,
        collection_name,
        splitter=splitter,
        splitter_params=splitter_params,
        model=model,
        model_params=model_params,
    )


async def get_prompt(user_input: str = ""):
    collection = await db.create_or_get_collection(collection_name)
    model_id = await collection.register_model("embedding", model, model_params)
    splitter_id = await collection.register_text_splitter(splitter, splitter_params)
    log.info("Model id: " + str(model_id) + " Splitter id: " + str(splitter_id))
    vector_results = await collection.vector_search(
        user_input,
        model_id=model_id,
        splitter_id=splitter_id,
        top_k=2,
        query_params=query_params,
    )
    log.info(vector_results)
    context = ""

    for result in vector_results:
        context += result[1] + "\n"

    query = base_prompt.format(context=context, question=user_input)

    return query


async def chat_cli():
    user_input = "Who are you?"
    while True:
        try:
            messages = [{"role": "system", "content": system_prompt}]
            if user_input:
                query = await get_prompt(user_input)
            messages.append({"role": "user", "content": query})
            response = await generate_response(
                messages, openai_api_key, max_tokens=512, temperature=0.0
            )
            print("PgBot: " + response)

            user_input = input("User (Ctrl-C to exit): ")
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
            messages = [{"role": "system", "content": system_prompt}]
            user_input = message["text"]

            query = await get_prompt(user_input)
            messages.append({"role": "user", "content": query})
            response = await generate_response(
                messages, openai_api_key, max_tokens=512, temperature=1.0
            )
            user = message["user"]

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
    print(f"We have logged in as {client.user}")

@client.event
async def on_message(message):
    bot_mention = f"<@{client.user.id}>"
    messages = [{"role": "system", "content": system_prompt}]
    if message.author != client.user and bot_mention in message.content:
        print("Discord response in progress ..")
        user_input = message.content
        query = await get_prompt(user_input)

        messages.append({"role": "user", "content": query})
        response = await generate_response(
            messages, openai_api_key, max_tokens=512, temperature=1.0
        )
        await message.channel.send(response)


async def run():
    """
    The `main` function connects to a database, ingests documents from a specified folder, generates
    chunks, and logs the total number of documents and chunks.
    """
    print("Starting pgml-chat.... ")

    if stage == "ingest":
        root_dir = args.root_dir
        await ingest_documents(
            db,
            collection_name,
            root_dir,
            splitter,
            splitter_params,
            model,
            model_params,
        )

    elif stage == "chat":
        if chat_interface == "cli":
            await chat_cli()
        elif chat_interface == "slack":
            await chat_slack()


def main():
    if stage == "chat" and chat_interface == "discord" and os.environ.get("DISCORD_BOT_TOKEN"):
        client.run(os.environ["DISCORD_BOT_TOKEN"])
    else:
        asyncio.run(run())
