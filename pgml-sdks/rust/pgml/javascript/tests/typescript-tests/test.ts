import pgml from '../../index.js'

////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////
// PLEASE BE AWARE THESE TESTS DO INVOLVE CHECKS ON LAZILY CREATD DATABASE ITEMS. //
// IF ANY OF THE COLLECTION NAMES ALREADY EXIST, SOME TESTS MAY FAIL              //
// THIS DOES NOT MEAN THE SDK IS BROKEN. PLEASE CLEAR YOUR DATABASE INSTANCE      //
// BEFORE RUNNING ANY TESTS                                                       //
////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////

const generate_documents = (count: number) => {
  let docs = [];
  for (let i = 0; i < count; i++) {
    docs.push({
      "id": i,
      "text": `This is a test document: ${i}`,
      "metadata": {
        "uuid": i * 10,
        "name": `Test Document ${i}`
      }
    });
  }
  return docs;
}

it("can lazily create collection", async () => {
  let collection_name = "j_ccc_test_2";
  let collection = pgml.newCollection(collection_name);
  let builtins = pgml.newBuiltins();
  let does_collection_exist = await builtins.does_collection_exist(collection_name);
  expect(does_collection_exist).toBe(false);
  // Do something that requires the collection to be created
  await collection.upsert_documents(generate_documents(1));
  // Now the collection will exit because it had to be created to upsert documents
  does_collection_exist = await builtins.does_collection_exist(collection_name);
  await collection.archive();
  expect(does_collection_exist).toBe(true);
});

it("can lazily create model", async () => {
  let model = pgml.newModel();
  expect(model.get_verified_in_database()).toBe(false);
  let id = await model.get_id();
  expect(id).toBeDefined();
  expect(model.get_verified_in_database()).toBe(true);
})

it("can lazily create splitter", async () => {
  let splitter = pgml.newSplitter();
  expect(splitter.get_verified_in_database()).toBe(false);
  let id = await splitter.get_id();
  expect(id).toBeDefined();
  expect(splitter.get_verified_in_database()).toBe(true);
})

it("can vector search", async () => {
  let collection_name = "j_cvs_test_0";
  let collection = pgml.newCollection(collection_name);
  let model = pgml.newModel();
  let splitter = pgml.newSplitter();
  await collection.upsert_documents(generate_documents(2));
  // Splitter should not be verified in the database yet
  expect(splitter.get_verified_in_database()).toBe(false);
  await collection.generate_chunks(splitter);
  // Now splitter should be verified in the database
  expect(splitter.get_verified_in_database()).toBe(true);
  // Model should not be verified in the database yet
  expect(model.get_verified_in_database()).toBe(false);
  await collection.generate_embeddings(model, splitter);
  // Now model should be verified in the database
  expect(model.get_verified_in_database()).toBe(true);
  let results = await collection.vector_search("Here is some query", model, splitter);
  await collection.archive();
  expect(results.length).toBe(2);
})

it("can vector search with remote embeddings", async () => {
  let collection_name = "j_cvswre_test_0";
  let collection = pgml.newCollection(collection_name);
  let model = pgml.newModel("text-embedding-ada-002", "embeddings", "openai");
  let splitter = pgml.newSplitter();
  await collection.upsert_documents(generate_documents(2));
  await collection.generate_chunks(splitter);
  await collection.generate_embeddings(model, splitter);
  let results = await collection.vector_search("Here is some query", model, splitter);
  await collection.archive();
  expect(results.length).toBe(2);
})

it("can vector search with query builder", async () => {
  let collection_name = "j_cvswqb_test_0";
  let collection = pgml.newCollection(collection_name);
  let model = pgml.newModel();
  let splitter = pgml.newSplitter();
  await collection.upsert_documents(generate_documents(2));
  await collection.generate_chunks(splitter);
  await collection.generate_embeddings(model, splitter);
  await collection.generate_tsvectors();
  let results = await collection.query().vector_recall("Here is some query", model, splitter).filter({
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
  await collection.archive();
  expect(results.length).toBe(2);
})
