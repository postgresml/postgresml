from pgml import Database
import os
import json
from datasets import load_dataset
from time import time
from rich import print as rprint

local_pgml = "postgres://postgres@127.0.0.1:5433/pgml_development"

conninfo = os.environ.get("PGML_CONNECTION", local_pgml)
db = Database(conninfo)

collection_name = "test_pgml_sdk_1"
collection = db.create_or_get_collection(collection_name)


data = load_dataset("squad", split="train")
data = data.to_pandas()
data = data.drop_duplicates(subset=["context"])

documents = [
    {"text": r["context"], "metadata": {"title": r["title"]}}
    for r in data.to_dict(orient="records")
]

collection.upsert_documents(documents[:200])
collection.generate_chunks()
collection.generate_embeddings()

start = time()
results = collection.vector_search("Who won 20 grammy awards?", top_k=2)
rprint(json.dumps(results, indent=2))
rprint("Query time %0.3f"%(time()-start))
db.delete_collection(collection_name)
