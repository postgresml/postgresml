# LLM based pipelines with PostgresML and dbt (data build tool)

In the realm of data analytics and machine learning, text processing and large language models (LLMs) have become pivotal in deriving insights from textual data. Efficient data pipelines play a crucial role in enabling streamlined workflows for processing and analyzing text. This blog explores the synergy between PostgresML and dbt, showcasing how they empower organizations to build efficient data pipelines that leverage large language models for text processing, unlocking valuable insights and driving data-driven decision-making.

## PostgresML

PostgresML, an open-source machine learning extension for PostgreSQL, is designed to handle text processing tasks using large language models. Its motivation lies in harnessing the power of LLMs within the familiar PostgreSQL ecosystem. By integrating LLMs directly into the database, PostgresML eliminates the need for data movement and offers scalable and secure text processing capabilities. This native integration enhances data governance, security, and ensures the integrity of text data throughout the pipeline.

## dbt (data build tool)

dbt is an open-source command-line tool that streamlines the process of building, testing, and maintaining data infrastructure. Specifically designed for data analysts and engineers, dbt offers a consistent and standardized approach to data transformation and analysis. By providing an intuitive and efficient workflow, dbt simplifies working with data, empowering organizations to seamlessly transform and analyze their data.

## PostgresML and dbt

The integration of PostgresML and dbt offers an exceptional advantage for data engineers seeking to swiftly incorporate text processing into their workflows. With PostgresML's advanced machine learning capabilities and dbt's streamlined data transformation framework, data engineers can seamlessly integrate text processing tasks into their existing pipelines. This powerful combination empowers data engineers to efficiently leverage PostgresML's text processing capabilities, accelerating the incorporation of sophisticated NLP techniques and large language models into their data workflows. By bridging the gap between machine learning and data engineering, PostgresML and dbt enable data engineers to unlock the full potential of text processing with ease and efficiency.

* Streamlined Text Processing: PostgresML seamlessly integrates large language models into the data pipeline, enabling efficient and scalable text processing. It leverages the power of the familiar PostgreSQL environment, ensuring data integrity and simplifying the overall workflow.
* Simplified Data Transformation: dbt simplifies the complexities of data transformation by automating repetitive tasks and providing a modular approach. It seamlessly integrates with PostgresML, enabling easy incorporation of large language models for feature engineering, model training, and text analysis.
* Scalable and Secure Pipelines: PostgresML's integration with PostgreSQL ensures scalability and security, allowing organizations to process and analyze large volumes of text data with confidence. Data governance, access controls, and compliance frameworks are seamlessly extended to the text processing pipeline.

## Tutorial

