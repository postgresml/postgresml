---
author: Santi Adavani
description: Unlock the Power of Large Language Models (LLM) in Data Pipelines with PostgresML and dbt. Streamline your text processing workflows and leverage the advanced capabilities of LLMs for efficient data transformation and analysis. Discover how PostgresML and dbt combine to deliver scalable and secure pipelines, enabling you to extract valuable insights from textual data. Supercharge your data-driven decision-making with LLM-based pipelines using PostgresML and dbt.
image: https://postgresml.org/dashboard/static/images/blog/dbt-pgml.png
image_alt: "LLM based pipelines with PostgresML and dbt (data build tool)"
---
# LLM based pipelines with PostgresML and dbt (data build tool)
<div class="d-flex align-items-center mb-4">
  <img width="54px" height="54px" src="/dashboard/static/images/team/santi.jpg" style="border-radius: 50%;" alt="Author" />
  <div class="ps-3 d-flex justify-content-center flex-column">
    <p class="m-0">Santi Adavani</p>
    <p class="m-0">July 13, 2023</p>
  </div>
</div>

In the realm of data analytics and machine learning, text processing and large language models (LLMs) have become pivotal in deriving insights from textual data. Efficient data pipelines play a crucial role in enabling streamlined workflows for processing and analyzing text. This blog explores the synergy between PostgresML and dbt, showcasing how they empower organizations to build efficient data pipelines that leverage large language models for text processing, unlocking valuable insights and driving data-driven decision-making.

<img src="/dashboard/static/images/blog/dbt-pgml.png" alt="pgml and dbt llm pipeline">

# PostgresML
PostgresML, an open-source machine learning extension for PostgreSQL, is designed to handle text processing tasks using large language models. Its motivation lies in harnessing the power of LLMs within the familiar PostgreSQL ecosystem. By integrating LLMs directly into the database, PostgresML eliminates the need for data movement and offers scalable and secure text processing capabilities. This native integration enhances data governance, security, and ensures the integrity of text data throughout the pipeline.

# dbt (data build tool)
dbt is an open-source command-line tool that streamlines the process of building, testing, and maintaining data infrastructure. Specifically designed for data analysts and engineers, dbt offers a consistent and standardized approach to data transformation and analysis. By providing an intuitive and efficient workflow, dbt simplifies working with data, empowering organizations to seamlessly transform and analyze their data.

# PostgresML and dbt
The integration of PostgresML and dbt offers an exceptional advantage for data engineers seeking to swiftly incorporate text processing into their workflows. With PostgresML's advanced machine learning capabilities and dbt's streamlined data transformation framework, data engineers can seamlessly integrate text processing tasks into their existing pipelines. This powerful combination empowers data engineers to efficiently leverage PostgresML's text processing capabilities, accelerating the incorporation of sophisticated NLP techniques and large language models into their data workflows. By bridging the gap between machine learning and data engineering, PostgresML and dbt enable data engineers to unlock the full potential of text processing with ease and efficiency.

- Streamlined Text Processing: PostgresML seamlessly integrates large language models into the data pipeline, enabling efficient and scalable text processing. It leverages the power of the familiar PostgreSQL environment, ensuring data integrity and simplifying the overall workflow.

- Simplified Data Transformation: dbt simplifies the complexities of data transformation by automating repetitive tasks and providing a modular approach. It seamlessly integrates with PostgresML, enabling easy incorporation of large language models for feature engineering, model training, and text analysis.

- Scalable and Secure Pipelines: PostgresML's integration with PostgreSQL ensures scalability and security, allowing organizations to process and analyze large volumes of text data with confidence. Data governance, access controls, and compliance frameworks are seamlessly extended to the text processing pipeline.

# Tutorial
By following this [technical tutorial](), you will gain hands-on experience in setting up a dbt project, defining models, and executing an LLM-based text processing pipeline. We will guide you through the process of incorporating LLM-based text processing into your data workflows using PostgresML and dbt. Here's a high-level summary of the tutorial:

## Prerequisites:
- Install PostgresML using Docker or sign up for a [free trial](https://postgresml.org/signup).
- Ensure you have dbt installed on your system.


## Setting up the dbt Project:

- Initialize a new dbt project or use an existing one.
- Configure your project settings, including database connection details and data source configurations.

## Defining Models:

- Define models in dbt to represent the desired data transformations like chunking and embeddings. 
- Leverage the power of PostgresML to incorporate langchain based text splitters and open-source HuggingFace models for embeddings.

## Building the Pipeline:

- Define the necessary transformations and pipelines in dbt to orchestrate the LLM-based text processing workflow.
- Specify incremental materialization and unique key configurations for efficient pipeline execution.

## Executing the Pipeline:
- Run the dbt commands to execute the LLM-based text processing pipeline.
- Monitor the command outputs for any errors or issues encountered during pipeline execution.

# Conclusions
With PostgresML and dbt, organizations can leverage the full potential of LLMs, transforming raw textual data into valuable knowledge, and staying at the forefront of data-driven innovation. By seamlessly integrating LLM-based transformations, data engineers can unlock deeper insights, perform advanced analytics, and drive informed decision-making. Data governance, access controls, and compliance frameworks seamlessly extend to the text processing pipeline, ensuring data integrity and security throughout the LLM-based workflow.