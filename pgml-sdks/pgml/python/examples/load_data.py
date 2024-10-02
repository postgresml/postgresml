import asyncio
from pgml import Collection, Pipeline
import pandas as pd
from dotenv import load_dotenv

load_dotenv()

# Initialize Collection
collection = Collection("load_data_demo")

# Iniitalize Pipeline
pipeline = Pipeline(
    "v1",
    {
        "text": {
            "splitter": {"model": "recursive_character"},
            "semantic_search": {"model": "intfloat/e5-small"},
        }
    },
)


async def init_collection():
    await collection.add_pipeline(pipeline)


def load_documents():
    # This can be any loading function. For our case, we will be loading in a CSV
    # The important piece is that our upsert_documents wants an array of dictionaries
    data = pd.read_csv("./data/example_data.csv")
    return data.to_dict("records")


async def main():
    # We only ever need to add a Pipeline once
    await init_collection()

    # Get our documents. Documents are just dictionaries with at least the `id` key
    # E.G. {"id": "document_one, "text": "here is some text"}
    documents = load_documents()

    # This does the actual uploading of our documents
    # It handles uploading in batches and guarantees that any documents uploaded are
    # split and embedded according to our Pipeline definition above
    await collection.upsert_documents(documents)

    # The default batch size is 100, but we can override that if we have thousands or
    # millions of documents to upload it will be faster with a larger batch size
    # await collection.upsert_documents(documents, {"batch_size": 1000})

    # Now we can search over our collection or do whatever else we want
    # See other examples for more information on searching


asyncio.run(main())
