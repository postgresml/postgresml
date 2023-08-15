from pgml import Database
import os
from datasets import load_dataset
from time import time
from dotenv import load_dotenv
from rich import print
import asyncio
from tqdm.auto import tqdm
from statistics import mean

async def main():
    load_dotenv()

    conninfo = os.environ.get("DATABASE_URL")
    db = Database(conninfo)

    collection_name = "squad_collection_benchmark"
    collection = await db.create_or_get_collection(collection_name)

    data = load_dataset("squad", split="train")
    data = data.to_pandas()
    data = data.drop_duplicates(subset=["context"])

    run_times = []
    for query in data["context"][0:100]:
        start = time()
        results = await collection.vector_search(query, top_k=5)
        _end = time()
        run_times.append(_end-start)
    #print("PGML Query times:")
    #print(run_times)
    print("PGML Average query time: %0.3f"%mean(run_times))

    #await db.archive_collection(collection_name)

if __name__ == "__main__":
    asyncio.run(main())
