from pgml import Database
import os
from time import time
from dotenv import load_dotenv
from rich.console import Console
from pypdf import PdfReader
from pgml.dbutils import run_select_statement

load_dotenv()
console = Console()

local_pgml = "postgres://postgres@127.0.0.1:5433/pgml_development"

conninfo = os.environ.get("PGML_CONNECTION", local_pgml)
db = Database(conninfo)

collection_name = "pdf_collection"
collection = db.create_or_get_collection(collection_name)

filename = "lincoln.pdf"
reader = PdfReader("lincoln.pdf")
number_of_pages = len(reader.pages)
documents = []
for page_number, page in enumerate(reader.pages):
    documents.append(
        {"text": page.extract_text(), "page": page_number, "source": filename}
    )

collection.upsert_documents(documents)
collection.generate_chunks()
collection.generate_embeddings()

start = time()
query = "When was Lincoln born?"
results = collection.vector_search(query, top_k=1)

# db.archive_collection(collection_name)
conn = db.pool.getconn()
context = " ".join(results[0]["chunk"].strip().split())
context = context.replace('"', '\\"').replace("'", "''")

select_statement = """SELECT pgml.transform(
    'question-answering',
    inputs => ARRAY[
        '{
            \"question\": \"%s\",
            \"context\": \"%s\"
        }'
    ]
) AS answer;""" % (
    query,
    context,
)

results = run_select_statement(conn, select_statement)
db.pool.putconn(conn)

console.print("\nResults for query '%s'" % query)
console.print(results)
db.archive_collection(collection_name)
_end = time()
console.print("Query time = %0.3f" % (_end - start))
