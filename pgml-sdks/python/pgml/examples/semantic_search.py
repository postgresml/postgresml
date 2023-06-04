from datasets import load_dataset
from pgml import Database
import os
from rich import print as rprint
from dotenv import load_dotenv
from time import time
from rich.console import Console

load_dotenv()
console = Console()

# Prepare Data
dataset = load_dataset("quora", split="train")
questions = []

for record in dataset["questions"]:
    questions.extend(record["text"])

# remove duplicates
documents = []
for question in list(set(questions)):
    if question:
        documents.append({"text": question})


# Get Database connection
local_pgml = "postgres://postgres@127.0.0.1:5433/pgml_development"
conninfo = os.environ.get("PGML_CONNECTION", local_pgml)
db = Database(conninfo, min_connections=4)

# Create or get collection
collection_name = "quora_collection"
collection = db.create_or_get_collection(collection_name)

# Upsert documents, chunk text, and generate embeddings
collection.upsert_documents(documents[:200])
collection.generate_chunks()
collection.generate_embeddings()

# Query vector embeddings
start = time()
query = "What is a good mobile os?"
result = collection.vector_search(query)
_end = time()

console.print("\nVector Results for '%s'" % (query), style="bold")
console.print(result)
console.print("Vector Query time = %0.3f" % (_end - start))


# Query vector embeddings
start = time()
query = "What is a good mobile os?"
result = collection.text_search(query, top_k=5)
_end = time()

console.print("\nText Results for '%s'" % (query), style="bold")
console.print(result)
console.print("Text Query time = %0.3f" % (_end - start))


db.archive_collection(collection_name)
