const pgml = require("pgml");
require("dotenv").config();


const main = async () => {
  // Initialize the collection
  const collection = pgml.newCollection("my_javascript_eqa_collection_2");

  // Add a pipeline
  const model = pgml.newModel();
  const splitter = pgml.newSplitter();
  const pipeline = pgml.newPipeline(
    "my_javascript_eqa_pipeline_1",
    model,
    splitter,
  );
  await collection.add_pipeline(pipeline);

  // Upsert documents, these documents are automatically split into chunks and embedded by our pipeline
  const documents = [
    {
      id: "Document One",
      text: "PostgresML is the best tool for machine learning applications!",
    },
    {
      id: "Document Two",
      text: "PostgresML is open source and available to everyone!",
    },
  ];
  await collection.upsert_documents(documents);

  const query = "What is the best tool for machine learning?";

  // Perform vector search
  const queryResults = await collection
    .query()
    .vector_recall(query, pipeline)
    .limit(1)
    .fetch_all();

  // Construct context from results
  const context = queryResults
    .map((result) => {
      return result[1];
    })
    .join("\n");

  // Query for answer
  const builtins = pgml.newBuiltins();
  const answer = await builtins.transform("question-answering", [
    JSON.stringify({ question: query, context: context }),
  ]);

  // Archive the collection
  await collection.archive();
  return answer;
};

main().then((results) => {
  console.log("Question answer: \n", results);
});
