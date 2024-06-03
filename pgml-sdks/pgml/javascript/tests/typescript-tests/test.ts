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
      title: `Test Document ${i}`,
      body: `Test body ${i}`,
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
  let pipeline = pgml.newPipeline("test_j_p_ccp");
  expect(pipeline).toBeTruthy();
});

it("can create single field pipeline", () => {
  let model = pgml.newModel();
  let splitter = pgml.newSplitter();
  let pipeline = pgml.newSingleFieldPipeline("test_j_p_ccsfp", model, splitter);
  expect(pipeline).toBeTruthy();
});

it("can create builtins", () => {
  let builtins = pgml.newBuiltins();
  expect(builtins).toBeTruthy();
});

///////////////////////////////////////////////////
// Test various searches ///////////////////
///////////////////////////////////////////////////

it("can search", async () => {
  let pipeline = pgml.newPipeline("test_j_p_cs", {
    title: { semantic_search: { model: "intfloat/e5-small-v2", parameters: { prompt: "passage: " } } },
    body: {
      splitter: { model: "recursive_character" },
      semantic_search: {
        model: "text-embedding-ada-002",
        source: "openai",
      },
      full_text_search: { configuration: "english" },
    },
  });
  let collection = pgml.newCollection("test_j_c_tsc_15")
  await collection.add_pipeline(pipeline)
  await collection.upsert_documents(generate_dummy_documents(5))
  let results = await collection.search(
    {
      query: {
        full_text_search: { body: { query: "Test", boost: 1.2 } },
        semantic_search: {
          title: {
            query: "This is a test", parameters: { prompt: "query: " }, boost: 2.0
          },
          body: { query: "This is the body test", boost: 1.01 },
        },
        filter: { id: { $gt: 1 } },
      },
      limit: 10
    },
    pipeline,
  );
  let ids = results["results"].map((r: any) => r["id"]);
  expect(ids).toEqual([4, 3, 5]);
  await collection.archive();
});

///////////////////////////////////////////////////
// Test various vector searches ///////////////////
///////////////////////////////////////////////////

it("can vector search", async () => {
  let pipeline = pgml.newPipeline("1", {
    title: {
      semantic_search: { model: "intfloat/e5-small-v2", parameters: { prompt: "passage: " } },
      full_text_search: { configuration: "english" },
    },
    body: {
      splitter: { model: "recursive_character" },
      semantic_search: {
        model: "text-embedding-ada-002",
        source: "openai",
      },
    },
  });
  let collection = pgml.newCollection("test_j_c_cvs_4")
  await collection.add_pipeline(pipeline)
  await collection.upsert_documents(generate_dummy_documents(5))
  let results = await collection.vector_search(
    {
      query: {
        fields: {
          title: { query: "Test document: 2", parameters: { prompt: "query: " }, full_text_filter: "test" },
          body: { query: "Test document: 2" },
        },
        filter: { id: { "$gt": 2 } },
      },
      limit: 5,
    },
    pipeline,
  );
  let ids = results.map(r => r["document"]["id"]);
  expect(ids).toEqual([4, 3, 3, 4]);
  await collection.archive();
});

it("can vector search with query builder", async () => {
  let model = pgml.newModel("intfloat/e5-small-v2", "pgml", { prompt: "passage: " });
  let splitter = pgml.newSplitter();
  let pipeline = pgml.newSingleFieldPipeline("0", model, splitter);
  let collection = pgml.newCollection("test_j_c_cvswqb_2");
  await collection.upsert_documents(generate_dummy_documents(3));
  await collection.add_pipeline(pipeline);
  let results = await collection
    .query()
    .vector_recall("Here is some query", pipeline)
    .limit(10)
    .fetch_all();
  let ids = results.map(r => r[2]["id"]);
  expect(ids).toEqual([1, 2, 0]);
  await collection.archive();
});

///////////////////////////////////////////////////
// Test rag ///////////////////////////////////////
///////////////////////////////////////////////////

it("can rag", async () => {
  let pipeline = pgml.newPipeline("0", {
    body: {
      splitter: { model: "recursive_character" },
      semantic_search: {
        model: "intfloat/e5-small-v2",
        parameters: { prompt: "passage: " },
      },
    },
  });
  let collection = pgml.newCollection("test_j_c_cr_0")
  await collection.add_pipeline(pipeline)
  await collection.upsert_documents(generate_dummy_documents(5))
  const results = await collection.rag(
    {
      "CONTEXT": {
        vector_search: {
          query: {
            fields: {
              body: { query: "Test document: 2", parameters: { prompt: "query: " } },
            },
          },
          document: { keys: ["id"] },
          limit: 5,
        },
        aggregate: { join: "\n" },
      },
      completion: {
        model: "meta-llama/Meta-Llama-3-8B-Instruct",
        prompt: "Some text with {CONTEXT}",
        max_tokens: 10,
      },
    },
    pipeline
  );
  expect(results["rag"][0].length).toBeGreaterThan(0);
  expect(results["sources"]["CONTEXT"].length).toBeGreaterThan(0);
  await collection.archive()
})


