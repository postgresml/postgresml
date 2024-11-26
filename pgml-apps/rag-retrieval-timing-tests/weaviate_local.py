import weaviate
import weaviate.classes as wvc
import os
import time
from dotenv import load_dotenv

# Load our environment variables
load_dotenv()
WEVIATE_API_KEY = os.getenv("WCS_API_KEY")
OPENAI_API_KEY = os.getenv("OPENAI_API_KEY")
HF_TOKEN = os.getenv("HF_TOKEN")

# Connect to a WCS instance
client = weaviate.connect_to_wcs(
    cluster_url="https://test-n0wsyrvs.weaviate.network",
    auth_credentials=weaviate.auth.AuthApiKey(WEVIATE_API_KEY),
    headers={
        "X-OpenAI-Api-Key": OPENAI_API_KEY,
        "X-HuggingFace-Api-Key": HF_TOKEN,
    },
)

# NOTE: We can only create this once or it seems to error out
# Create Weaviate Collection
# test_collection = client.collections.create(
#     name="test",
#     vectorizer_config=wvc.config.Configure.Vectorizer.text2vec_huggingface(
#         "intfloat/e5-small"
#     ),
#     generative_config=wvc.config.Configure.Generative.openai(),
#     properties=[
#         wvc.config.Property(
#             name="text",
#             data_type=wvc.config.DataType.TEXT,
#             vectorize_property_name=True,
#             tokenization=wvc.config.Tokenization.LOWERCASE,
#         ),
#     ],
#     vector_index_config=wvc.config.Configure.VectorIndex.hnsw(
#         distance_metric=wvc.config.VectorDistances.COSINE,
#         quantizer=wvc.config.Configure.VectorIndex.Quantizer.pq(),
#     ),
# )
test_collection = client.collections.get("test")


def upsert_data(documents):
    documents = [
        wvc.data.DataObject(properties={"text": document["metadata"]["text"]})
        for document in documents
    ]
    print("Starting PostgresML upsert")
    tic = time.perf_counter()
    test_collection.data.insert_many(documents)
    toc = time.perf_counter()
    time_taken = toc - tic
    print(f"Done PostgresML upsert: {time_taken:0.4f}\n")


def get_llm_response(query):
    print("\tDoing Embedding, Search, and Getting LLM Fesponse from Weaviate")
    tic = time.perf_counter()
    response = test_collection.generate.near_text(
        query=query, limit=1, grouped_task=query
    )
    toc = time.perf_counter()
    time_taken = toc - tic
    print(
        f"\tDone Doing Embedding, Search, and Getting LLM Response from Weaviate: {time_taken:0.4f}"
    )
    return (response.generated, time_taken)
