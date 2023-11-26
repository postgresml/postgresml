import os
import requests
from time import time
from rich import print
from datasets import load_dataset
import pinecone
from tqdm.auto import tqdm
from datasets import Dataset

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

total_documents = 10000
batch_size = 64
passages = passages[:total_documents]

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

start = time()
# we will use batches of 64
for i in tqdm(range(0, len(passages), batch_size)):
    # find end of batch
    i_end = min(i+batch_size, len(passages))
    # extract batch
    batch = passages[i:i_end]
    # generate embeddings for batch via endpoints
    res = requests.post(
        endpoint,
        headers=headers,
        json={"inputs": batch}
    )
    emb = res.json()['embeddings']
    # get metadata (just the original text)
    meta = [{'text': text} for text in batch]
    # create IDs
    ids = [str(x) for x in range(i, i_end)]
    # add all to upsert list
    to_upsert = list(zip(ids, emb, meta))
    # upsert/insert these records to pinecone
    _ = index.upsert(vectors=to_upsert)

print("Time taken for HF for %d documents = %0.3f" % (len(passages),time() - start))
