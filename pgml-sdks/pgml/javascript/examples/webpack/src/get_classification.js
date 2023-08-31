const pgml = require("pgml");

const get_classification = async (text) => {
  const builtins = pgml.newBuiltins();
  const results = await builtins.transform("text-classification", [text]);
  return results;
};

module.exports = get_classification;
