from pgml import Database
import os
import json
from datasets import load_dataset
from time import time
from dotenv import load_dotenv
from rich.console import Console

load_dotenv()
console = Console()

local_pgml = "postgres://postgres@127.0.0.1:5433/pgml_development"

conninfo = os.environ.get("PGML_CONNECTION", local_pgml)
db = Database(conninfo)

collection_name = "squad_collection"
collection = db.create_or_get_collection(collection_name)


data = load_dataset("squad", split="train")
data = data.to_pandas()
data = data.drop_duplicates(subset=["context"])

documents = [
    {"id": r["id"], "text": r["context"], "title": r["title"]}
    for r in data.to_dict(orient="records")
]

collection.upsert_documents(documents[:200])
collection.generate_chunks()

#register instructor model
model_id = collection.register_model(model_name="hkunlp/instructor-base", model_params={"instruction": "Represent the Wikipedia document for retrieval: "})
collection.generate_embeddings(model_id=model_id)

start = time()
query = "Who won 20 grammy awards?"
results = collection.vector_search(query, top_k=5, model_id = model_id, query_parameters={"instruction": "Represent the Wikipedia question for retrieving supporting documents: "})
_end = time()
console.print("\nResults for '%s'" % (query), style="bold")
console.print(results)
console.print("Query time = %0.3f" % (_end - start))

db.archive_collection(collection_name)