By following this [tutorial](https://github.com/postgresml/postgresml/tree/master/pgml-extension/examples/dbt/embeddings), you will gain hands-on experience in setting up a dbt project, defining models, and executing an LLM-based text processing pipeline. We will guide you through the process of incorporating LLM-based text processing into your data workflows using PostgresML and dbt. Here's a high-level summary of the tutorial:

### Prerequisites

* [PostgresML DB](https://github.com/postgresml/postgresml#installation)
* Python >=3.7.2,<4.0
* [Poetry](https://python-poetry.org/)
* Install `dbt` using the following commands
  * `poetry shell`
  * `poetry install`
* Documents in a table

### dbt Project Setup

Once you have the pre-requisites satisfied, update `dbt` project configuration files.

### Project name

You can find the name of the `dbt` project in `dbt_project.yml`.

```yaml
# Name your project! Project names should contain only lowercase characters
# and underscores. A good package name should reflect your organization's
# name or the intended use of these models
name: 'pgml_flow'
version: '1.0.0'
```

### Dev and prod DBs

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

Run `dbt debug` at the command line where the project's Python environment is activated to make sure the DB credentials are correct.

### Source

Update `models/schema.yml` with schema and table where documents are ingested.

```yaml
  sources:
  - name: <schema>
    tables:
      - name: <documents table>
```

### Variables

The provided YAML configuration includes various parameters that define the setup for a specific task involving embeddings and models.

```yaml
vars:
  splitter_name: "recursive_character"
  splitter_parameters: {"chunk_size": 100, "chunk_overlap": 20}
  task: "embedding"
  model_name: "intfloat/e5-small-v2"
  query_string: 'Lorem ipsum 3'
  limit: 2
```

Here's a summary of the key parameters:

* `splitter_name`: Specifies the name of the splitter, set as "recursive\_character".
* `splitter_parameters`: Defines the parameters for the splitter, such as a chunk size of 100 and a chunk overlap of 20.
* `task`: Indicates the task being performed, specified as "embedding".
* `model_name`: Specifies the name of the model to be used, set as "intfloat/e5-small-v2".
* `query_string`: Provides a query string, set as 'Lorem ipsum 3'.
* `limit`: Specifies a limit of 2, indicating the maximum number of results to be processed.

These configuration parameters offer a specific setup for the task, allowing for customization and flexibility in performing embeddings with the chosen splitter, model, table, query, and result limit.

## Models

dbt models form the backbone of data transformation and analysis pipelines. These models allow you to define the structure and logic for processing your data, enabling you to extract insights and generate valuable outputs.

### Splitters

The Splitters model serves as a central repository for storing information about text splitters and their associated hyperparameters, such as chunk size and chunk overlap. This model allows you to keep track of the different splitters used in your data pipeline and their specific configuration settings.

### Chunks

Chunks build upon splitters and process documents, generating individual chunks. Each chunk represents a smaller segment of the original document, facilitating more granular analysis and transformations. Chunks capture essential information like IDs, content, indices, and creation timestamps.

### Models

Models serve as a repository for storing information about different embeddings models and their associated hyperparameters. This model allows you to keep track of the various embedding techniques used in your data pipeline and their specific configuration settings.

### Embeddings

Embeddings focus on generating feature embeddings from chunks using an embedding model in models table. These embeddings capture the semantic representation of textual data, facilitating more effective machine learning models.

### Transforms

The Transforms maintains a mapping between the splitter ID, model ID, and the corresponding embeddings table for each combination. It serves as a bridge connecting the different components of your data pipeline.

## Pipeline execution

In order to run the pipeline, execute the following command:

```bash
dbt run
```

You should see an output similar to below:

```bash
22:29:58  Running with dbt=1.5.2
22:29:58  Registered adapter: postgres=1.5.2
22:29:58  Unable to do partial parsing because a project config has changed
22:29:59  Found 7 models, 10 tests, 0 snapshots, 0 analyses, 307 macros, 0 operations, 0 seed files, 1 source, 0 exposures, 0 metrics, 0 groups
22:29:59  
22:29:59  Concurrency: 1 threads (target='dev')
22:29:59  
22:29:59  1 of 7 START sql view model test_collection_1.characters ....................... [RUN]
22:29:59  1 of 7 OK created sql view model test_collection_1.characters .................. [CREATE VIEW in 0.11s]
22:29:59  2 of 7 START sql incremental model test_collection_1.models .................... [RUN]
22:29:59  2 of 7 OK created sql incremental model test_collection_1.models ............... [INSERT 0 1 in 0.15s]
22:29:59  3 of 7 START sql incremental model test_collection_1.splitters ................. [RUN]
22:30:00  3 of 7 OK created sql incremental model test_collection_1.splitters ............ [INSERT 0 1 in 0.07s]
22:30:00  4 of 7 START sql incremental model test_collection_1.chunks .................... [RUN]
22:30:00  4 of 7 OK created sql incremental model test_collection_1.chunks ............... [INSERT 0 0 in 0.08s]
22:30:00  5 of 7 START sql incremental model test_collection_1.embedding_36b7e ........... [RUN]
22:30:00  5 of 7 OK created sql incremental model test_collection_1.embedding_36b7e ...... [INSERT 0 0 in 0.08s]
22:30:00  6 of 7 START sql incremental model test_collection_1.transforms ................ [RUN]
22:30:00  6 of 7 OK created sql incremental model test_collection_1.transforms ........... [INSERT 0 1 in 0.07s]
22:30:00  7 of 7 START sql table model test_collection_1.vector_search ................... [RUN]
22:30:05  7 of 7 OK created sql table model test_collection_1.vector_search .............. [SELECT 2 in 4.81s]
22:30:05  
22:30:05  Finished running 1 view model, 5 incremental models, 1 table model in 0 hours 0 minutes and 5.59 seconds (5.59s).
22:30:05  
22:30:05  Completed successfully
22:30:05  
22:30:05  Done. PASS=7 WARN=0 ERROR=0 SKIP=0 TOTAL=7
```

As part of the pipeline execution, some models in the workflow utilize incremental materialization. Incremental materialization is a powerful feature provided by dbt that optimizes the execution of models by only processing and updating the changed or new data since the last run. This approach reduces the processing time and enhances the efficiency of the pipeline.

By configuring certain models with incremental materialization, dbt intelligently determines the changes in the source data and applies only the necessary updates to the target tables. This allows for faster iteration cycles, particularly when working with large datasets, as dbt can efficiently handle incremental updates instead of reprocessing the entire dataset.

## Conclusions

With PostgresML and dbt, organizations can leverage the full potential of LLMs, transforming raw textual data into valuable knowledge, and staying at the forefront of data-driven innovation. By seamlessly integrating LLM-based transformations, data engineers can unlock deeper insights, perform advanced analytics, and drive informed decision-making. Data governance, access controls, and compliance frameworks seamlessly extend to the text processing pipeline, ensuring data integrity and security throughout the LLM-based workflow.
