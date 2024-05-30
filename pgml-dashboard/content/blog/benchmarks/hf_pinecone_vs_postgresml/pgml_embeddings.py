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
    model_id = await collection.register_model(model_name="Alibaba-NLP/gte-base-en-v1.5")
    await collection.generate_embeddings(model_id=model_id)

if __name__ == "__main__":
    asyncio.run(main())
