from pgml import Collection, Model, Splitter, Pipeline
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

    # Initialize collection
    collection = Collection("ott_qa_20k_collection")

    # Create and add pipeline
    pipeline = Pipeline(
        "ott_qa_20kv1",
        {
            "text": {
                "splitter": {"model": "recursive_character"},
                # A SentenceTransformer model trained specifically for embedding tabular data for retrieval
                "semantic_search": {"model": "deepset/all-mpnet-base-v2-table"},
            }
        },
    )
    await collection.add_pipeline(pipeline)

    # Prep documents for upserting
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
                "id": doc["uid"],
            }
        )

    # Upsert documents
    await collection.upsert_documents(documents[:100])

    # Query
    query = "Which country has the highest GDP in 2020?"
    console.print("Querying for %s..." % query)
    start = time()
    results = await collection.vector_search(
        {"query": {"fields": {"text": {"query": query}}}, "limit": 5}, pipeline
    )
    end = time()
    console.print("\n Results for '%s' " % (query), style="bold")
    console.print(results)
    console.print("Query time = %0.3f" % (end - start))

    # Archive collection
    await collection.archive()


if __name__ == "__main__":
    asyncio.run(main())
