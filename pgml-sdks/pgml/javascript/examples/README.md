# Examples

## Prerequisites
Before running any examples first install dependencies and set the DATABASE_URL environment variable:
```
npm i
export DATABASE_URL={YOUR DATABASE URL}
```

Optionally, configure a .env file containing a DATABASE_URL variable.

## [Semantic Search](./semantic_search.js)
This is a basic example to perform semantic search on a collection of documents. Embeddings are created using `intfloat/e5-small-v2` model. The results are semantically similar documemts to the query. Finally, the collection is archived.

## [Question Answering](./question_answering.js)
This is an example to find documents relevant to a question from the collection of documents. The query is passed to vector search to retrieve documents that match closely in the embeddings space. A score is returned with each of the search result.

## [Question Answering using Instructore Model](./question_answering_instructor.js)
In this example, we will use `hknlp/instructor-base` model to build text embeddings instead of the default `intfloat/e5-small-v2` model.

## [Extractive Question Answering](./extractive_question_answering.js)
In this example, we will show how to use `vector_recall` result as a `context` to a HuggingFace question answering model. We will use `Builtins.transform()` to run the model on the database.

## [Summarizing Question Answering](./summarizing_question_answering.js)
This is an example to find documents relevant to a question from the collection of documents and then summarize those documents.

## [Webpack](./webpack)
This is an example of how to use webpack with the SDK
