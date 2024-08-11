---
description: >-
  Perform dozens of state-of-the-art natural language processing (NLP) tasks
  with thousands of models. Serve with the same Postgres infrastructure.
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

The `pgml.transform()` function is the most powerful feature of PostgresML. It integrates open-source large language models, like Llama, Mixtral, and many more, which allows to perform complex tasks on your data.

The models are downloaded from [🤗 Hugging Face](https://huggingface.co/transformers) which hosts tens of thousands of pre-trained and fine-tuned models for various tasks like text generation, question answering, summarization, text classification, and more.

## API

The `pgml.transform()` function comes in two flavors, task-based and model-based.

### Task-based API

The task-based API automatically chooses a model based on the task:

```postgresql
pgml.transform(
    task TEXT,
    args JSONB,
    inputs TEXT[]
)
```

| Argument | Description | Example | Required |
|----------|-------------|---------|----------|
| task | The name of a natural language processing task. | `'text-generation'` | Required |
| args | Additional kwargs to pass to the pipeline. | `'{"max_new_tokens": 50}'::JSONB` | Optional |
| inputs | Array of prompts to pass to the model for inference. Each prompt is evaluated independently and a separate result is returned. | `ARRAY['Once upon a time...']` | Required |

#### Examples

{% tabs %}
{% tabs %}
{% tab title="Text generation" %}

```postgresql
SELECT *
FROM pgml.transform(
  task => 'text-generation',
  inputs => ARRAY['In a galaxy far far away']
);
```

{% endtab %}
{% tab title="Translation" %}

```postgresql
SELECT *
FROM pgml.transform(
  task => 'translation_en_to_fr',
  inputs => ARRAY['How do I say hello in French?']
);
```

{% endtab %}
{% endtabs %}

### Model-based API

The model-based API requires the name of the model and the task, passed as a JSON object. This allows it to be more generic and support more models:

```postgresql
pgml.transform(
    model JSONB,
    args JSONB,
    inputs TEXT[]
)
```

<table class="table-sm table">
  <thead>
    <th>Argument</th>
    <th>Description</th>
    <th>Example</th>
  </thead>
  <tbody>
    <tr>
      <td>model</td>
      <td>Model configuration, including name and task.</td>
      <td>
        <div class="code-multi-line font-monospace">
          '{
            <br>&nbsp;&nbsp;"task": "text-generation",
            <br>&nbsp;&nbsp;"model": "mistralai/Mixtral-8x7B-v0.1"
          <br>}'::JSONB
        </div>
      </td>
    </tr>
    <tr>
      <td>args</td>
      <td>Additional kwargs to pass to the pipeline.</td>
      <td><code>'{"max_new_tokens": 50}'::JSONB</code></td>
    </tr>
    <tr>
      <td>inputs</td>
      <td>Array of prompts to pass to the model for inference. Each prompt is evaluated independently.</td>
      <td><code>ARRAY['Once upon a time...']</code></td>
    </tr>
</table>

#### Example

{% tabs %}
{% tab title="PostgresML SQL" %}

```postgresql
SELECT pgml.transform(
  task   => '{
    "task": "text-generation",
    "model": "meta-llama/Meta-Llama-3.1-8B-Instruct",
    "model_type": "mistral",
    "revision": "main",
    "device_map": "auto"
  }'::JSONB,
  inputs  => ARRAY['AI is going to'],
  args   => '{
    "max_new_tokens": 100
  }'::JSONB
);
```

{% endtab %}

{% tab title="Equivalent Python" %}

```python
import transformers

def transform(task, call, inputs):
    return transformers.pipeline(**task)(inputs, **call)

transform(
    {
        "task": "text-generation",
        "model": "meta-llama/Meta-Llama-3.1-8B-Instruct",
        "model_type": "mistral",
        "revision": "main",
    },
    {"max_new_tokens": 100},
    ['AI is going to change the world in the following ways:']
)
```

{% endtab %}
{% endtabs %}

## Guides

See also: [LLM guides](../guides/llms/) for more examples
