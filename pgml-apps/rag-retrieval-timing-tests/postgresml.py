from pgml import Collection, Pipeline
from dotenv import load_dotenv
import time

# Load our environment variables
load_dotenv()

# Initialize our Collection and Pipeline
collection = Collection("test_collection")
pipeline = Pipeline(
    "test_pipeline",
    {
        "text": {
            "semantic_search": {
                "model": "intfloat/e5-small",
            },
        }
    },
)


# Add the Pipeline to our collection
# We only need to do this once
async def setup_pipeline():
    await collection.add_pipeline(pipeline)


async def upsert_data(documents):
    documents = [
        {"id": document["id"], "text": document["metadata"]["text"]}
        for document in documents
    ]
    print("Starting PostgresML upsert")
    tic = time.perf_counter()
    await collection.upsert_documents(documents)
    toc = time.perf_counter()
    time_taken = toc - tic
    print(f"Done PostgresML upsert: {time_taken:0.4f}\n")


async def do_search(query):
    print(
        "\tDoing embedding and cosine similarity search over our PostgresML Collection"
    )
    tic = time.perf_counter()
    results = await collection.vector_search(
        {
            "query": {
                "fields": {
                    "text": {
                        "query": query,
                    },
                }
            },
            "limit": 1,
        },
        pipeline,
    )
    toc = time.perf_counter()
    time_taken = toc - tic
    print(f"\tDone doing embedding and cosine similarity search: {time_taken:0.4f}\n")
    return (results[0]["chunk"], time_taken)
