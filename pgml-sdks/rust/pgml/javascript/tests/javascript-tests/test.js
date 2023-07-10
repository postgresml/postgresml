import pgml from '../../index.js'

const CONNECTION_STRING = process.env.DATABASE_URL;

async function vector_recall() {
  let db = await pgml.newDatabase(CONNECTION_STRING);
  let collection_name = "jtest7"
  let collection = await db.create_or_get_collection(collection_name);
  console.log("The Collection:")
  console.log(collection)
  let doc = {
    "name": "Test",
    "text": "Hello, World! - From Javascript",
  }
  await collection.upsert_documents([doc]);
  await collection.register_text_splitter("recursive_character", { chunk_size: 1500, chunk_overlap: 4 })
  let splitters = await collection.get_text_splitters();
  console.log("The Splitters:")
  splitters.forEach((splitter) => {
    console.log(splitter);
  })
  await collection.generate_chunks(2);
  await collection.register_model("embedding", "intfloat/e5-small");
  let models = await collection.get_models();
  console.log("The Models:")
  models.forEach((model) => {
    console.log(model);
  })
  await collection.generate_embeddings(1, 2);
  let results = await collection.vector_search("small", {}, 2, 1, 2);
  console.log("The Results:")
  results.forEach((result) => {
    console.log(result);
  })
  await db.archive_collection(collection_name);
}

async function query_builder() {
  let db = await pgml.newDatabase(CONNECTION_STRING);
  let collection_name = "jqtest1"
  let collection = await db.create_or_get_collection(collection_name);
  let docs = [
    {
      "name": "Test",
      "text": "Hello, World! - From Javascript",
    },
    {
      "name": "Test2",
      "text": "Hello, World2! - From Javascript",
    }
  ]
  await collection.upsert_documents(docs);
  await collection.generate_chunks();
  await collection.generate_embeddings();
  let results = await collection.query().vector_recall("Hello").limit(5).run();
  console.log("The Results:")
  results.forEach((result) => {
    console.log(result);
  })
  await db.archive_collection(collection_name);
}

async function main() {
  // await test();
  await query_builder();
}

main().then(() => console.log("\nTests Done!")).catch((err) => console.log(err));
