const pgml = require("pgml");
require("dotenv").config();

const main = async () => {
  // Initialize the collection
  const collection = pgml.newCollection("my_javascript_collection");

  // Add a pipeline
  const model = pgml.newModel();
  const splitter = pgml.newSplitter();
  const pipeline = pgml.newPipeline("my_javascript_pipeline", model, splitter);
  await collection.add_pipeline(pipeline);

  // Upsert documents, these documents are automatically split into chunks and embedded by our pipeline
  const documents = [
    {
      id: "Document One",
      text: "document one contents...",
    },
    {
      id: "Document Two",
      text: "document two contents...",
    },
  ];
  await collection.upsert_documents(documents);

  // Perform vector search
  const queryResults = await collection
    .query()
    .vector_recall(
      "Some user query that will match document one first",
      pipeline,
    )
    .limit(2)
    .fetch_all();

  // Convert the results to an array of objects
  const results = queryResults.map((result) => {
    const [similarity, text, metadata] = result;
    return {
      similarity,
      text,
      metadata,
    };
  });

  // Archive the collection
  await collection.archive();
  return results;
};

main().then((results) => {
  console.log("Vector search Results: \n", results);
});
