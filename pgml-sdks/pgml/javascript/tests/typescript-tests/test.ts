import pgml from "../../index.js";

////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////
// PLEASE BE AWARE THESE TESTS DO INVOLVE CHECKS ON LAZILY CREATD DATABASE ITEMS  //
// IF ANY OF THE COLLECTION NAMES ALREADY EXIST, SOME TESTS MAY FAIL              //
// THIS DOES NOT MEAN THE SDK IS BROKEN. PLEASE CLEAR YOUR DATABASE INSTANCE      //
// BEFORE RUNNING ANY TESTS                                                       //
////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////

const LOG_LEVEL = process.env.LOG_LEVEL ? process.env.LOG_LEVEL : "ERROR";
pgml.init_logger(LOG_LEVEL);

const generate_dummy_documents = (count: number) => {
  let docs = [];
  for (let i = 0; i < count; i++) {
    docs.push({
      id: i,
      text: `This is a test document: ${i}`,
      project: "a10",
      uuid: i * 10,
      floating_uuid: i * 1.1,
      test: null,
      name: `Test Document ${i}`,
    });
  }
  return docs;
};

///////////////////////////////////////////////////
// Test the API exposed is correct ////////////////
///////////////////////////////////////////////////

it("can create collection", () => {
  let collection = pgml.newCollection("test_j_c_ccc_0");
  expect(collection).toBeTruthy();
});

it("can create model", () => {
  let model = pgml.newModel("test", "openai", {
    some_example_parameter: "test 0123948712394871234987",
  });
  expect(model).toBeTruthy();
});

it("can create splitter", () => {
  let splitter = pgml.newSplitter();
  expect(splitter).toBeTruthy();
});

it("can create pipeline", () => {
  let model = pgml.newModel();
  let splitter = pgml.newSplitter();
  let pipeline = pgml.newPipeline("test_j_p_ccc_0", model, splitter);
  expect(pipeline).toBeTruthy();
});

it("can create builtins", () => {
  let builtins = pgml.newBuiltins();
  expect(builtins).toBeTruthy();
});

///////////////////////////////////////////////////
// Test various vector searches ///////////////////
///////////////////////////////////////////////////

it("can vector search with local embeddings", async () => {
  let model = pgml.newModel();
  let splitter = pgml.newSplitter();
  let pipeline = pgml.newPipeline("test_j_p_cvswle_0", model, splitter);
  let collection = pgml.newCollection("test_j_c_cvswle_3");
  await collection.upsert_documents(generate_dummy_documents(3));
  await collection.add_pipeline(pipeline);
  let results = await collection.vector_search("Here is some query", pipeline);
  expect(results).toHaveLength(3);
  await collection.archive();
});

it("can vector search with remote embeddings", async () => {
  let model = pgml.newModel("text-embedding-ada-002", "openai");
  let splitter = pgml.newSplitter();
  let pipeline = pgml.newPipeline("test_j_p_cvswre_0", model, splitter);
  let collection = pgml.newCollection("test_j_c_cvswre_1");
  await collection.upsert_documents(generate_dummy_documents(3));
  await collection.add_pipeline(pipeline);
  let results = await collection.vector_search("Here is some query", pipeline);
  expect(results).toHaveLength(3);
  await collection.archive();
});

it("can vector search with query builder", async () => {
  let model = pgml.newModel();
  let splitter = pgml.newSplitter();
  let pipeline = pgml.newPipeline("test_j_p_cvswqb_0", model, splitter);
  let collection = pgml.newCollection("test_j_c_cvswqb_1");
  await collection.upsert_documents(generate_dummy_documents(3));
  await collection.add_pipeline(pipeline);
  let results = await collection
    .query()
    .vector_recall("Here is some query", pipeline)
    .limit(10)
    .fetch_all();
  expect(results).toHaveLength(3);
  await collection.archive();
});

it("can vector search with query builder with remote embeddings", async () => {
  let model = pgml.newModel("text-embedding-ada-002", "openai");
  let splitter = pgml.newSplitter();
  let pipeline = pgml.newPipeline("test_j_p_cvswqbwre_0", model, splitter);
  let collection = pgml.newCollection("test_j_c_cvswqbwre_1");
  await collection.upsert_documents(generate_dummy_documents(3));
  await collection.add_pipeline(pipeline);
  let results = await collection
    .query()
    .vector_recall("Here is some query", pipeline)
    .limit(10)
    .fetch_all();
  expect(results).toHaveLength(3);
  await collection.archive();
});

