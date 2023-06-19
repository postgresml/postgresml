from pgml import Database
import os
from datasets import load_dataset
from time import time
from dotenv import load_dotenv
from rich.console import Console
from rich.progress import track
import pandas as pd
import asyncio


async def main():
    load_dotenv()
    console = Console()

    local_pgml = "postgres://postgres@127.0.0.1:5433/pgml_development"

    conninfo = os.environ.get("PGML_CONNECTION", local_pgml)
    db = Database(conninfo)

    collection_name = "ott_qa_20k_collection"
    collection = await db.create_or_get_collection(collection_name)


    data = load_dataset("ashraq/ott-qa-20k", split="train")
    documents = []

    # loop through the dataset and convert tabular data to pandas dataframes
    for doc in track(data):
        table = pd.DataFrame(doc["data"], columns=doc["header"])
        processed_table = "\n".join([table.to_csv(index=False)])
        documents.append(
            {
                "text": processed_table,
                "title": doc["title"],
                "url": doc["url"],
                "uid": doc["uid"],
            }
        )

    console.print("Upserting documents ..")
    await collection.upsert_documents(documents[:100])
    console.print("Generating chunks ..")
    await collection.generate_chunks()

    # SentenceTransformer model trained specifically for embedding tabular data for retrieval tasks
    model_name = "deepset/all-mpnet-base-v2-table"
    model_id = await collection.register_model(model_name=model_name)
    console.print("Generating embeddings .. for model %s" % (model_id), style="bold")
    await collection.generate_embeddings(model_id=model_id)

    console.print("Querying ..")
    start = time()
    query = "which country has the highest GDP in 2020?"
    results = await collection.vector_search(query, top_k=5, model_id=model_id)
    _end = time()
    console.print("\nResults for '%s'" % (query), style="bold")
    console.print(results)
    console.print("Query time = %0.3f" % (_end - start))

    await db.archive_collection(collection_name)

if __name__ == "__main__":
    asyncio.run(main())