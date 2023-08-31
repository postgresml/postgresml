require('dotenv').config()
const get_classification = require("./get_classification");

const main = async () => {
  let text = process.argv[2];
  if (!text) {
    console.log("Please provide a text to classify. E.G: node dist/index.js \"This is a test\")");
  } else {
    console.log("Classifying text:", text, "(may take a few seconds...)")
    const classified_text = await get_classification(text);
    console.log("Classification results:");
    console.log(classified_text)
  }
};

main()
  .then(() => console.log("Webpack demo done!"))
  .catch((e) => console.error(e));
