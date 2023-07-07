In this tutorial, we will show how to build a text processing pipeline to chunk documents in a Postgres DB using langchain text splitters and generate embeddings using open-source Hugging Face models. We will use  `PostgresML` to chunk documents and compute embeddigns and `dbt` (data build tools) to orchestrate the pipeline (figure below).

![dbt-flow](./images/dbt-pgml.png)

# Prerequisites

- [PostgresML DB](https://github.com/postgresml/postgresml#installation)
- `Python >=3.7.2,<4.0`
- [Poetry](https://python-poetry.org/)
- Install `dbt` using the following commands
  - `poetry shell`
  - `poetry install`
- Documents in a table

# dbt Project Setup
Once you have the pre-requisites satisfied, update `dbt` project configuration files with the right properties to execute the pipeline. 

## Project name
You can find the name of the `dbt` project in `dbt_project.yml`.

```yaml
# Name your project! Project names should contain only lowercase characters
# and underscores. A good package name should reflect your organization's
# name or the intended use of these models
name: 'pgml_flow'
version: '1.0.0'
```

## Dev and prod DBs
Update `profiles.yml` file with development and production database properties. If you are using Docker based local PostgresML installation, `profiles.yml` will be as follows:

```yaml
pgml_flow:
  outputs:

    dev:
      type: postgres
      threads: 1
      host: 127.0.0.1
      port: 5433
      user: postgres
      pass: ""
      dbname: pgml_development
      schema: <schema_name>
    
    prod:
      type: postgres
      threads: [1 or more]
      host: [host]
      port: [port]
      user: [prod_username]
      pass: [prod_password]
      dbname: [dbname]
      schema: [prod_schema]

  target: dev  
```

Run `dbt debug` at the command line where the project's Python environemnt is activated to make sure the DB credentials are correct.

## Source
Update `models/schema.yml` with schema and table where documents are ingested.

```yaml
  sources:
  - name: <schema>
    tables:
      - name: <documents table>
```

# Variables

```yaml
vars:
  splitter_name: "recursive_character"
  splitter_parameters: {"chunk_size": 100, "chunk_overlap": 20}
  task: "embedding"
  model_name: "intfloat/e5-base"
  embeddings_table_name: "embeddings_intfloat_e5_small"
  query_string: 'Lorem ipsum 3'
  limit: 2
```
# Models

## Splitters

## Chunks

## Models

## Transforms

## Embeddings

## Transforms

# Pipeline execution

# Conclusions