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

DATABASE_URL = os.environ.get("DATABASE_URL")
if DATABASE_URL is None:
    print("No DATABASE_URL environment variable found. Please set one")
    exit(1)

pgml.init_logger()


def generate_dummy_documents(count: int) -> List[Dict[str, Any]]:
    dummy_documents = []
    for i in range(count):
        dummy_documents.append(
            {
                "id": i,
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
async def test_can_vector_search_with_local_embeddings():
    model = pgml.Model()
    splitter = pgml.Splitter()
    pipeline = pgml.Pipeline("test_p_p_tcvs_0", model, splitter)
    collection = pgml.Collection(name="test_p_c_tcvs_4")
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
    collection = pgml.Collection(name="test_p_c_tcvswre_3")
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
    collection = pgml.Collection(name="test_p_c_tcvswqb_5")
    await collection.upsert_documents(generate_dummy_documents(3))
    await collection.add_pipeline(pipeline)
    results = (
        await collection.query()
        .vector_recall("Here is some query", pipeline)
        .limit(10)
        .fetch_all()
    )
    assert len(results) == 3
    await collection.archive()


@pytest.mark.asyncio
async def test_can_vector_search_with_query_builder_with_remote_embeddings():
    model = pgml.Model(name="text-embedding-ada-002", source="openai")
    splitter = pgml.Splitter()
    pipeline = pgml.Pipeline("test_p_p_tcvswqbwre_1", model, splitter)
    collection = pgml.Collection(name="test_p_c_tcvswqbwre_1")
    await collection.upsert_documents(generate_dummy_documents(3))
    await collection.add_pipeline(pipeline)
    results = (
        await collection.query()
        .vector_recall("Here is some query", pipeline)
        .limit(10)
        .fetch_all()
    )
    assert len(results) == 3
    await collection.archive()


@pytest.mark.asyncio
async def test_can_vector_search_with_query_builder_and_metadata_filtering():
    model = pgml.Model()
    splitter = pgml.Splitter()
    pipeline = pgml.Pipeline("test_p_p_tcvswqbamf_1", model, splitter)
    collection = pgml.Collection(name="test_p_c_tcvswqbamf_2")
    await collection.upsert_documents(generate_dummy_documents(3))
    await collection.add_pipeline(pipeline)
    results = (
        await collection.query()
        .vector_recall("Here is some query", pipeline)
        .filter(
            {
                "metadata": {
                    "$or": [{"uuid": {"$eq": 0}}, {"floating_uuid": {"$lt": 2}}],
                    "project": {"$eq": "a10"},
                },
            }
        )
        .limit(10)
        .fetch_all()
    )
    assert len(results) == 2
    await collection.archive()


@pytest.mark.asyncio
async def test_can_vector_search_with_query_builder_and_custom_hnsw_ef_search_value():
    model = pgml.Model()
    splitter = pgml.Splitter()
    pipeline = pgml.Pipeline("test_p_p_tcvswqbachesv_0", model, splitter)
    collection = pgml.Collection(name="test_p_c_tcvswqbachesv_0")
    await collection.upsert_documents(generate_dummy_documents(3))
    await collection.add_pipeline(pipeline)
    results = (
        await collection.query()
        .vector_recall("Here is some query", pipeline)
        .filter({"hnsw": {"ef_search": 2}})
        .limit(10)
        .fetch_all()
    )
    assert len(results) == 3
    await collection.archive()


@pytest.mark.asyncio
async def test_can_vector_search_with_query_builder_and_custom_hnsw_ef_search_value_and_remote_embeddings():
    model = pgml.Model(name="text-embedding-ada-002", source="openai")
    splitter = pgml.Splitter()
    pipeline = pgml.Pipeline("test_p_p_tcvswqbachesvare_0", model, splitter)
    collection = pgml.Collection(name="test_p_c_tcvswqbachesvare_0")
    await collection.upsert_documents(generate_dummy_documents(3))
    await collection.add_pipeline(pipeline)
    results = (
        await collection.query()
        .vector_recall("Here is some query", pipeline)
        .filter({"hnsw": {"ef_search": 2}})
        .limit(10)
        .fetch_all()
    )
    assert len(results) == 3
    await collection.archive()


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
    await collection.remove_pipeline(pipeline)
    await collection.archive()


###################################################
## Test document related functions ################
###################################################


@pytest.mark.asyncio
async def test_upsert_and_get_documents():
    model = pgml.Model()
    splitter = pgml.Splitter()
    pipeline = pgml.Pipeline(
        "test_p_p_tuagd_0",
        model,
        splitter,
        {"full_text_search": {"active": True, "configuration": "english"}},
    )
    collection = pgml.Collection(name="test_p_c_tuagd_2")
    await collection.add_pipeline(
        pipeline,
    )
    await collection.upsert_documents(generate_dummy_documents(10))

    documents = await collection.get_documents()
    assert len(documents) == 10

    documents = await collection.get_documents(
        {"offset": 1, "limit": 2, "filter": {"metadata": {"id": {"$gt": 0}}}}
    )
    assert len(documents) == 2 and documents[0]["document"]["id"] == 2
    last_row_id = documents[-1]["row_id"]

    documents = await collection.get_documents(
        {
            "filter": {
                "metadata": {"id": {"$gt": 3}},
                "full_text_search": {"configuration": "english", "text": "4"},
            },
            "last_row_id": last_row_id,
        }
    )
    assert len(documents) == 1 and documents[0]["document"]["id"] == 4

    await collection.archive()


@pytest.mark.asyncio
async def test_delete_documents():
    model = pgml.Model()
    splitter = pgml.Splitter()
    pipeline = pgml.Pipeline(
        "test_p_p_tdd_0",
        model,
        splitter,
        {"full_text_search": {"active": True, "configuration": "english"}},
    )
    collection = pgml.Collection("test_p_c_tdd_1")
    await collection.add_pipeline(pipeline)
    await collection.upsert_documents(generate_dummy_documents(3))
    await collection.delete_documents(
        {
            "metadata": {"id": {"$gte": 0}},
            "full_text_search": {"configuration": "english", "text": "0"},
        }
    )
    documents = await collection.get_documents()
    assert len(documents) == 2 and documents[0]["document"]["id"] == 1
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
    t = pgml.TransformerPipeline("text-generation")
    it = await t.transform(["AI is going to"], {"max_new_tokens": 5})
    assert len(it) > 0


@pytest.mark.asyncio
async def test_transformer_pipeline_stream():
    t = pgml.TransformerPipeline("text-generation")
    it = await t.transform_stream("AI is going to", {"max_new_tokens": 5})
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
        "HuggingFaceH4/zephyr-7b-beta",
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
    )
    assert len(results["choices"]) > 0


@pytest.mark.asyncio
async def test_open_source_ai_create_async():
    client = pgml.OpenSourceAI()
    results = await client.chat_completions_create_async(
        "HuggingFaceH4/zephyr-7b-beta",
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
    )
    assert len(results["choices"]) > 0


def test_open_source_ai_create_stream():
    client = pgml.OpenSourceAI()
    results = client.chat_completions_create_stream(
        "HuggingFaceH4/zephyr-7b-beta",
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
    for c in results:
        assert len(c["choices"]) > 0


@pytest.mark.asyncio
async def test_open_source_ai_create_stream_async():
    client = pgml.OpenSourceAI()
    results = await client.chat_completions_create_stream_async(
        "HuggingFaceH4/zephyr-7b-beta",
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
    async for c in results:
        assert len(c["choices"]) > 0


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


###################################################
## Manual tests ###################################
###################################################


# async def test_add_pipeline():
#     model = pgml.Model()
#     splitter = pgml.Splitter()
#     pipeline = pgml.Pipeline("silas_test_p_1", model, splitter)
#     collection = pgml.Collection(name="silas_test_c_10")
#     await collection.add_pipeline(pipeline)
#
# async def test_upsert_documents():
#     collection = pgml.Collection(name="silas_test_c_9")
#     await collection.upsert_documents(generate_dummy_documents(10))
#
# async def test_vector_search():
#     pipeline = pgml.Pipeline("silas_test_p_1")
#     collection = pgml.Collection(name="silas_test_c_9")
#     results = await collection.vector_search("Here is some query", pipeline)
#     print(results)

# asyncio.run(test_add_pipeline())
# asyncio.run(test_upsert_documents())
# asyncio.run(test_vector_search())
