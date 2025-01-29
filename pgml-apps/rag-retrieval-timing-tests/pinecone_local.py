from pinecone import Pinecone, ServerlessSpec
from dotenv import load_dotenv
import time
import os

# Load our environment variables
load_dotenv()
PINECONE_API_KEY = os.getenv("PINECONE_API_KEY")

# Create our Pinecone client
# Note we created their default index using their gcp-start region and us-central1 region
pc = Pinecone(api_key=PINECONE_API_KEY)
index = pc.Index("test")


# Store some initial documents to retrieve
def upsert_data(documents, embeddings):
    for document, embedding in zip(documents, embeddings):
        document["values"] = embedding
    print("\tStarting PineCone upsert")
    tic = time.perf_counter()
    index.upsert(documents, namespace="ns1")
    toc = time.perf_counter()
    time_taken_to_upsert = toc - tic
    print(f"\tDone PineCone upsert: {time_taken_to_upsert:0.4f}")
    return time_taken_to_upsert


# Do cosine similarity search over our pinecone index
def do_search(vector):
    print("\tDoing cosine similarity search with PineCone")
    tic = time.perf_counter()
    results = index.query(
        namespace="ns1",
        vector=vector,
        top_k=1,
        include_metadata=True,
    )
    toc = time.perf_counter()
    time_done = toc - tic
    print(f"\tDone doing cosine similarity search: {time_done:0.4f}\n")
    result = results["matches"][0]["metadata"]["text"]
    return (result, time_done)
