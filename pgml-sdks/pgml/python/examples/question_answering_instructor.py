from pgml import Collection, Model, Splitter, Pipeline
from datasets import load_dataset
from time import time
from dotenv import load_dotenv
from rich.console import Console
import asyncio


async def main():
    load_dotenv()
    console = Console()

    # Initialize collection
    collection = Collection("squad_collection_1")

    # Create a pipeline using hkunlp/instructor-base
    model = Model(
        name="hkunlp/instructor-base",
        parameters={"instruction": "Represent the Wikipedia document for retrieval: "},
    )
    splitter = Splitter()
    pipeline = Pipeline("squad_instruction", model, splitter)
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

    # Query
    query = "Who won more than 20 grammy awards?"
    console.print("Querying for %s..." % query)
    start = time()
    results = (
        await collection.query()
        .vector_recall(
            query,
            pipeline,
            query_parameters={
                "instruction": "Represent the Wikipedia question for retrieving supporting documents: "
            },
        )
        .limit(5)
        .fetch_all()
    )
    end = time()
    console.print("\n Results for '%s' " % (query), style="bold")
    console.print(results)
    console.print("Query time = %0.3f" % (end - start))

    # Archive collection
    await collection.archive()


if __name__ == "__main__":
    asyncio.run(main())