it("can vector search with query builder and metadata filtering", async () => {
  let model = pgml.newModel();
  let splitter = pgml.newSplitter();
  let pipeline = pgml.newPipeline("test_j_p_cvswqbamf_0", model, splitter);
  let collection = pgml.newCollection("test_j_c_cvswqbamf_4");
  await collection.upsert_documents(generate_dummy_documents(3));
  await collection.add_pipeline(pipeline);
  let results = await collection
    .query()
    .vector_recall("Here is some query", pipeline)
    .filter({
      metadata: {
        $or: [{ uuid: { $eq: 0 } }, { floating_uuid: { $lt: 2 } }],
        project: { $eq: "a10" },
      },
    })
    .limit(10)
    .fetch_all();
  expect(results).toHaveLength(2);
  await collection.archive();
});

it("can vector search with query builder and custom hnsfw ef_search value", async () => {
  let model = pgml.newModel();
  let splitter = pgml.newSplitter();
  let pipeline = pgml.newPipeline("test_j_p_cvswqbachesv_0", model, splitter);
  let collection = pgml.newCollection("test_j_c_cvswqbachesv_0");
  await collection.upsert_documents(generate_dummy_documents(3));
  await collection.add_pipeline(pipeline);
  let results = await collection
    .query()
    .vector_recall("Here is some query", pipeline)
    .filter({
      hnsw: {
        ef_search: 2,
      },
    })
    .limit(10)
    .fetch_all();
  expect(results).toHaveLength(3);
  await collection.archive();
});

it("can vector search with query builder and custom hnsfw ef_search value and remote embeddings", async () => {
  let model = pgml.newModel("text-embedding-ada-002", "openai");
  let splitter = pgml.newSplitter();
  let pipeline = pgml.newPipeline(
    "test_j_p_cvswqbachesvare_0",
    model,
    splitter,
  );
  let collection = pgml.newCollection("test_j_c_cvswqbachesvare_0");
  await collection.upsert_documents(generate_dummy_documents(3));
  await collection.add_pipeline(pipeline);
  let results = await collection
    .query()
    .vector_recall("Here is some query", pipeline)
    .filter({
      hnsw: {
        ef_search: 2,
      },
    })
    .limit(10)
    .fetch_all();
  expect(results).toHaveLength(3);
  await collection.archive();
});

///////////////////////////////////////////////////
// Test user output facing functions //////////////
///////////////////////////////////////////////////

it("pipeline to dict", async () => {
  let model = pgml.newModel("text-embedding-ada-002", "openai");
  let splitter = pgml.newSplitter();
  let pipeline = pgml.newPipeline("test_j_p_ptd_0", model, splitter);
  let collection = pgml.newCollection("test_j_c_ptd_2");
  await collection.add_pipeline(pipeline);
  let pipeline_dict = await pipeline.to_dict();
  expect(pipeline_dict["name"]).toBe("test_j_p_ptd_0");
  await collection.archive();
});

///////////////////////////////////////////////////
// Test document related functions ////////////////
///////////////////////////////////////////////////

it("can upsert and get documents", async () => {
  let model = pgml.newModel();
  let splitter = pgml.newSplitter();
  let pipeline = pgml.newPipeline("test_p_p_cuagd_0", model, splitter, {
    full_text_search: { active: true, configuration: "english" },
  });
  let collection = pgml.newCollection("test_p_c_cuagd_1");
  await collection.add_pipeline(pipeline);
  await collection.upsert_documents(generate_dummy_documents(10));

  let documents = await collection.get_documents();
  expect(documents).toHaveLength(10);

  documents = await collection.get_documents({
    offset: 1,
    limit: 2,
    filter: { metadata: { id: { $gt: 0 } } },
  });
  expect(documents).toHaveLength(2);
  expect(documents[0]["document"]["id"]).toBe(2);
  let last_row_id = documents[1]["row_id"];

  documents = await collection.get_documents({
    filter: {
      metadata: { id: { $gt: 3 } },
      full_text_search: { configuration: "english", text: "4" },
    },
    last_row_id: last_row_id,
  });
  expect(documents).toHaveLength(1);
  expect(documents[0]["document"]["id"]).toBe(4);

  await collection.archive();
});

