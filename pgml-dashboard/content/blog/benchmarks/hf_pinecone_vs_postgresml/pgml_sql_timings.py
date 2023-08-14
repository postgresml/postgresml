import requests
import os
from time import time
from rich import print
from pgml import Database
import asyncio
from statistics import mean
from datasets import load_dataset
import psycopg
from psycopg import sql
from psycopg_pool import ConnectionPool
from psycopg import Connection
from pgml import Database
from tqdm.auto import tqdm

async def pgml_query(db,model, inputs):  
    start = time()      
    query = "SELECT embed FROM pgml.embed(transformer => '%s', inputs => %s)"%(model,inputs)
    result = await db.query(query).fetch_all()
    response_time = time() - start
    # print("PGML Run %d: Time taken for %s = %0.3f" % (i, model, response_time))
    return response_time

if __name__ == "__main__":
    pool = ConnectionPool(os.environ.get("DATABASE_URL"))
    conn = pool.getconn()
    database = os.environ["DATABASE_URL"]
    model = 'intfloat/e5-large'

    data = load_dataset("squad", split="train")
    data = data.to_pandas()
    data = data.drop_duplicates(subset=["context"])

    documents = [r["context"] for r in data.to_dict(orient="records")]
    literal_docs = [sql.Literal(t).as_string(conn) for t in documents]
    pool.putconn(conn)
    total_documents = 1000
    batch_size = 64

    documents = literal_docs[:total_documents]
    
    db = Database(os.environ.get("DATABASE_URL"))
    start = time()
    for i in tqdm(range(0, len(documents), batch_size)):
        # find end of batch
        i_end = min(i+batch_size, len(documents))
        # extract batch
        batch = documents[i:i_end]
        inputs = "ARRAY[" + ",".join([sql.Literal(t).as_string(conn) for t in batch]) + "]"
        asyncio.run(pgml_query(db,model, inputs))
    print("Time taken for PGML for %d documents = %0.3f" % (len(documents), time() - start))
    
