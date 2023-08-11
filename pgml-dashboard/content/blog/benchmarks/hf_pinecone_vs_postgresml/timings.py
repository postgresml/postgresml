import requests
import os
from time import time
from rich import print
from pgml import Database
import asyncio
from statistics import mean

hf_token = os.environ["HF_API_KEY"]
database = os.environ["DATABASE_URL"]
embeddings_models = ["intfloat/e5-small", "intfloat/e5-large"]
headers = {"Authorization": f"Bearer {hf_token}"}


def query(model, texts, headers):
    api_url = (
        f"https://api-inference.huggingface.co/pipeline/feature-extraction/{model}"
    )
    response = requests.post(
        api_url,
        headers=headers,
        json={"inputs": texts, "options": {"wait_for_model": True}},
    )
    return response.json()


texts = [
   'How do I get a replacement Medicare card?',
   'What is the monthly premium for Medicare Part B?',
    'How do I terminate my Medicare Part B (medical insurance)?',
        'How do I sign up for Medicare?',
        'Can I sign up for Medicare Part B if I am working and have health insurance through an employer?',
        'How do I sign up for Medicare Part B if I already have Part A?',
        'What are Medicare late enrollment penalties?',
        'What is Medicare and who can get it?',
        'How can I get help with my Medicare Part A and Part B premiums?',
        'What are the different parts of Medicare?'
]

async def pgml_query(model, texts):
    db = Database(database)
    query = """SELECT pgml.embed(transformer=> '{model}',inputs=>ARRAY{text})"""
    
    for model in embeddings_models:
        run_times = []
        print("Running model: ", model)
        for i in range(10):
            start = time()
            output = await db.query(query.format(model=model, text=texts)).fetch_all()
            response_time = time() - start
            print("PGML Run %d: Time taken for %s = %0.3f" % (i, model, response_time))
            run_times.append(response_time)
        print("Average: %0.3f"%mean(run_times[5:]))
    return mean(run_times)

if __name__ == "__main__":
    for model in embeddings_models:
        run_times = []
        print("Running model: ", model)
        for i in range(10):
            start = time()
            output = query(model, texts, headers)
            response_time = time() - start
            print("HF Run %d: Time taken for %s = %0.3f" % (i, model, response_time))
            run_times.append(response_time)
        print("Average: %0.3f"%mean(run_times[5:]))

    asyncio.run(pgml_query(embeddings_models, texts))