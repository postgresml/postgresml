export const generateSql = (task, model, userInput) => {
  let input = generateInput(task, model, "sql");
  let args = generateModelArgs(task, model, "sql");
  let extraTaskArgs = generateTaskArgs(task, model, "sql");

  if (!userInput && task == "embedded-query") {
    userInput ="What is Postgres?"
  }

  let argsOutput = "";
  if (args) {
    argsOutput = `,
  args   => ${args}`;
  }

  if (task == "text-generation") {
    return `SELECT pgml.transform_stream(
  task   => '{
    "task": "${task}",
    "model": "${model}"${extraTaskArgs}
  }'::JSONB,
  input  => ${input}${argsOutput}
);`
  } else if (task === "embeddings") {
    return `SELECT pgml.embed(
    '${model}',
    'AI is changing the world as we know it.'
);`;
  } else if (task === "embedded-query") {
    return `WITH embedded_query AS (
  SELECT pgml.embed('mixedbread-ai/mxbai-embed-large-v1', 'What is Postgres?', '{"prompt": "Represent this sentence for searching relevant passages: "}'::JSONB)::vector embedding
),
context_query AS (
  SELECT chunks.chunk FROM chunks
  INNER JOIN embeddings ON embeddings.chunk_id = chunks.id
  ORDER BY embeddings.embedding <=> (SELECT embedding FROM embedded_query)
  LIMIT 1
)
SELECT
  pgml.transform(
    task => '{
      "task": "conversational",
      "model": "meta-llama/Meta-LLama-3.1-8B-Instruct"
    }'::jsonb,
    inputs => ARRAY['{"role": "system", "content": "You are a friendly and helpful chatbot."}'::jsonb, jsonb_build_object('role', 'user', 'content', replace('Given the context answer the following question. ${userInput}? Context:\n{CONTEXT}', '{CONTEXT}', chunk))],
    args => '{
      "max_new_tokens": 100
    }'::jsonb
  )
FROM context_query;`
  } else if (task === "create-table") {
    return `CREATE TABLE IF NOT EXISTS 
documents_embeddings_table (
  document text,
  embedding vector(384));`
  } else {
        let inputs = "    ";
    if (Array.isArray(input))
      inputs += input.map(v => `'${v}'`).join(",\n    ");
    else
      inputs += input;

    return `SELECT pgml.transform(
  task   => '{
    "task": "${task}",
    "model": "${model}"${extraTaskArgs}
  }'::JSONB,
  inputs => ARRAY[
${inputs}
  ]${argsOutput}
);`;
  
  }
};

export const generatePython = (task, model) => {
  let input = generateInput(task, model, "python");
  let modelArgs = generateModelArgs(task, model, "python");
  let taskArgs = generateTaskArgs(task, model, "python");

  let argsOutput = "";
  if (modelArgs) {
    argsOutput = `, ${modelArgs}`;
  }

  if (task == "text-generation") {
    return `from pgml import TransformerPipeline
pipe = TransformerPipeline("${task}", "${model}", ${taskArgs}, "postgres://pg:ml@sql.cloud.postgresml.org:6432/pgml")
async for t in await pipe.transform_stream(${input}${argsOutput}):
  print(t)`;
  } else if (task === "embeddings") {
    return `from pgml import Builtins
connection = Builtins("postgres://pg:ml@sql.cloud.postgresml.org:6432/pgml")
await connection.embed('${model}', 'AI is changing the world as we know it.')`
  } else {
    let inputs;
    if (Array.isArray(input))
      inputs = input.map(v => `"${v}"`).join(", ");
    else
      inputs = input;
    return `from pgml import TransformerPipeline
pipe = TransformerPipeline("${task}", "${model}", ${taskArgs}"postgres://pg:ml@sql.cloud.postgresml.org:6432/pgml")
await pipe.transform([${inputs}]${argsOutput})`;
  }
}

