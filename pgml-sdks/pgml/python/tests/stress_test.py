import asyncio
import pgml
import time
from datasets import load_dataset

pgml.init_logger()

TOTAL_ROWS = 10000
BATCH_SIZE = 1000
OFFSET = 0

dataset = load_dataset(
    "wikipedia", "20220301.en", trust_remote_code=True, split="train"
)

collection = pgml.Collection("stress-test-collection-3")
pipeline = pgml.Pipeline(
    "stress-test-pipeline-1",
    {
        "text": {
            "splitter": {
                "model": "recursive_character",
            },
            "semantic_search": {
                "model": "Alibaba-NLP/gte-base-en-v1.5",
            },
        },
    },
)


async def upsert_data():
    print(f"\n\nUploading {TOTAL_ROWS} in batches of {BATCH_SIZE}")
    total = 0
    batch = []
    tic = time.perf_counter()
    for d in dataset:
        total += 1
        if total < OFFSET:
            continue
        batch.append(d)
        if len(batch) >= BATCH_SIZE or total >= TOTAL_ROWS:
            await collection.upsert_documents(batch, {"batch_size": 1000})
            batch = []
        if total >= TOTAL_ROWS:
            break
    toc = time.perf_counter()
    print(f"Done in {toc - tic:0.4f} seconds\n\n")


async def test_document_search():
    print("\n\nDoing document search")
    tic = time.perf_counter()

    results = await collection.search(
        {
            "query": {
                "semantic_search": {
                    "text": {
                        "query": "What is the best fruit?",
                        "parameters": {
                            "instruction": "Represent the Wikipedia question for retrieving supporting documents: "
                        },
                    }
                },
                "filter": {"title": {"$ne": "filler"}},
            },
            "limit": 1,
        },
        pipeline,
    )
    toc = time.perf_counter()
    print(f"Done in {toc - tic:0.4f} seconds\n\n")


async def test_vector_search():
    print("\n\nDoing vector search")
    tic = time.perf_counter()
    results = await collection.vector_search(
        {
            "query": {
                "fields": {
                    "text": {
                        "query": "What is the best fruit?",
                        "parameters": {
                            "instruction": "Represent the Wikipedia question for retrieving supporting documents: "
                        },
                    },
                },
                "filter": {"title": {"$ne": "filler"}},
            },
            "limit": 5,
        },
        pipeline,
    )
    toc = time.perf_counter()
    print(f"Done in {toc - tic:0.4f} seconds\n\n")


async def main():
    await collection.add_pipeline(pipeline)
    await upsert_data()
    await test_document_search()
    await test_vector_search()


asyncio.run(main())
