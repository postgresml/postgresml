const pgml = require("pgml");
require("dotenv").config();

const CONNECTION_STRING =
  process.env.PGML_CONNECTION ||
  "postgres://postgres@127.0.0.1:5433/pgml_development";

const main = async () => {
  const db = await pgml.newDatabase(CONNECTION_STRING);
  const collection_name = "hello_world";
  const collection = await db.create_or_get_collection(collection_name);
  const documents = [
    {
      name: "Document One",
      text: "document one contents...",
    },
    {
      name: "Document Two",
      text: "document two contents...",
    },
  ];
  await collection.upsert_documents(documents);
  await collection.generate_chunks();
  await collection.generate_embeddings();
  const queryResults = await collection.vector_search(
    "What are the contents of document one?", // query text
    {}, // embedding model parameters
    1 // top_k
  );

  // convert the results to array of objects
  const results = queryResults.map((result) => {
    const [similarity, text, metadata] = result;
    return {
      similarity,
      text,
      metadata,
    };
  });

  await db.archive_collection(collection_name);
  return results;
};

main().then((results) => {
  console.log("Vector search Results: ", results);
});
