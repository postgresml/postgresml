import os
import pgml
import pytest
from typing import List, Dict, Any

####################################################################################
####################################################################################
## PLEASE BE AWARE THESE TESTS DO INVOLVE CHECKS ON LAZILY CREATD DATABASE ITEMS. ##
## IF ANY OF THE COLLECTION NAMES ALREADY EXIST, SOME TESTS MAY FAIL              ##
## THIS DOES NOT MEAN THE SDK IS BROKEN. PLEASE CLEAR YOUR DATABASE INSTANCE      ##
## BEFORE RUNNING ANY TESTS                                                       ##
####################################################################################
####################################################################################

CONNECTION_STRING = os.environ.get("DATABASE_URL")

if CONNECTION_STRING is None:
    print("No DATABASE_URL environment variable found")
    exit(1)

# pgml.setup_logger("DEBUG")

def generate_dummy_documents(count: int) -> List[Dict[str, Any]]:
    dummy_documents = []
    for i in range(count):
        dummy_documents.append({
            "id": i,
            "text": "This is a test document: {}".format(i),
            "metadata": {
                "uuid": i * 10,
                "name": "Test Document {}".format(i)
            }
        })
    return dummy_documents

@pytest.mark.asyncio
async def test_can_lazily_create_collection():
    collection_name = "p_ccc_test_5"
    collection = pgml.Collection(name=collection_name)
    builtins = pgml.Builtins()
    does_collection_exist = await builtins.does_collection_exist(collection_name)
    assert not does_collection_exist
    # Do something that requires the collection to be created
    await collection.upsert_documents(generate_dummy_documents(1))
    does_collection_exist = await builtins.does_collection_exist(collection_name)
    # Now the collection will exist because it had to be created to upsert documents
    await collection.archive()
    assert does_collection_exist

@pytest.mark.asyncio
async def test_can_lazily_create_model():
    model = pgml.Model()
    assert not model.get_verified_in_database()
    id = await model.get_id()
    assert id is not None
    assert model.get_verified_in_database()

@pytest.mark.asyncio
async def test_can_lazily_create_splitter():
    splitter = pgml.Splitter()
    assert not splitter.get_verified_in_database()
    id = await splitter.get_id()
    assert id is not None
    assert splitter.get_verified_in_database()

@pytest.mark.asyncio
async def test_can_vector_search():
    collection_name = "p_cvs_test_0"
    collection = pgml.Collection(name=collection_name)
    model = pgml.Model()
    splitter = pgml.Splitter()
    await collection.upsert_documents(generate_dummy_documents(2))
    # Splitter should not be verified in the database yet
    assert not splitter.get_verified_in_database()
    await collection.generate_chunks(splitter)
    # Now splitter should be verified in the database
    assert splitter.get_verified_in_database()
    # Model should not be verified in the database yet
    assert not model.get_verified_in_database()
    await collection.generate_embeddings(model, splitter)
    # Now model should be verified in the database
    assert model.get_verified_in_database()
    results = await collection.vector_search("Here is some query", model, splitter)
    await collection.archive()
    assert len(results) > 0

@pytest.mark.asyncio
async def test_can_vector_search_with_remote_embeddings():
    collection_name = "p_cvswre_test_0"
    collection = pgml.Collection(name=collection_name)
    model = pgml.Model(name="text-embedding-ada-002", source="openai")
    splitter = pgml.Splitter()
    await collection.upsert_documents(generate_dummy_documents(2))
    await collection.generate_chunks(splitter)
    await collection.generate_embeddings(model, splitter)
    results = await collection.vector_search("Here is some query", model, splitter)
    assert len(results) > 0

@pytest.mark.asyncio
async def test_can_vector_search_with_query_builder():
    collection_name = "p_cvswqb_test_0"
    collection = pgml.Collection(name=collection_name)
    model = pgml.Model()
    splitter = pgml.Splitter()
    await collection.upsert_documents(generate_dummy_documents(2))
    await collection.generate_chunks(splitter)
    await collection.generate_embeddings(model, splitter)
    await collection.generate_tsvectors()
    results = await collection.query().vector_recall("Here is some query", model, splitter).filter({
        "metadata": {
            "metadata": {
                "$or": [
                    {"uuid": {"$eq": 0 }},
                    {"uuid": {"$eq": 10 }},
                    {"category": {"$eq": [1, 2, 3]}}
                ]

            }
        },
        "full_text": {
            "text": "Test document"
        }
    }).run()
    await collection.archive()
    assert len(results) > 0
