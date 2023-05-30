from pgml import Database
import os
import json
from datasets import load_dataset
from time import time
from dotenv import load_dotenv
from rich.console import Console
from rich.progress import track
from psycopg import sql
from pgml.dbutils import run_select_statement
import pandas as pd

load_dotenv()
console = Console()

local_pgml = "postgres://postgres@127.0.0.1:5433/pgml_development"

conninfo = os.environ.get("PGML_CONNECTION", local_pgml)
db = Database(conninfo)

collection_name = "ott_qa_20k_collection"
collection = db.create_or_get_collection(collection_name)


data = load_dataset("ashraq/ott-qa-20k", split="train")
documents = []

# loop through the dataset and convert tabular data to pandas dataframes
for doc in track(data):
    table = pd.DataFrame(doc["data"], columns=doc["header"])
    processed_table = "\n".join([table.to_csv(index=False)])
    documents.append({"text": processed_table, "title": doc["title"], "url": doc["url"], "uid": doc["uid"]})

collection.upsert_documents(documents)
collection.generate_chunks()

# SentenceTransformer model trained specifically for embedding tabular data for retrieval tasks
model_id = collection.register_model(model_name="deepset/all-mpnet-base-v2-table")
collection.generate_embeddings(model_id=model_id)

start = time()
query = "which country has the highest GDP in 2020?"
results = collection.vector_search(query, top_k=5, model_id=model_id)
_end = time()
console.print("\nResults for '%s'" % (query), style="bold")
console.print(results)
console.print("Query time = %0.3f" % (_end - start))

db.archive_collection(collection_name)
