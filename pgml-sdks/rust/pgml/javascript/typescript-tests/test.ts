import pgml from '../../index.js'

const CONNECTION_STRING = process.env.DATABASE_URL ? process.env.DATABASE_URL : "";
const COLLECTION_NAME = "ttest2";

async function test() {
  let db: pgml.Database = await pgml.newDatabase(CONNECTION_STRING);
  let collection: pgml.Collection = await db.create_or_get_collection(COLLECTION_NAME);
  console.log(collection)
}

test().then(() => console.log("\nTests Done!")).catch((err) => console.log(err));
