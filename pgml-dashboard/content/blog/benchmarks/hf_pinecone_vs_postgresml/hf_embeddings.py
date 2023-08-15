import os
import requests
from time import time
from rich import print
from datasets import load_dataset
from tqdm.auto import tqdm
from datasets import Dataset
from dotenv import load_dotenv

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

total_documents = 10000
batch_size = 1
passages = passages[:total_documents]

start = time()
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

print("Time taken for HF for %d documents = %0.3f" % (len(passages),time() - start))
