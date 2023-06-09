import asyncio
import pgml

async def main():
    collection_name = "ptest20"
    # db = pgml_sdk.DatabasePython("postgres://localhost:28815/pgml")
    db = pgml.Database("postgres://postgres@127.0.0.1:5433/pgml_development")
    collection = await db.create_or_get_collection(collection_name)
    print(collection)
    # x = [
    #     {"text": "It was the best of times, it was the worst of times, it was the age of wisdom"},
    #     {"text": "it was the age of foolishness, it was the epoch of belief, it was the epoch of incredulity, it was the season of Light"},
    #     {"text": "it was the season of Darkness, it was the spring of hope, it was the winter of despair, we had everything before us"},
    #     {"text": "IT WAS LEGENDARY"},
    #     {"text": "THIS NEEDS TO CHANGE TIMES 2"},
    #     {"text": "small"},
    # ]
    x = [{'id': '5733be284776f41900661182', 'text': 'Architecturally, the school has a Catholic character. Atop the Main Building\'s gold dome is a golden statue of the Virgin Mary. Immediately in front of the Main Building and facing it, is a copper statue of Christ with arms upraised with the legend "Venite Ad Me Omnes". Next to the Main Building is the Basilica of the Sacred Heart. Immediately behind the basilica is the Grotto, a Marian place of prayer and reflection. It is a replica of the grotto at Lourdes, France where the Virgin Mary reputedly appeared to Saint Bernadette Soubirous in 1858. At the end of the main drive (and in a direct line that connects through 3 statues and the Gold Dome), is a simple, modern stone statue of Mary.', 'title': 'University_of_Notre_Dame'}] 
    await collection.upsert_documents(x)
    await collection.register_text_splitter()
    splitters = await collection.get_text_splitters()
    print(splitters)
    await collection.generate_chunks()
    await collection.register_model("embedding", "intfloat/e5-small")
    models = await collection.get_models()
    print(models)
    await collection.generate_embeddings()
    results = await collection.vector_search("small", {}, 2);
    print(results)
    await db.archive_collection(collection_name)

if __name__ == "__main__":
    asyncio.run(main())    
