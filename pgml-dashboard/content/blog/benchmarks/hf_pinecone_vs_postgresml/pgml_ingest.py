from pgml import Database
import os
from datasets import load_dataset
from time import time
from dotenv import load_dotenv
from rich import print
import asyncio
from tqdm.auto import tqdm

async def main():
    load_dotenv()
    conninfo = os.environ.get("DATABASE_URL")
    db = Database(conninfo)

    collection_name = "squad_collection_benchmark"
    collection = await db.create_or_get_collection(collection_name)

    data = load_dataset("squad", split="train")
    data = data.to_pandas()
    data = data.drop_duplicates(subset=["context"])

    documents = [
        {"id": r["id"], "text": r["context"], "title": r["title"]}
        for r in data.to_dict(orient="records")
    ]

    print("Ingesting and chunking documents ..")
    total_documents = 10000
    batch_size = 64
    embedding_times = []
    total_time = 0
    documents = documents[:total_documents]
    for i in tqdm(range(0,len(documents),batch_size)):
        i_end = min(i+batch_size,len(documents))
        batch = documents[i:i_end]
        await collection.upsert_documents(batch)
        await collection.generate_chunks()
    print("Ingesting and chunking completed")

if __name__ == "__main__":
    asyncio.run(main())
