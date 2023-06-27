const pgml = require('../index.node');

const CONNECTION_STRING = process.env.DATABASE_URL;

async function test() {
  let db = await pgml.newDatabase(CONNECTION_STRING);
  let collection_name = "jtest2"
  let collection = await db.create_or_get_collection(collection_name);
  console.log("The Collection:")
  console.log(collection)
  let doc = {
    "name": "Test",
    "text": "Hello, World! - From Javascript",
  }
  await collection.upsert_documents([doc]);
  await collection.register_text_splitter("recursive_character", {chunk_size: 1500, chunk_overlap: 4})
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
  await collection.generate_embeddings();
  let results = await collection.vector_search("small", {}, 2);
  console.log("The Results:")
  results.forEach((result) => {
    console.log(result);
  })
  await db.archive_collection(collection_name);
}

test().then(() => console.log("\nTests Done!")).catch((err) => console.log(err));
