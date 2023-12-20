---
layout:
  title:
    visible: true
  description:
    visible: true
  tableOfContents:
    visible: true
  outline:
    visible: true
  pagination:
    visible: true
---

# pgml.transform()

PostgresML integrates [🤗 Hugging Face Transformers](https://huggingface.co/transformers) to bring state-of-the-art models into the data layer. There are tens of thousands of pre-trained models with pipelines to turn raw inputs into useful results. Many state of the art deep learning architectures have been published and made available for download. You will want to browse all the [models](https://huggingface.co/models) available to find the perfect solution for your [dataset](https://huggingface.co/dataset) and [task](https://huggingface.co/tasks).

We'll demonstrate some of the tasks that are immediately available to users of your database upon installation: [translation](https://github.com/postgresml/postgresml/blob/v2.7.12/pgml-dashboard/content/docs/guides/transformers/pre\_trained\_models.md#translation), [sentiment analysis](https://github.com/postgresml/postgresml/blob/v2.7.12/pgml-dashboard/content/docs/guides/transformers/pre\_trained\_models.md#sentiment-analysis), [summarization](https://github.com/postgresml/postgresml/blob/v2.7.12/pgml-dashboard/content/docs/guides/transformers/pre\_trained\_models.md#summarization), [question answering](https://github.com/postgresml/postgresml/blob/v2.7.12/pgml-dashboard/content/docs/guides/transformers/pre\_trained\_models.md#question-answering) and [text generation](https://github.com/postgresml/postgresml/blob/v2.7.12/pgml-dashboard/content/docs/guides/transformers/pre\_trained\_models.md#text-generation).

### Examples

All of the tasks and models demonstrated here can be customized by passing additional arguments to the `Pipeline` initializer or call. You'll find additional links to documentation in the examples below.

The Hugging Face [`Pipeline`](https://huggingface.co/docs/transformers/main\_classes/pipelines) API is exposed in Postgres via:

```sql
pgml.transform(
    task TEXT OR JSONB,      -- task name or full pipeline initializer arguments
    call JSONB,              -- additional call arguments alongside the inputs
    inputs TEXT[] OR BYTEA[] -- inputs for inference
)
```

This is roughly equivalent to the following Python:

```python
import transformers

def transform(task, call, inputs):
    return transformers.pipeline(**task)(inputs, **call)
```

Most pipelines operate on `TEXT[]` inputs, but some require binary `BYTEA[]` data like audio classifiers. `inputs` can be `SELECT`ed from tables in the database, or they may be passed in directly with the query. The output of this call is a `JSONB` structure that is task specific. See the [Postgres JSON](https://www.postgresql.org/docs/14/functions-json.html) reference for ways to process this output dynamically.

!!! tip

Models will be downloaded and stored locally on disk after the first call. They are also cached per connection to improve repeated calls in a single session. To free that memory, you'll need to close your connection. You may want to establish dedicated credentials and connection pools via [pgcat](https://github.com/levkk/pgcat) or [pgbouncer](https://www.pgbouncer.org/) for larger models that have billions of parameters. You may also pass `{"cache": false}` in the JSON `call` args to prevent this behavior.

!!!
