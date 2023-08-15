import os
import requests
from time import time
from rich import print
import pinecone
from tqdm.auto import tqdm
from datasets import Dataset
from dotenv import load_dotenv
from statistics import mean

load_dotenv(".env")
api_org =os.environ["HF_API_KEY"]
endpoint = os.environ["HF_ENDPOINT"]
# add the api org token to the headers
headers = {
    'Authorization': f'Bearer {api_org}'
}

#squad = load_dataset("squad", split='train')
squad = Dataset.from_file("squad-train.arrow")
data = squad.to_pandas()
data = data.drop_duplicates(subset=["context"])
passages = list(data['context'])

# connect to pinecone environment
pinecone.init(
    api_key=os.environ["PINECONE_API_KEY"],
    environment=os.environ["PINECONE_ENVIRONMENT"]
)

index_name = 'hf-endpoints'

# check if the movie-emb index exists
if index_name not in pinecone.list_indexes():
    # create the index if it does not exist
    pinecone.create_index(
        index_name,
        dimension=dim,
        metric="cosine"
    )

# connect to movie-emb index we created
index = pinecone.Index(index_name)


run_times = []
for query in data["context"][0:100]:
    start = time()
    # encode with HF endpoints
    res = requests.post(endpoint, headers=headers, json={"inputs": query})
    xq = res.json()['embeddings']
    # query and return top 5
    xc = index.query(xq, top_k=5, include_metadata=True)
    _end = time()
    run_times.append(_end-start)
print("HF + Pinecone Average query time: %0.3f"%(mean(run_times)))



