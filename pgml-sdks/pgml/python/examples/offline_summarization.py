from pgml import Collection, Pipeline
import psycopg2
import asyncio
import pandas as pd
from dotenv import load_dotenv
import os

load_dotenv()
db_url = os.environ['PGML_DATABASE_URL']

# Initialize Collection
collection = Collection("summary_demo")

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

    # Now that we have the documents in our database, let's do some summarization
    conn = psycopg2.connect(db_url)
    cur = conn.cursor()

    # First lets create our summary table
    # This table has three columns:
    # id - auto incrementing primary key
    # document_id - the document id we summarized
    # summary - the summary of the text of the document
    # version - the document `text` key version. This is really just
    # the hash of the `text` column. Stored in the version column of the documents table.
    # We store this column so we don't recompute summaries
    cur.execute("""
        CREATE TABLE IF NOT EXISTS summary_demo.summaries (
            id SERIAL PRIMARY KEY,
            document_id INTEGER REFERENCES summary_demo.documents (id) UNIQUE,
            summary TEXT,
            version VARCHAR(32)
        )
        """)
    conn.commit()

    # Now let's fill up our summary table
    # This query is very efficient as it only updates the summary for documents not currently in
    # the table, or whos `text` key has changed since the last summary
    cur.execute("""
        INSERT INTO summary_demo.summaries (document_id, summary, version)
        SELECT 
            sdd.id, (
                SELECT transform[0]->>'summary_text' FROM pgml.transform(
                  task   => '{
                    "task": "summarization",
                    "model": "google/pegasus-xsum"
                  }'::JSONB,
                  inputs => ARRAY[
                    sdd.document->>'text'
                  ]
                )
            ),
            sdd.version->'text'->>'md5'
        FROM summary_demo.documents sdd
        LEFT OUTER JOIN summary_demo.summaries as sds ON sds.document_id = sdd.id
        WHERE sds.document_id IS NULL OR sds.version != sdd.version->'text'->>'md5'
        ON CONFLICT (document_id) DO UPDATE SET version = EXCLUDED.version, summary = EXCLUDED.summary
        """)
    conn.commit()

    # Let's see what our summaries are
    cur.execute("SELECT * FROM summary_demo.summaries")
    for row in cur.fetchall():
        print(row)

    # Purposefully not removing the tables and collection so they can be inspected in the database
    

asyncio.run(main())
