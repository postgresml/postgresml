from qdrant_client import QdrantClient
from qdrant_client.models import Distance, VectorParams, PointStruct
from dotenv import load_dotenv
import time
import os

# Load our environment variables
load_dotenv()
QDRANT_API_KEY = os.getenv("QDRANT_API_KEY")

# Create our Qdrant client
qdrant = QdrantClient(
    url="https://059364f6-62c5-4f80-9f19-cf6d6394caae.us-east4-0.gcp.cloud.qdrant.io:6333",
    api_key=QDRANT_API_KEY,
)

# Create our Qdrant collection
qdrant.recreate_collection(
    collection_name="test",
    vectors_config=VectorParams(size=384, distance=Distance.COSINE),
)


# Store some initial documents to retrieve
def upsert_data(documents, embeddings):
    points = [
        PointStruct(
            id=int(document["id"]), vector=embedding, payload=document["metadata"]
        )
        for document, embedding in zip(documents, embeddings)
    ]
    print("\tStarting Qdrant upsert")
    tic = time.perf_counter()
    qdrant.upsert(collection_name="test", points=points)
    toc = time.perf_counter()
    time_taken_to_upsert = toc - tic
    print(f"\tDone Qdrant upsert: {time_taken_to_upsert:0.4f}")
    return time_taken_to_upsert


# Do cosine similarity search over our Qdrant collection
def do_search(vector):
    print("\tDoing cosine similarity search with Qdrant")
    tic = time.perf_counter()
    results = qdrant.search(collection_name="test", query_vector=vector, limit=1)
    toc = time.perf_counter()
    time_done = toc - tic
    print(f"\tDone doing cosine similarity search: {time_done:0.4f}\n")
    return (results, time_done)
