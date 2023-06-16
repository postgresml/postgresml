from pgml import Database
import os
from datasets import load_dataset
from time import time
from dotenv import load_dotenv
from rich.console import Console
import asyncio

async def main():
    load_dotenv()
    console = Console()
    local_pgml = "postgres://postgres@127.0.0.1:5433/pgml_development"

    conninfo = os.environ.get("PGML_CONNECTION", local_pgml)
    db = Database(conninfo)

    collection_name = "squad_collection"
    collection = await db.create_or_get_collection(collection_name)

    data = load_dataset("squad", split="train")
    data = data.to_pandas()
    data = data.drop_duplicates(subset=["context"])

    documents = [
        {"id": r["id"], "text": r["context"], "title": r["title"]}
        for r in data.to_dict(orient="records")
    ]

    console.print("Upserting documents ..")
    await collection.upsert_documents(documents[:200])
    console.print("Generating chunks ..")
    await collection.generate_chunks()
    console.print("Generating embeddings ..")
    await collection.generate_embeddings()

    console.print("Querying ..")
    start = time()
    query = "Who won 20 grammy awards?"
    results = await collection.vector_search(query, top_k=5)
    _end = time()
    console.print("\nResults for '%s'" % (query), style="bold")
    console.print(results)
    console.print("Query time = %0.3f" % (_end - start))

    await db.archive_collection(collection_name)

if __name__ == "__main__":
    asyncio.run(main())