export const generateJavaScript = (task, model) => {
  let input = generateInput(task, model, "javascript");
  let modelArgs = generateModelArgs(task, model, "javascript");
  let taskArgs = generateTaskArgs(task, model, "javascript");
  let argsOutput = "{}";
  if (modelArgs)
    argsOutput = modelArgs;
  
  if (task == "text-generation") {
    return `const pgml = require("pgml");
const pipe = pgml.newTransformerPipeline("${task}", "${model}", ${taskArgs}"postgres://pg:ml@sql.cloud.postgresml.org:6432/pgml");
const it = await pipe.transform_stream(${input}, ${argsOutput});
let result = await it.next();
while (!result.done) {
  console.log(result.value);
  result = await it.next();
}`;
  } else if (task === "embeddings") {
    return `const pgml = require("pgml");
const connection = pgml.newBuiltins("postgres://pg:ml@sql.cloud.postgresml.org:6432/pgml");
let embedding = await connection.embed('${model}', 'AI is changing the world as we know it!');
`
  } else {
    let inputs;
    if (Array.isArray(input))
      inputs = input.map(v => `"${v}"`).join(", ");
    else
      inputs = input;
    return `const pgml = require("pgml");
const pipe = pgml.newTransformerPipeline("${task}", "${model}", ${taskArgs}"postgres://pg:ml@sql.cloud.postgresml.org:6432/pgml");
await pipe.transform([${inputs}], ${argsOutput});`;
  }
}

const generateTaskArgs = (task, model, language) => {
  if (model == "bert-base-uncased") {
    if (language == "sql")
      return `,
    "trust_remote_code": true`;
    else if (language == "python")
      return `{"trust_remote_code": True}, `
    else if (language == "javascript")
      return `{trust_remote_code: true}, `
  } else if (model == "lmsys/fastchat-t5-3b-v1.0" || model == "SamLowe/roberta-base-go_emotions") {
    if (language == "sql")
      return `,
    "device_map": "auto"`;
    else if (language == "python")
      return `{"device_map": "auto"}, `
    else if (language == "javascript")
      return `{device_map: "auto"}, `
  }

  if (task == "summarization") {
    if (language == "sql")
      return ``
    else if (language == "python")
      return `{}, `
    else if (language == "javascript")
      return `{}, `
  }

  if (task == "text-generation") {
    if (language == "sql") {
      return ``
    } else if (language == "python")
      return `{}`
    else if (language == "javascript")
      return `{}, `
  }

  if (language == "python" || language == "javascript")
    return "{}, "

  return ""
}

const generateModelArgs = (task, model, language) => {
  switch (model) {
    case "sileod/deberta-v3-base-tasksource-nli":
    case "facebook/bart-large-mnli":
      if (language == "sql") {
        return `'{
    "candidate_labels": ["amazing", "not amazing"]
  }'::JSONB`;
      } else if (language == "python") {
        return `{"candidate_labels": ["amazing", "not amazing"]}`;
      } else if (language == "javascript") {
        return `{candidate_labels: ["amazing", "not amazing"]}`;
      }
    case "mDeBERTa-v3-base-mnli-xnli":
      if (language == "sql") {
        return `'{
    "candidate_labels": ["politics", "economy", "entertainment", "environment"]
  }'::JSONB`;
      } else if (language == "python") {
        return `{"candidate_labels": ["politics", "economy", "entertainment", "environment"]}`;
      } else if (language == "javascript") {
        return `{candidate_labels: ["politics", "economy", "entertainment", "environment"]}`;
      }
  }

  if (task == "text-generation") {
      if (language == "sql") {
        return `'{
    "max_new_tokens": 100
  }'::JSONB`;
      } else if (language == "python") {
        return `{"max_new_tokens": 100}`;
      } else if (language == "javascript") {
        return `{max_new_tokens: 100}`;
    }
  }

  if (language == "python" || language == "javascript")
    return "{}"

  return "";
};

