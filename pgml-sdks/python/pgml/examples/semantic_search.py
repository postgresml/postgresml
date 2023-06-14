from datasets import load_dataset
from pgml import Database
import os
import asyncio
from dotenv import load_dotenv
from time import time
from rich.console import Console
from psycopg import Connection
from psycopg_pool import ConnectionPool

async def discard_all(conninfo: str):
    pool = ConnectionPool(conninfo)
    conn = pool.getconn()
    conn.autocommit = True
    cursor = conn.cursor()
    cursor.execute("DISCARD ALL;")
    cursor.close()
    pool.putconn(conn)
    pool.close()

async def main():
    load_dotenv()
    console = Console()
    local_pgml = "postgres://postgres@127.0.0.1:5433/pgml_development"
    conninfo = os.environ.get("PGML_CONNECTION", local_pgml)
    console.print("Discarding all connections ..")
    await discard_all(conninfo)

    # Prepare Data
    dataset = load_dataset("quora", split="train")
    questions = []

    for record in dataset["questions"]:
        questions.extend(record["text"])

    # remove duplicates
    documents = []
    for question in list(set(questions)):
        if question:
            documents.append({"text": question})


    # Get Database connection
    db = Database(conninfo)
    # Create or get collection
    collection_name = "quora_collection"
    collection = await db.create_or_get_collection(collection_name)

    # Upsert documents, chunk text, and generate embeddings
    await collection.upsert_documents(documents[:200])
    await collection.generate_chunks()
    await collection.generate_embeddings()

    # Query vector embeddings
    start = time()
    query = "What is a good mobile os?"
    result = await collection.vector_search(query)
    _end = time()

    console.print("\nResults for '%s'" % (query), style="bold")
    console.print(result)
    console.print("Query time = %0.3f" % (_end - start))

    db.archive_collection(collection_name)

if __name__ == "__main__":
    asyncio.run(main())    