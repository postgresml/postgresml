import os
import asyncio
import pgml
# import pytest
from typing import List, Dict, Any

CONNECTION_STRING = os.environ.get("DATABASE_URL")

if CONNECTION_STRING is None:
    print("No DATABASE_URL environment variable found")
    exit(1)

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

async def test_can_create_collection():
    collection_name = "p_ccc_test_0"
    db = pgml.Database(CONNECTION_STRING)
    _ = await db.create_or_get_collection(collection_name)
    does_collection_exist = await db.does_collection_exist(collection_name)
    assert does_collection_exist

if __name__ == "__main__":
    asyncio.run(test_can_create_collection())


# @pytest.mark.asyncio
# async def test_can_connect_to_database():
#     db = pgml.Database(CONNECTION_STRING)
#     assert db is not None
#
# @pytest.mark.asyncio
# async def test_can_create_collection():
#     collection_name = "p_ccc_test_0"
#     db = pgml.Database(CONNECTION_STRING)
#     _ = await db.create_or_get_collection(collection_name)
#     does_collection_exist = await db.does_collection_exist(collection_name)
#     assert does_collection_exist
#
# @pytest.mark.asyncio
# async def test_can_register_and_get_model():
#     db = pgml.Database(CONNECTION_STRING)
#     model = await db.register_model()
#     model_from_db = await db.get_model(model.get_id())
#     assert model.get_id() == model_from_db.get_id()
#
# @pytest.mark.asyncio
# async def test_can_register_and_get_text_splitter():
#     db = pgml.Database(CONNECTION_STRING)
#     text_splitter = await db.register_text_splitter()
#     text_splitter_from_db = await db.get_text_splitter(text_splitter.get_id())
#     assert text_splitter.get_id() == text_splitter_from_db.get_id()
#
# @pytest.mark.asyncio
# async def test_can_vector_search():
#     db = pgml.Database(CONNECTION_STRING)
#     collection_name = "p_cvs_test_0"
#     collection = await db.create_or_get_collection(collection_name)
#     model = await db.register_model()
#     text_splitter = await db.register_text_splitter()
#     await collection.upsert_documents(generate_dummy_documents(2))
#     await collection.generate_chunks(text_splitter)
#     await collection.generate_embeddings(model, text_splitter)
#     results = await collection.vector_search("Here is some query", model, text_splitter)
#     await db.archive_collection(collection_name)
#     assert len(results) != 0
#
# @pytest.mark.asyncio
# async def test_can_vector_search_with_remote_embeddings():
#     db = pgml.Database(CONNECTION_STRING)
#     collection_name = "p_cvs_test_1"
#     collection = await db.create_or_get_collection(collection_name)
#     model = await db.register_model(model_name="text-embedding-ada-002", model_source="openai")
#     text_splitter = await db.register_text_splitter()
#     await collection.upsert_documents(generate_dummy_documents(2))
#     await collection.generate_chunks(text_splitter)
#     await collection.generate_embeddings(model, text_splitter)
#     results = await collection.vector_search("Here is some query", model, text_splitter)
#     await db.archive_collection(collection_name)
#     assert len(results) != 0 

# async def main():
#     collection_name = "ptest22"
#     db = pgml.Database(CONNECTION_STRING)
#     collection = await db.create_or_get_collection(collection_name)
#     print("The Collection")
#     print(collection)
#     collection_does_exist = await db.does_collection_exist(collection_name)
#     print("Collection does exist")
#     print(collection_does_exist)
#     x = [{'id': '5733be284776f41900661182', 'text': 'Architecturally, the school has a Catholic character. Atop the Main Building\'s gold dome is a golden statue of the Virgin Mary. Immediately in front of the Main Building and facing it, is a copper statue of Christ with arms upraised with the legend "Venite Ad Me Omnes". Next to the Main Building is the Basilica of the Sacred Heart. Immediately behind the basilica is the Grotto, a Marian place of prayer and reflection. It is a replica of the grotto at Lourdes, France where the Virgin Mary reputedly appeared to Saint Bernadette Soubirous in 1858. At the end of the main drive (and in a direct line that connects through 3 statues and the Gold Dome), is a simple, modern stone statue of Mary.', 'title': 'University_of_Notre_Dame'}] 
#     await collection.upsert_documents(x)
#     await collection.register_text_splitter("recursive_character", {"chunk_size": 1500, "chunk_overlap": 40})
#     splitters = await collection.get_text_splitters()
#     print("The Splitters")
#     print(splitters)
#     await collection.generate_chunks()
#     await collection.register_model("embedding", "intfloat/e5-small")
#     models = await collection.get_models()
#     print("The Models")
#     print(models)
#     await collection.generate_embeddings()
#     results = await collection.vector_search("small")
#     print("The Results")
#     print(results)
#     await db.archive_collection(collection_name)
#
# async def query_builder():
#     collection_name = "pqtest2"
#     db = pgml.Database(CONNECTION_STRING)
#     collection = await db.create_or_get_collection(collection_name)
#     print("The collection:")
#     print(collection)
#     documents = [
#         {
#             "id": 1,
#             "metadata": {
#                 "uuid": 1
#             },
#             "text": "This is a test document",
#         },
#         {
#             "id": 2,
#             "metadata": {
#                 "uuid": 2
#             },
#             "text": "This is another test document",
#         },
#         {
#             "id": 3,
#             "metadata": {
#                 "uuid": 3
#             },
#             "text": "PostgresML",
#         }
#
#     ]
#     await collection.upsert_documents(documents)
#     await collection.generate_tsvectors('english')
#     await collection.generate_chunks()
#     await collection.generate_embeddings()
#
#     query = collection.query().vector_recall("test").filter({
#         "metadata": {
#             "metadata": {
#                 "$or": [
#                     {"uuid": {"$eq": 1}},
#                     {"uuid": {"$lt": 4}}
#                 ]
#             }
#         },
#         "full_text": {
#             "text": "postgresml"
#         }
#     }).limit(10)
#     print("Running query:")
#     print(query.to_full_string())
#     results = await query.run()
#     print("The results:")
#     print(results)
#
#     # await db.archive_collection(collection_name)
#
# async def query_runner():
#     db = pgml.Database(CONNECTION_STRING)
#     # results = await db.query("SELECT * from pgml.collections WHERE id = $1").bind_int(1).fetch_all()
#     results = await db.query("SELECT * from pgml.collections").fetch_all()
#     print(results)
#
# async def transform():
#     db = pgml.Database(CONNECTION_STRING)
#     # results = await db.query("SELECT * from pgml.collections WHERE id = $1").bind_int(1).fetch_all()
#     results = await db.transform("translation_en_to_fr", ["This is a test", "This is a test 2"])
#     print(results)
#
#
# if __name__ == "__main__":
#     asyncio.run(query_builder())    
#     # asyncio.run(main())    
#     # asyncio.run(query_runner())
#     # asyncio.run(transform())
