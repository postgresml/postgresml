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
    collection = Collection("quora_collection")

    # Create and add pipeline
    pipeline = Pipeline(
        "quorav1",
        {
            "text": {
                "splitter": {"model": "recursive_character"},
                "semantic_search": {"model": "Alibaba-NLP/gte-base-en-v1.5"},
            }
        },
    )
    await collection.add_pipeline(pipeline)
 
    # Prep documents for upserting
    dataset = load_dataset("quora", split="train")
    questions = []
    for record in dataset["questions"]:
        questions.extend(record["text"])

    # Remove duplicates and add id
    documents = []
    for i, question in enumerate(list(set(questions))):
        if question:
            documents.append({"id": i, "text": question})

    # Upsert documents
    await collection.upsert_documents(documents[:2000])

    # Query
    query = "What is a good mobile os?"
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
