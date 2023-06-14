# Types
from typing import List
from pydantic import BaseModel

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware

from langchain.document_loaders.unstructured import UnstructuredFileLoader
from langchain.docstore.document import Document

import os
import pgml


app = FastAPI()


@app.on_event("startup")
async def startup():
    local_pgml = "postgres://postgres@127.0.0.1:5433/pgml_development"
    conninfo = os.environ.get("PGML_CONNECTION", local_pgml)
    app.state.db: pgml.Database = pgml.Database(conninfo, min_connections=4)


class IngestionBody(BaseModel):
    collection_name: str
    document_path: str

class SearchBody(BaseModel):
    question: str
    collection_name: str
    k: int = 5
    metadata_filter : dict = {}

@app.post("/ingest")
async def insert_documents(body: IngestionBody):
    """
    Example:
        curl --location 'http://0.0.0.0:8888/ingest' \
        --header 'Content-Type: application/json' \
        --data '{
            "collection_name": "test_collection",
            "document_path": "~/path/to/pdf"
        }'
    """
    
    try:
    
        # # Get Db connection from pgml
        db: pgml.Database = app.state.db
        collection: pgml.Collection = db.create_or_get_collection(body.collection_name)
        
        
        # get documents, using Langchain Unstructored loader. 
        print("Loading Document")
        loader: List[Document] = UnstructuredFileLoader(body.document_path).load()
        
        # Converting from  Langchain Document to regular dict for pgml to process
        documents: List[dict] = [
            {
                "text": x.page_content,
                **x.metadata
            }
            for x in loader
        ]
        
        # fun stuff
        collection.upsert_documents(documents)
        collection.generate_chunks()
        collection.generate_embeddings()
    
        return documents
    
    except Exception as e:
        raise HTTPException(status_code=404, detail=str(e))


@app.post("/search")
async def search_documents(body: SearchBody):
    """
    Example:
       curl --location 'http://0.0.0.0:8888/search' \
        --header 'Content-Type: application/json' \
        --data '{
            "collection_name": "testing",
            "question": "What people did he met?"
        }'
    """
    
    try:
    
        # # Get Db connection from pgml
        db: pgml.Database = app.state.db
        collection: pgml.Collection = db.create_or_get_collection(body.collection_name)
        return collection.vector_search(body.question, top_k=body.k, metadata_filter=body.metadata_filter)
    
    except Exception as e:
        raise HTTPException(status_code=404, detail=str(e))
  
    
if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8888)
