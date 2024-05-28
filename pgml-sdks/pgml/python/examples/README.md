# Examples

## Prerequisites
Before running any examples first install dependencies and set the DATABASE_URL environment variable:
```
pip install -r requirements.txt
export DATABASE_URL={YOUR DATABASE URL}
```

Optionally, configure a .env file containing a DATABASE_URL variable.

## [Semantic Search](./semantic_search.py)
This is a basic example to perform semantic search on a collection of documents. It loads the Quora dataset, creates a collection in a PostgreSQL database, upserts documents, generates chunks and embeddings, and then performs a vector search on a query. Embeddings are created using `intfloat/e5-small-v2` model. The results are semantically similar documemts to the query. Finally, the collection is archived.

## [Question Answering](./question_answering.py)
This is an example to find documents relevant to a question from the collection of documents. It loads the Stanford Question Answering Dataset (SQuAD) into the database, generates chunks and embeddings. Query is passed to vector search to retrieve documents that match closely in the embeddings space. A score is returned with each of the search result.

## [Question Answering using Instructor Model](./question_answering_instructor.py)
In this example, we will use `hknlp/instructor-base` model to build text embeddings instead of the default `intfloat/e5-small-v2` model.

## [Extractive Question Answering](./extractive_question_answering.py)
In this example, we will show how to use `vector_recall` result as a `context` to a HuggingFace question answering model. We will use `Builtins.transform()` to run the model on the database.

## [Table Question Answering](./table_question_answering.py)
In this example, we will use [Open Table-and-Text Question Answering (OTT-QA)](https://github.com/wenhuchen/OTT-QA) dataset to run queries on tables. We will use `deepset/all-mpnet-base-v2-table` model that is trained for embedding tabular data for retrieval tasks. 

## [Summarizing Question Answering](./summarizing_question_answering.py)
This is an example to find documents relevant to a question from the collection of documents and then summarize those documents.
