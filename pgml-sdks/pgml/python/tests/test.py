import os
import pgml
import pytest
from multiprocessing import Pool
from typing import List, Dict, Any
import asyncio

####################################################################################
####################################################################################
## PLEASE BE AWARE THESE TESTS DO INVOLVE CHECKS ON LAZILY CREATED DATABASE ITEMS ##
## IF ANY OF THE COLLECTION NAMES ALREADY EXIST, SOME TESTS MAY FAIL              ##
## THIS DOES NOT MEAN THE SDK IS BROKEN. PLEASE CLEAR YOUR DATABASE INSTANCE      ##
## BEFORE RUNNING ANY TESTS                                                       ##
####################################################################################
####################################################################################

pgml.init_logger()


def generate_dummy_documents(count: int) -> List[Dict[str, Any]]:
    dummy_documents = []
    for i in range(count):
        dummy_documents.append(
            {
                "id": i,
                "title": "Test Document {}".format(i),
                "body": "Test body {}".format(i),
                "text": "This is a test document: {}".format(i),
                "project": "a10",
                "floating_uuid": i * 1.01,
                "uuid": i * 10,
                "test": None,
                "name": "Test Document {}".format(i),
            }
        )
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
    pipeline = pgml.Pipeline("test_p_p_tccp_0", {})
    assert pipeline is not None


def test_can_create_single_field_pipeline():
    model = pgml.Model()
    splitter = pgml.Splitter()
    pipeline = pgml.SingleFieldPipeline("test_p_p_tccsfp_0", model, splitter, {})
    assert pipeline is not None


def test_can_create_builtins():
    builtins = pgml.Builtins()
    assert builtins is not None

@pytest.mark.asyncio
async def test_can_embed_with_builtins():
    builtins = pgml.Builtins()
    result = await builtins.embed("intfloat/e5-small-v2", "test")
    assert result is not None

@pytest.mark.asyncio
async def test_can_embed_batch_with_builtins():
    builtins = pgml.Builtins()
    result = await builtins.embed_batch("intfloat/e5-small-v2", ["test"])
    assert result is not None


###################################################
## Test searches ##################################
###################################################


@pytest.mark.asyncio
async def test_can_search():
    pipeline = pgml.Pipeline(
        "test_p_p_tcs_0",
        {
            "title": {
                "semantic_search": {
                    "model": "intfloat/e5-small-v2",
                    "parameters": {"prompt": "passage: "},
                }
            },
            "body": {
                "splitter": {"model": "recursive_character"},
                "semantic_search": {
                    "model": "text-embedding-ada-002",
                    "source": "openai",
                },
                "full_text_search": {"configuration": "english"},
            },
        },
    )
    collection = pgml.Collection("test_p_c_tsc_13")
    await collection.add_pipeline(pipeline)
    await collection.upsert_documents(generate_dummy_documents(5))
    results = await collection.search(
        {
            "query": {
                "full_text_search": {"body": {"query": "Test", "boost": 1.2}},
                "semantic_search": {
                    "title": {
                        "query": "This is a test",
                        "parameters": {"prompt": "passage: "},
                        "boost": 2.0,
                    },
                    "body": {"query": "This is the body test", "boost": 1.01},
                },
                "filter": {"id": {"$gt": 1}},
            },
            "limit": 5,
        },
        pipeline,
    )
    ids = [result["id"] for result in results["results"]]
    assert ids == [3, 5, 4]
    await collection.archive()


###################################################
## Test various vector searches ###################
###################################################


@pytest.mark.asyncio
async def test_can_vector_search():
    pipeline = pgml.Pipeline(
        "test_p_p_tcvs_0",
        {
            "title": {
                "semantic_search": {
                    "model": "intfloat/e5-small-v2",
                    "parameters": {"prompt": "passage: "},
                },
                "full_text_search": {"configuration": "english"},
            },
            "text": {
                "splitter": {"model": "recursive_character"},
                "semantic_search": {
                    "model": "intfloat/e5-small-v2",
                    "parameters": {"prompt": "passage: "},
                },
            },
        },
    )
    collection = pgml.Collection("test_p_c_tcvs_3")
    await collection.add_pipeline(pipeline)
    await collection.upsert_documents(generate_dummy_documents(5))
    results = await collection.vector_search(
        {
            "query": {
                "fields": {
                    "title": {
                        "query": "Test document: 2",
                        "parameters": {"prompt": "passage: "},
                        "full_text_filter": "test",
                    },
                    "text": {
                        "query": "Test document: 2",
                        "parameters": {"prompt": "passage: "},
                    },
                },
                "filter": {"id": {"$gt": 2}},
            },
            "limit": 5,
        },
        pipeline,
    )
    ids = [result["document"]["id"] for result in results]
    assert ids == [3, 3, 4, 4]
    await collection.archive()


