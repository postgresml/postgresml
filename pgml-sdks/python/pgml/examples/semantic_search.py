from datasets import load_dataset
from pgml import Database
import os
from rich import print as rprint

dataset = load_dataset('quora', split='train')
questions = []

for record in dataset['questions']:
    questions.extend(record['text'])
  
# remove duplicates
documents = []
for question in list(set(questions)):
    if question:
        documents.append({"text": question})


local_pgml = "postgres://postgres@127.0.0.1:5433/pgml_development"

conninfo = os.environ.get("PGML_CONNECTION",local_pgml)
db = Database(conninfo,min_connections=4)

collection_name = "quora_collection"
collection = db.create_or_get_collection(collection_name)

collection.upsert_documents(documents[:20])
collection.generate_chunks()
collection.generate_embeddings()

query = "which city has the highest population in the world?"
result = collection.vector_search(query)
rprint(result)