import requests
import time
import os
import sys
from dotenv import load_dotenv

# Load our environment variables
load_dotenv()
HF_TOKEN = os.getenv("HF_TOKEN")


# Get the embedding from HuggingFace
def get_embeddings(inputs):
    print("\tGetting embeddings from HuggingFace")
    tic = time.perf_counter()
    headers = {"Authorization": f"Bearer {HF_TOKEN}"}
    payload = {"inputs": inputs}
    response = requests.post(
        "https://api-inference.huggingface.co/pipeline/feature-extraction/intfloat/e5-small",
        headers=headers,
        json=payload,
    )
    toc = time.perf_counter()
    time_taken = toc - tic
    print(f"\tDone getting embeddings: {toc - tic:0.4f}\n")
    response = response.json()
    if "error" in response:
        sys.exit(response)
    return (response, time_taken)
