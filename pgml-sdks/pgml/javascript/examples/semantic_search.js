const pgml = require("pgml");
require("dotenv").config();

const main = async () => {
  // Initialize the collection
  const collection = pgml.newCollection("semantic_search_collection");

  // Add a pipeline
  const pipeline = pgml.newPipeline("semantic_search_pipeline", {
    text: {
      splitter: { model: "recursive_character" },
      semantic_search: {
        model: "Alibaba-NLP/gte-base-en-v1.5",
      },
    },
  });
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
  const query = "Something that will match document one first";
  const queryResults = await collection.vector_search(
    {
      query: {
        fields: {
          text: { query: query }
        }
      }, limit: 2
    }, pipeline);
  console.log("The results");
  console.log(queryResults);

  // Archive the collection
  await collection.archive();
};

main().then(() => console.log("Done!"));