export const generateModels = (task) => {
  switch (task) {
  case "embeddings":
    return [
      "intfloat/e5-small-v2",
      "Alibaba-NLP/gte-large-en-v1.5",
      "mixedbread-ai/mxbai-embed-large-v1",
    ];
    case "text-classification":
      return [
        "distilbert-base-uncased-finetuned-sst-2-english",
        "SamLowe/roberta-base-go_emotions",
        "ProsusAI/finbert",
      ];
    case "token-classification":
      return [
        "dslim/bert-base-NER",
        "vblagoje/bert-english-uncased-finetuned-pos",
        "d4data/biomedical-ner-all",
      ];
    case "translation":
      return ["google-t5/t5-base"];
    case "summarization":
      return [
        "google/pegasus-xsum",
      ];
    case "question-answering":
      return [
        "deepset/roberta-base-squad2",
        "distilbert-base-cased-distilled-squad",
        "distilbert-base-uncased-distilled-squad",
      ];
    case "text-generation":
      return [
        "meta-llama/Meta-LLama-3.1-8B-Instruct",
        "meta-llama/Meta-LLama-3.1-70B-Instruct",
        "mistralai/Mixtral-8x7B-Instruct-v0.1",
        "mistralai/Mistral-7B-Instruct-v0.2",
      ];
    case "text2text-generation":
      return [
        "google/flan-t5-base",
        "lmsys/fastchat-t5-3b-v1.0",
        "grammarly/coedit-large",
      ];
    case "fill-mask":
      return ["bert-base-uncased", "distilbert-base-uncased", "roberta-base"];
    case "zero-shot-classification":
      return [
        "facebook/bart-large-mnli",
        "sileod/deberta-v3-base-tasksource-nli",
      ];
    case "embedded-query":
      return [
        "many"
      ]
    case "create-table":
      return [
        "none"
      ]
  }
};

const generateInput = (task, model, language) => {
  let sd;
  if (language == "sql")
    sd = "'"
  else
    sd = '"'
  
  if (task == "text-classification") {
    if (model == "ProsusAI/finbert")
      return ["Stocks rallied and the British pound gained", "Stocks fell and the British pound lost"];
    return ["I love how amazingly simple ML has become!", "I hate doing mundane and thankless tasks."];

  } else if (task == "zero-shot-classification") {
    return `${sd}PostgresML is an absolute game changer!${sd}`;

  } else if (task == "token-classification") {
    if (model == "d4data/biomedical-ner-all")
      return `${sd}CASE: A 28-year-old previously healthy man presented with a 6-week history of palpitations. The symptoms occurred during rest, 2–3 times per week, lasted up to 30 minutes at a time and were associated with dyspnea. Except for a grade 2/6 holosystolic tricuspid regurgitation murmur (best heard at the left sternal border with inspiratory accentuation), physical examination yielded unremarkable findings.${sd}`;
    return `${sd}PostgresML - the future of machine learning${sd}`;

  } else if (task == "summarization") {
    return `${sd}PostgresML is the future of GPU accelerated machine learning! It is the best tool for doing machine learning in the database.${sd}`;

  } else if (task == "translation") {
    return `${sd}translate English to French: You know Postgres. Now you know machine learning.${sd}`;

  } else if (task == "question-answering") {
    if (language == "sql") {
      return `'{
        "question": "Is PostgresML the best?",
        "context": "PostgresML is the best!"
    }'`;
    } else if (language == "python") {
      return `'{"question": "Is PostgresML the best?", "context": "PostgresML is the best!"}'`
    } else if (language == "javascript") {
      return `'{"question": "Is PostgresML the best?", "context": "PostgresML is the best!"}'`
    } 

  } else if (task == "text2text-generation") {
    if (model == "grammarly/coedit-large")
      return `${sd}Make this text coherent: PostgresML is the best. It provides super fast machine learning in the database.${sd}`;
    return `${sd}translate from English to French: Welcome to the future!${sd}`;

  } else if (task == "fill-mask") {
    if (model == "roberta-base") {
      return `${sd}Paris is the <mask> of France.${sd}`;
    }
    return `${sd}Paris is the [MASK] of France.${sd}`;
  }

  else if (task == "text-generation") {
    return `${sd}AI is going to${sd}`;
  }

  else if (task === "embedding-query") {
    return `A complete RAG pipeline in a single line of SQL. It does embedding, retrieval and text generation all-in-one SQL query.`;
  }

  return `${sd}AI is going to${sd}`;
};

export const generateOutput = (task) => {
  switch (task) {
    case "create-table":
      return `Table "public.document_embeddings_table"  
   Column  |    Type     | Collation | Nullable | Default 
-----------+-------------+-----------+----------+---------
 document  | text        |           |          |          
 embedding | vector(384) |           |          |          `
  }
};
