---
description: >-
  The pgml extension for PostgreSQL provides Machine Learning and Artificial
  Intelligence APIs with access to algorithms to train your models, or download
  SOTA open source models from HuggingFace.
---

# SQL Extensions

## Open Source Models

PostgresML integrates [🤗 Hugging Face Transformers](https://huggingface.co/transformers) to bring state-of-the-art models into the data layer. There are tens of thousands of pre-trained models with pipelines to turn raw inputs into useful results. Many LLMs have been published and made available for download. You will want to browse all the [models](https://huggingface.co/models) available to find the perfect solution for your [dataset](https://huggingface.co/dataset) and [task](https://huggingface.co/tasks). The pgml extension provides a few APIs for different use cases:

* [pgml.embed.md](pgml.embed.md "mention") returns vector embeddings for nearest neighbor searches and other vector database use cases
* [pgml.generate.md](pgml.generate.md "mention") returns streaming text responses for chatbots
* [pgml.transform](pgml.transform/ "mention") allows you to perform dozens of natural language processing (NLP) tasks with thousands of models, like sentiment analysis, question and answering, translation, summarization and text generation
* [pgml.tune.md](pgml.tune.md "mention") fine tunes an open source model on your own data

## Train & deploy your own models

PostgresML also supports more than 50 machine learning algorithms to train your own models for classification, regression or clustering. We organize a family of Models in Projects that are intended to address a particular opportunity. Different algorithms can be used in the same Project, to test and compare the performance of various approaches, and track progress over time, all within your database.&#x20;

### Train

Training creates a Model based on the data in your database.

```sql
SELECT pgml.train(
  project_name = > 'Sales Forecast',
  task => 'regression',
  relation_name => 'hist_sales',
  y_column_name => 'next_sales',
  algorithm => 'xgboost'
);
```

&#x20;See [pgml.train](pgml.train/ "mention") for more information.

### Deploy

Deploy an active Model for a particular Project, using a deployment strategy to select the best model.

```sql
SELECT pgml.deploy(
  project_name => 'Sales Forecast',
  strategy => 'best_score',
  algorithm => 'xgboost'
);
```

See [pgml.deploy.md](pgml.deploy.md "mention") for more information.

### Predict

Use your Model on novel data points not seen during training to infer a new data point.&#x20;

```sql
SELECT pgml.predict(
  project_name => 'Sales Forecast',
  features => ARRAY[
    last_week_sales,
    week_of_year
  ]
) AS prediction
FROM new_sales
ORDER BY prediction DESC;
```

See[pgml.predict](pgml.predict/ "mention") for more information.
