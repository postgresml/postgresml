import os
import pgml
import pytest
from multiprocessing import Pool
from typing import List, Dict, Any

####################################################################################
####################################################################################
## PLEASE BE AWARE THESE TESTS DO INVOLVE CHECKS ON LAZILY CREATED DATABASE ITEMS ##
## IF ANY OF THE COLLECTION NAMES ALREADY EXIST, SOME TESTS MAY FAIL              ##
## THIS DOES NOT MEAN THE SDK IS BROKEN. PLEASE CLEAR YOUR DATABASE INSTANCE      ##
## BEFORE RUNNING ANY TESTS                                                       ##
####################################################################################
####################################################################################

DATABASE_URL = os.environ.get("DATABASE_URL")
if DATABASE_URL is None:
    print("No DATABASE_URL environment variable found. Please set one")
    exit(1)

LOG_LEVEL = os.environ.get("LOG_LEVEL")
if LOG_LEVEL is None:
    print("No LOG_LEVEL environment variable found setting to ERROR")
    LOG_LEVEL = "ERROR"
pgml.py_init_logger(LOG_LEVEL)

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


###################################################
## Test the API exposed is correct ################
###################################################

def test_can_create_collection():
    collection = pgml.Collection(name="test_p_c_tscc_0")
    assert collection is not None

def test_can_create_model():
    model = pgml.Model()
    assert model is not None

def test_can_create_splitter():
    splitter = pgml.Splitter()
    assert splitter is not None

def test_can_create_pipeline():
    model = pgml.Model()
    splitter = pgml.Splitter()
    pipeline = pgml.Pipeline("test_p_p_tccp_0", model, splitter)
    assert pipeline is not None

def test_can_create_builtins():
    builtins = pgml.Builtins()
    assert builtins is not None


###################################################
## Test various vector searches ###################
###################################################

@pytest.mark.asyncio
async def test_can_vector_search():
    model = pgml.Model()
    splitter = pgml.Splitter()
    pipeline = pgml.Pipeline("test_p_p_tcvs_0", model, splitter)
    collection = pgml.Collection(name="test_p_c_tcvs_3")
    await collection.upsert_documents(generate_dummy_documents(3))
    await collection.add_pipeline(pipeline)
    results = await collection.vector_search("Here is some query", pipeline)
    assert len(results) == 3
    await collection.archive()

@pytest.mark.asyncio
async def test_can_vector_search_with_remote_embeddings():
    model = pgml.Model(name="text-embedding-ada-002", source="openai")
    splitter = pgml.Splitter()
    pipeline = pgml.Pipeline("test_p_p_tcvswre_0", model, splitter)
    collection = pgml.Collection(name="test_p_c_tcvswre_2")
    await collection.upsert_documents(generate_dummy_documents(3))
    await collection.add_pipeline(pipeline)
    results = await collection.vector_search("Here is some query", pipeline)
    assert len(results) == 3
    await collection.archive()

@pytest.mark.asyncio
async def test_can_vector_search_with_query_builder():
    model = pgml.Model()
    splitter = pgml.Splitter()
    pipeline = pgml.Pipeline("test_p_p_tcvswqb_1", model, splitter)
    collection = pgml.Collection(name="test_p_c_tcvswqb_4")
    await collection.upsert_documents(generate_dummy_documents(3))
    await collection.add_pipeline(pipeline)
    results = await collection.query().vector_recall("Here is some query", pipeline).limit(10).run()
    await collection.archive()
    assert len(results) == 3

@pytest.mark.asyncio
async def test_can_vector_search_with_query_builder_with_remote_embeddings():
    model = pgml.Model(name="text-embedding-ada-002", source="openai")
    splitter = pgml.Splitter()
    pipeline = pgml.Pipeline("test_p_p_tcvswqbwre_1", model, splitter)
    collection = pgml.Collection(name="test_p_c_tcvswqbwre_0")
    await collection.upsert_documents(generate_dummy_documents(3))
    await collection.add_pipeline(pipeline)
    results = await collection.query().vector_recall("Here is some query", pipeline).limit(10).run()
    await collection.archive()
    assert len(results) == 3


###################################################
## Test user output facing functions ##############
###################################################

@pytest.mark.asyncio
async def test_pipeline_to_dict():
    model = pgml.Model(name="text-embedding-ada-002", source="openai")
    splitter = pgml.Splitter()
    pipeline = pgml.Pipeline("test_p_p_tptd_1", model, splitter)
    collection = pgml.Collection(name="test_p_c_tptd_1")
    await collection.add_pipeline(pipeline)
    pipeline_dict = await pipeline.to_dict()
    assert pipeline_dict["name"] == "test_p_p_tptd_1"
    await collection.remove_pipeline(pipeline, {"delete": True})


###################################################
## Test with multiprocessing ######################
###################################################
# async def vector_search(collection, pipeline):
#     results = await collection.query().vector_recall("Here is some query", pipeline).limit(10).run()
#     return len(results)
#
# @pytest.mark.asyncio
# async def test_multiprocessing():
#     model = pgml.Model()
#     splitter = pgml.Splitter()
#     pipeline = pgml.Pipeline("test_p_p_tm_1", model, splitter)
#     collection = pgml.Collection(name="test_p_c_tm_4")
#     await collection.upsert_documents(generate_dummy_documents(3))
#     await collection.add_pipeline(pipeline)
#
#      with Pool(5) as p:
#         results = p.map_async(vector_search, [(collection, pipeline), (co)])
#         for x in results.get():
#             assert(x == 3)
#
#     await collection.archive()