@pytest.mark.asyncio
async def test_can_vector_search_with_query_builder():
    model = pgml.Model("intfloat/e5-small-v2", "pgml", {"prompt": "passage: "})
    splitter = pgml.Splitter()
    pipeline = pgml.SingleFieldPipeline("test_p_p_tcvswqb_1", model, splitter)
    collection = pgml.Collection(name="test_p_c_tcvswqb_5")
    await collection.upsert_documents(generate_dummy_documents(3))
    await collection.add_pipeline(pipeline)
    results = (
        await collection.query()
        .vector_recall("Here is some query", pipeline)
        .limit(10)
        .fetch_all()
    )
    ids = [document["id"] for (_, _, document) in results]
    assert ids == [1, 2, 0]
    await collection.archive()


###################################################
## Test RAG #######################################
###################################################


@pytest.mark.asyncio
async def test_can_rag():
    pipeline = pgml.Pipeline(
        "1",
        {
            "body": {
                "splitter": {"model": "recursive_character"},
                "semantic_search": {
                    "model": "intfloat/e5-small-v2",
                    "parameters": {"prompt": "passage: "},
                },
            },
        },
    )
    collection = pgml.Collection("test_p_c_cr")
    await collection.add_pipeline(pipeline)
    await collection.upsert_documents(generate_dummy_documents(5))
    results = await collection.rag(
        {
            "CONTEXT": {
                "vector_search": {
                    "query": {
                        "fields": {
                            "body": {
                                "query": "test",
                                "parameters": {"prompt": "query: "},
                            },
                        },
                    },
                    "document": {"keys": ["id"]},
                    "limit": 5,
                },
                "aggregate": {"join": "\n"},
            },
            "completion": {
                "model": "meta-llama/Meta-Llama-3-8B-Instruct",
                "prompt": "Some text with {CONTEXT}",
                "max_tokens": 10,
            },
        },
        pipeline,
    )
    assert len(results["rag"][0]) > 0
    assert len(results["sources"]["CONTEXT"]) > 0
    await collection.archive()


@pytest.mark.asyncio
async def test_can_rag_stream():
    pipeline = pgml.Pipeline(
        "1",
        {
            "body": {
                "splitter": {"model": "recursive_character"},
                "semantic_search": {
                    "model": "intfloat/e5-small-v2",
                    "parameters": {"prompt": "passage: "},
                },
            },
        },
    )
    collection = pgml.Collection("test_p_c_crs")
    await collection.add_pipeline(pipeline)
    await collection.upsert_documents(generate_dummy_documents(5))
    results = await collection.rag_stream(
        {
            "CONTEXT": {
                "vector_search": {
                    "query": {
                        "fields": {
                            "body": {
                                "query": "test",
                                "parameters": {"prompt": "query: "},
                            },
                        },
                    },
                    "document": {"keys": ["id"]},
                    "limit": 5,
                },
                "aggregate": {"join": "\n"},
            },
            "completion": {
                "model": "meta-llama/Meta-Llama-3-8B-Instruct",
                "prompt": "Some text with {CONTEXT}",
                "max_tokens": 10,
            },
        },
        pipeline,
    )
    async for c in results.stream():
        assert len(c) > 0
    await collection.archive()


###################################################
## Test document related functions ################
###################################################


@pytest.mark.asyncio
async def test_upsert_and_get_documents():
    collection = pgml.Collection("test_p_c_tuagd_2")
    await collection.upsert_documents(generate_dummy_documents(10))
    documents = await collection.get_documents()
    assert len(documents) == 10
    documents = await collection.get_documents(
        {"offset": 1, "limit": 2, "filter": {"id": {"$gt": 0}}}
    )
    assert len(documents) == 2 and documents[0]["document"]["id"] == 2
    last_row_id = documents[-1]["row_id"]
    documents = await collection.get_documents(
        {
            "filter": {
                "id": {"$lt": 7},
            },
            "last_row_id": last_row_id,
        }
    )
    assert len(documents) == 3 and documents[0]["document"]["id"] == 4
    await collection.archive()


@pytest.mark.asyncio
async def test_delete_documents():
    collection = pgml.Collection("test_p_c_tdd_1")
    await collection.upsert_documents(generate_dummy_documents(3))
    await collection.delete_documents(
        {
            "id": {"$gte": 2},
        }
    )
    documents = await collection.get_documents()
    assert len(documents) == 2 and documents[0]["document"]["id"] == 0
    await collection.archive()


