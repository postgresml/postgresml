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
import json
import ast

def handler(signum, frame):
    print('Exiting...')
    exit(0)

signal.signal(signal.SIGINT, handler)

parser = argparse.ArgumentParser(description="Process some integers.")
parser.add_argument(
    "--root_dir",
    dest="root_dir",
    type=str,
    help="Input folder to scan for markdown files",
)
parser.add_argument(
    "--collection_name",
    dest="collection_name",
    type=str,
    help="Name of the collection to store the data in",
)
parser.add_argument(
    "--splitter",
    dest="splitter",
    type=str,
    help="Name of the splitter to use",
    default="recursive_character",
)
parser.add_argument(
    "--splitter_params",
    dest="splitter_params",
    type=json.loads,
    help="Parameters for the splitter",
    default={"chunk_size": 1500, "chunk_overlap": 40},
)
parser.add_argument(
    "--model",
    dest="model",
    type=str,
    help="Name of the model to use",
    default="intfloat/e5-small",
)
parser.add_argument(
    "--model_params",
    dest="model_params",
    type=str,
    help="Parameters for the model",
    default={},
)
parser.add_argument(
    "--stage",
    dest="stage",
    choices=["ingest", "chat"],
    type=str,
    default="chat",
    help="Stage to run",
)

args = parser.parse_args()

FORMAT = "%(message)s"
logging.basicConfig(
    level=os.environ.get("LOG_LEVEL", "DEBUG"),
    format="%(asctime)s - %(message)s",
    datefmt="[%X]",
    handlers=[RichHandler()],
)
log = logging.getLogger("rich")

# Load .env file
load_dotenv()


async def ingest_documents(db: Database, collection_name: str, folder: str) -> int:
    log.info("Scanning " + folder + " for markdown files")
    md_files = []
    # root_dir needs a trailing slash (i.e. /root/dir/)
    for filename in glob.iglob(folder + "**/*.md", recursive=True):
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
    await collection.register_text_splitter(splitter, splitter_params)
    query_string = """SELECT count(*) from {collection_name}.chunks""".format(
        collection_name=collection_name
    )
    results = await db.query(query_string).fetch_all()
    start_chunks = results[0]["count"]
    log.info("Starting chunk count: " + str(start_chunks))
    await collection.generate_chunks()
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

    start = time()
    await collection.generate_embeddings(model_id, splitter_id)
    log.info("Embeddings generated in %0.3f seconds" % (time() - start))

    return 0


async def generate_response(
    messages, openai_api_key, temperature=0.7, max_tokens=256, top_p=1.0
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


async def main():
    """
    The `main` function connects to a database, ingests documents from a specified folder, generates
    chunks, and logs the total number of documents and chunks.
    """
    log.info("Starting pgml_chatbot")
    collection_name = args.collection_name
    log.info("Connecting to database")
    db = Database(os.environ.get("DATABASE_URL"))

    stage = args.stage
    splitter = args.splitter
    splitter_params = args.splitter_params
    model = args.model
    if args.model_params:    
        model_params = ast.literal_eval(args.model_params)
    else:
        model_params = args.model_params

    if stage == "ingest":
        root_dir = args.root_dir

        total_docs = await ingest_documents(db, collection_name, folder=root_dir)
        total_chunks = await generate_chunks(
            db, collection_name, splitter=splitter, splitter_params=splitter_params
        )
        log.info(
            "Total documents: "
            + str(total_docs)
            + " Total chunks: "
            + str(total_chunks)
        )

        await generate_embeddings(
            db,
            collection_name,
            splitter=splitter,
            splitter_params=splitter_params,
            model=model,
            model_params=model_params,
        )
    elif stage == "chat":
        system_prompt = """You are an assistant to answer questions about an open source software named PostgresML. Your name is PgBot. You are based out of San Francisco, California.
        """
        base_prompt = """Given relevant parts of a document and a question, create a final answer. Include a SQL query in the answer wherever possible. If you don't find relevant answer then politely say that you don't know and ask for clarification. If the context is empty then ask for clarification and suggest user to send an email to team@postgresml.org or join PostgresML [Discord](https://discord.gg/DmyJP3qJ7U). Use the following portion of a long document to see if any of the text is relevant to answer the question.
        \nReturn any relevant text verbatim.\n{context}\nQuestion: {question}\n"""
        openai_api_key = os.environ.get("OPENAI_API_KEY")

        collection = await db.create_or_get_collection(collection_name)
        model_id = await collection.register_model("embedding", model, model_params)
        splitter_id = await collection.register_text_splitter(splitter, splitter_params)
        log.info("Model id: " + str(model_id) + " Splitter id: " + str(splitter_id))
        while True:
            try:
                messages = [{"role": "system", "content": system_prompt}]
                user_input = input("Ctrl-C to exit\nUser: ")
                vector_results = await collection.vector_search(
                    user_input, model_id=model_id, splitter_id=splitter_id, top_k=2, query_params=model_params
                )
                log.info(vector_results)
                context = ""
                for result in vector_results:
                    if result[0] > 0.7:
                        context += result[1] + "\n"
                if context:
                    query = base_prompt.format(context=context, question=user_input)
                else:
                    query = user_input
                log.info("User: " + query)
                messages.append({"role": "user", "content": query})
                response = await generate_response(messages, openai_api_key, max_tokens=512, temperature=0.0)
                print("PgBot: " + response)
            except KeyboardInterrupt:
                print("Exiting...")
                break


if __name__ == "__main__":
    asyncio.run(main())
