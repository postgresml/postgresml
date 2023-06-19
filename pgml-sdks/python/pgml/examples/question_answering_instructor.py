from pgml import Database
import os
from datasets import load_dataset
from time import time
from dotenv import load_dotenv
from rich.console import Console
import asyncio

async def main():
    load_dotenv()
    console = Console()

    local_pgml = "postgres://postgres@127.0.0.1:5433/pgml_development"

    conninfo = os.environ.get("PGML_CONNECTION", local_pgml)
    db = Database(conninfo)

    collection_name = "squad_collection"
    collection = await db.create_or_get_collection(collection_name)



    data = load_dataset("squad", split="train")
    data = data.to_pandas()
    data = data.drop_duplicates(subset=["context"])

    documents = [
        {"id": r["id"], "text": r["context"], "title": r["title"]}
        for r in data.to_dict(orient="records")
    ]

    console.print("Upserting documents ..")
    await collection.upsert_documents(documents[:200])
    console.print("Generating chunks ..")
    await collection.generate_chunks(splitter_id=1)

    # register instructor model
    console.print("Registering instructor model ..")
    instructor_model = "hkunlp/instructor-base" 
    instructor_model_params = {"instruction": "Represent the Wikipedia document for retrieval: "}

    model_id = await collection.register_model(
        model_name=instructor_model,
        model_params=instructor_model_params,
    )


    console.print("Generating embeddings .. for model %s" % (model_id), style="bold")
    await collection.generate_embeddings(model_id=model_id)

    console.print("Querying ..")
    start = time()
    query = "Who won 20 grammy awards?"
    results = await collection.vector_search(
        query,
        top_k=5,
        model_id=model_id,
        query_params={
            "instruction": "Represent the Wikipedia question for retrieving supporting documents: "
        },
    )
    _end = time()
    console.print("\nResults for '%s'" % (query), style="bold")
    console.print(results)
    console.print("Query time = %0.3f" % (_end - start))

    await db.archive_collection(collection_name)

if __name__ == "__main__":
    asyncio.run(main())