it("can rag stream", async () => {
  let pipeline = pgml.newPipeline("0", {
    body: {
      splitter: { model: "recursive_character" },
      semantic_search: {
        model: "intfloat/e5-small-v2",
        parameters: { prompt: "passage: " },
      },
    },
  });
  let collection = pgml.newCollection("test_j_c_cr_0")
  await collection.add_pipeline(pipeline)
  await collection.upsert_documents(generate_dummy_documents(5))
  const results = await collection.rag_stream(
    {
      "CONTEXT": {
        vector_search: {
          query: {
            fields: {
              body: { query: "Test document: 2", parameters: { prompt: "query: " } },
            },
          },
          document: { keys: ["id"] },
          limit: 5,
        },
        aggregate: { join: "\n" },
      },
      completion: {
        model: "meta-llama/Meta-Llama-3-8B-Instruct",
        prompt: "Some text with {CONTEXT}",
        max_tokens: 10,
      },
    },
    pipeline
  );
  let output = [];
  let it = results.stream();
  let result = await it.next();
  while (!result.done) {
    output.push(result.value);
    result = await it.next();
  }
  expect(output.length).toBeGreaterThan(0);
  await collection.archive()
})

///////////////////////////////////////////////////
// Test document related functions ////////////////
///////////////////////////////////////////////////

it("can upsert and get documents", async () => {
  let collection = pgml.newCollection("test_p_c_cuagd_1");
  await collection.upsert_documents(generate_dummy_documents(10));
  let documents = await collection.get_documents();
  expect(documents).toHaveLength(10);
  documents = await collection.get_documents({
    offset: 1,
    limit: 2,
    filter: { id: { $gt: 0 } },
  });
  expect(documents).toHaveLength(2);
  expect(documents[0]["document"]["id"]).toBe(2);
  let last_row_id = documents[1]["row_id"];
  documents = await collection.get_documents({
    filter: {
      id: { $lt: 7 },
    },
    last_row_id: last_row_id,
  });
  expect(documents).toHaveLength(3);
  expect(documents[0]["document"]["id"]).toBe(4);
  await collection.archive();
});

it("can delete documents", async () => {
  let collection = pgml.newCollection("test_p_c_cdd_2");
  await collection.upsert_documents(generate_dummy_documents(3));
  await collection.delete_documents({
    id: { $gte: 2 },
  });
  let documents = await collection.get_documents();
  expect(documents).toHaveLength(2);
  expect(documents[0]["document"]["id"]).toBe(0);

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
  const t = pgml.newTransformerPipeline("text-generation", "meta-llama/Meta-Llama-3-8B-Instruct");
  const it = await t.transform(["AI is going to"], { max_tokens: 5 });
  expect(it.length).toBeGreaterThan(0)
});

it("can transformer pipeline stream", async () => {
  const t = pgml.newTransformerPipeline("text-generation", "meta-llama/Meta-Llama-3-8B-Instruct");
  const it = await t.transform_stream("AI is going to", { max_tokens: 5 });
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
    "meta-llama/Meta-Llama-3-8B-Instruct",
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
    10
  );
  expect(results.choices.length).toBeGreaterThan(0);
});


it("can open source ai create async", async () => {
  const client = pgml.newOpenSourceAI();
  const results = await client.chat_completions_create_async(
    "meta-llama/Meta-Llama-3-8B-Instruct",
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
    10
  );
  expect(results.choices.length).toBeGreaterThan(0);
});


it("can open source ai create stream", () => {
  const client = pgml.newOpenSourceAI();
  const it = client.chat_completions_create_stream(
    "meta-llama/Meta-Llama-3-8B-Instruct",
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
    10
  );
  let result = it.next();
  while (!result.done) {
    expect(result.value.choices.length).toBeGreaterThanOrEqual(0);
    result = it.next();
  }
});

it("can open source ai create stream async", async () => {
  const client = pgml.newOpenSourceAI();
  const it = await client.chat_completions_create_stream_async(
    "meta-llama/Meta-Llama-3-8B-Instruct",
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
    10
  );
  let result = await it.next();
  while (!result.done) {
    expect(result.value.choices.length).toBeGreaterThanOrEqual(0);
    result = await it.next();
  }
});

///////////////////////////////////////////////////
// Test migrations ////////////////////////////////
///////////////////////////////////////////////////

it("can migrate", async () => {
  await pgml.migrate();
});