@pytest.mark.asyncio
async def test_order_documents():
    collection = pgml.Collection("test_p_c_tod_0")
    await collection.upsert_documents(generate_dummy_documents(3))
    documents = await collection.get_documents({"order_by": {"id": "desc"}})
    assert len(documents) == 3
    assert documents[0]["document"]["id"] == 2
    await collection.archive()


###################################################
## Transformer Pipeline Tests #####################
###################################################


@pytest.mark.asyncio
async def test_transformer_pipeline():
    t = pgml.TransformerPipeline(
        "text-generation", "meta-llama/Meta-Llama-3-8B-Instruct"
    )
    it = await t.transform(["AI is going to"], {"max_new_tokens": 5})
    assert len(it) > 0


@pytest.mark.asyncio
async def test_transformer_pipeline_stream():
    t = pgml.TransformerPipeline(
        "text-generation", "meta-llama/Meta-Llama-3-8B-Instruct"
    )
    it = await t.transform_stream("AI is going to", {"max_tokens": 5})
    total = []
    async for c in it:
        total.append(c)
    assert len(total) > 0


###################################################
## OpenSourceAI tests ###########################
###################################################


def test_open_source_ai_create():
    client = pgml.OpenSourceAI()
    results = client.chat_completions_create(
        "meta-llama/Meta-Llama-3-8B-Instruct",
        [
            {
                "role": "system",
                "content": "You are a friendly chatbot who always responds in the style of a pirate",
            },
            {
                "role": "user",
                "content": "How many helicopters can a human eat in one sitting?",
            },
        ],
        max_tokens=10,
        temperature=0.85,
    )
    assert len(results["choices"]) > 0


@pytest.mark.asyncio
async def test_open_source_ai_create_async():
    client = pgml.OpenSourceAI()
    results = await client.chat_completions_create_async(
        "meta-llama/Meta-Llama-3-8B-Instruct",
        [
            {
                "role": "system",
                "content": "You are a friendly chatbot who always responds in the style of a pirate",
            },
            {
                "role": "user",
                "content": "How many helicopters can a human eat in one sitting?",
            },
        ],
        max_tokens=10,
        temperature=0.85,
    )
    assert len(results["choices"]) > 0


def test_open_source_ai_create_stream():
    client = pgml.OpenSourceAI()
    results = client.chat_completions_create_stream(
        "meta-llama/Meta-Llama-3-8B-Instruct",
        [
            {
                "role": "system",
                "content": "You are a friendly chatbot who always responds in the style of a pirate",
            },
            {
                "role": "user",
                "content": "How many helicopters can a human eat in one sitting?",
            },
        ],
        temperature=0.85,
        n=3,
    )
    output = []
    for c in results:
        output.append(c["choices"])
    assert len(output) > 0


@pytest.mark.asyncio
async def test_open_source_ai_create_stream_async():
    client = pgml.OpenSourceAI()
    results = await client.chat_completions_create_stream_async(
        "meta-llama/Meta-Llama-3-8B-Instruct",
        [
            {
                "role": "system",
                "content": "You are a friendly chatbot who always responds in the style of a pirate",
            },
            {
                "role": "user",
                "content": "How many helicopters can a human eat in one sitting?",
            },
        ],
        temperature=0.85,
        n=3,
    )
    output = []
    async for c in results:
        output.append(c["choices"])
    assert len(output) > 0


###################################################
## Migration tests ################################
###################################################


@pytest.mark.asyncio
async def test_migrate():
    await pgml.migrate()


###################################################
## Test with multiprocessing ######################
###################################################


# def vector_search(collection_name, pipeline_name):
#     collection = pgml.Collection(collection_name)
#     pipeline = pgml.Pipeline(pipeline_name)
#     result = asyncio.run(
#         collection.query()
#         .vector_recall("Here is some query", pipeline)
#         .limit(10)
#         .fetch_all()
#     )
#     print(result)
#     return [0, 1, 2]


# @pytest.mark.asyncio
# async def test_multiprocessing():
#     collection_name = "test_p_p_tm_1"
#     pipeline_name = "test_p_c_tm_4"
#
#     model = pgml.Model()
#     splitter = pgml.Splitter()
#     pipeline = pgml.Pipeline(pipeline_name, model, splitter)
#
#     collection = pgml.Collection(collection_name)
#     await collection.upsert_documents(generate_dummy_documents(3))
#     await collection.add_pipeline(pipeline)
#
#     with Pool(5) as p:
#         results = p.starmap(
#             vector_search, [(collection_name, pipeline_name) for _ in range(5)]
#         )
#         for x in results:
#             print(x)
#             assert len(x) == 3
#
#     await collection.archive()