it("can delete documents", async () => {
  let model = pgml.newModel();
  let splitter = pgml.newSplitter();
  let pipeline = pgml.newPipeline(
    "test_p_p_cdd_0",
    model,
    splitter,

    { full_text_search: { active: true, configuration: "english" } },
  );
  let collection = pgml.newCollection("test_p_c_cdd_2");
  await collection.add_pipeline(pipeline);
  await collection.upsert_documents(generate_dummy_documents(3));
  await collection.delete_documents({
    metadata: { id: { $gte: 0 } },
    full_text_search: { configuration: "english", text: "0" },
  });
  let documents = await collection.get_documents();
  expect(documents).toHaveLength(2);
  expect(documents[0]["document"]["id"]).toBe(1);

  await collection.archive();
});

it("can order documents", async () => {
  let collection = pgml.newCollection("test_j_c_cod_0");
  await collection.upsert_documents(generate_dummy_documents(3));
  let documents = await collection.get_documents({
    order_by: {
      id: "desc",
    },
  });
  expect(documents).toHaveLength(3);
  expect(documents[0]["document"]["id"]).toBe(2);
  await collection.archive();
});

///////////////////////////////////////////////////
// Transformer Pipeline Tests /////////////////////
///////////////////////////////////////////////////

it("can transformer pipeline", async () => {
  const t = pgml.newTransformerPipeline("text-generation");
  const it = await t.transform(["AI is going to"], {max_new_tokens: 5});
  expect(it.length).toBeGreaterThan(0)
});

it("can transformer pipeline stream", async () => {
  const t = pgml.newTransformerPipeline("text-generation");
  const it = await t.transform_stream("AI is going to", {max_new_tokens: 5});
  let result = await it.next();
  let output = [];
  while (!result.done) {
    output.push(result.value);
    result = await it.next();
  }
  expect(output.length).toBeGreaterThan(0);
});

///////////////////////////////////////////////////
// Test OpenSourceAI //////////////////////////////
///////////////////////////////////////////////////

it("can open source ai create", () => {
  const client = pgml.newOpenSourceAI();
  const results = client.chat_completions_create(
        "HuggingFaceH4/zephyr-7b-beta",
        [
            {
                role: "system",
                content: "You are a friendly chatbot who always responds in the style of a pirate",
            },
            {
                role: "user",
                content: "How many helicopters can a human eat in one sitting?",
            },
        ],
  );
  expect(results.choices.length).toBeGreaterThan(0);
});


it("can open source ai create async", async () => {
  const client = pgml.newOpenSourceAI();
  const results = await client.chat_completions_create_async(
        "HuggingFaceH4/zephyr-7b-beta",
        [
            {
                role: "system",
                content: "You are a friendly chatbot who always responds in the style of a pirate",
            },
            {
                role: "user",
                content: "How many helicopters can a human eat in one sitting?",
            },
        ],
  );
  expect(results.choices.length).toBeGreaterThan(0);
});


it("can open source ai create stream", () => {
  const client = pgml.newOpenSourceAI();
  const it = client.chat_completions_create_stream(
        "HuggingFaceH4/zephyr-7b-beta",
        [
            {
                role: "system",
                content: "You are a friendly chatbot who always responds in the style of a pirate",
            },
            {
                role: "user",
                content: "How many helicopters can a human eat in one sitting?",
            },
        ],
  );
  let result = it.next();
  while (!result.done) {
    expect(result.value.choices.length).toBeGreaterThan(0);
    result = it.next();
  }
});

it("can open source ai create stream async", async () => {
  const client = pgml.newOpenSourceAI();
  const it = await client.chat_completions_create_stream_async(
        "HuggingFaceH4/zephyr-7b-beta",
        [
            {
                role: "system",
                content: "You are a friendly chatbot who always responds in the style of a pirate",
            },
            {
                role: "user",
                content: "How many helicopters can a human eat in one sitting?",
            },
        ],
  );
  let result = await it.next();
  while (!result.done) {
    expect(result.value.choices.length).toBeGreaterThan(0);
    result = await it.next();
  }
});

///////////////////////////////////////////////////
// Test migrations ////////////////////////////////
///////////////////////////////////////////////////

it("can migrate", async () => {
  await pgml.migrate();
});
