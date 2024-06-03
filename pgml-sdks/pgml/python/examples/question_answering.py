from pgml import Collection, Pipeline
from datasets import load_dataset
from time import time
from dotenv import load_dotenv
from rich.console import Console
import asyncio


async def main():
    load_dotenv()
    console = Console()

    # Initialize collection
    collection = Collection("squad_collection")

    # Create and add pipeline
    pipeline = Pipeline(
        "squadv1",
        {
            "text": {
                "splitter": {"model": "recursive_character"},
                "semantic_search": {"model": "Alibaba-NLP/gte-base-en-v1.5"},
            }
        },
    )
    await collection.add_pipeline(pipeline)

    # Prep documents for upserting
    data = load_dataset("squad", split="train")
    data = data.to_pandas()
    data = data.drop_duplicates(subset=["context"])
    documents = [
        {"id": r["id"], "text": r["context"], "title": r["title"]}
        for r in data.to_dict(orient="records")
    ]

    # Upsert documents
    await collection.upsert_documents(documents[:200])

    # Query for answer
    query = "Who won more than 20 grammy awards?"
    console.print("Querying for context ...")
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
