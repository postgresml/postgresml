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

The `pgml.transform()` is the most powerful function in PostgresML. It integrates open-source large language models, like Llama, Mixtral, and many more, which allows to perform complex tasks on your data.

The models are downloaded from [🤗 Hugging Face](https://huggingface.co/transformers) which hosts tens of thousands of pre-trained and fine-tuned models for various tasks like text generation, question answering, summarization, text classification, and more.

## API

The `pgml.transform()` function comes in two flavors, task-based and model-based.

### Task-based API

The task-based API automatically chooses a model to use based on the task:

```postgresql
pgml.transform(
    task TEXT,
    args JSONB,
    inputs TEXT[]
)
```

| Argument | Description | Example |
|----------|-------------|---------|
| task | The name of a natural language processing task. | `text-generation` |
| args | Additional kwargs to pass to the pipeline. | `{"max_new_tokens": 50}` |
| inputs | Array of prompts to pass to the model for inference. | `['Once upon a time...']` |

#### Example

{% tabs %}
{% tab title="SQL" %}

```postgresql
SELECT *
FROM pgml.transform (
  'translation_en_to_fr',
  'How do I say hello in French?',
);
```

{% endtab %}
{% endtabs %}

### Model-based API

The model-based API requires the name of the model and the task, passed as a JSON object, which allows it to be more generic:

```postgresql
pgml.transform(
    model JSONB,
    args JSONB,
    inputs TEXT[]
)
```

| Argument | Description | Example |
|----------|-------------|---------|
| task | Model configuration, including name and task. | `{"task": "text-generation", "model": "mistralai/Mixtral-8x7B-v0.1"}` |
| args | Additional kwargs to pass to the pipeline. | `{"max_new_tokens": 50}` |
| inputs | Array of prompts to pass to the model for inference. | `['Once upon a time...']` |

#### Example

{% tabs %}
{% tab title="SQL" %}

```postgresql
SELECT pgml.transform(
  task   => '{
    "task": "text-generation",
    "model": "TheBloke/zephyr-7B-beta-GPTQ",
    "model_type": "mistral",
    "revision": "main",
  }'::JSONB,
  inputs  => ['AI is going to change the world in the following ways:'],
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
        "model": "TheBloke/zephyr-7B-beta-GPTQ",
        "model_type": "mistral",
        "revision": "main",
    },
    {"max_new_tokens": 100},
    ['AI is going to change the world in the following ways:']
)
```

{% endtab %}
{% endtabs %}


### Supported tasks

PostgresML currently supports most NLP tasks available on Hugging Face:

| Task | Name | Description |
|------|-------------|---------|
| [Fill mask](fill-mask) | `key-mask` | Fill in the blank in a sentence. |
| [Question answering](question-answering) | `question-answering` | Answer a question based on a context. |
| [Summarization](summarization) | `summarization` | Summarize a long text. |
| [Text classification](text-classification) | `text-classification` | Classify a text as positive or negative. |
| [Text generation](text-generation) | `text-generation` | Generate text based on a prompt. |
| [Text-to-text generation](text-to-text-generation) | `text-to-text-generation` | Generate text based on an instruction in the prompt. |
| [Token classification](token-classification) | `token-classification` | Classify tokens in a text. |
| [Translation](translation) | `translation` | Translate text from one language to another. |
| [Zero-shot classification](zero-shot-classification) | `zero-shot-classification` | Classify a text without training data. |


## Performance

Much like `pgml.embed()`, the models used in `pgml.transform()` are downloaded from Hugging Face and cached locally. If the connection to the database is kept open, the model remains in memory, which allows for faster inference on subsequent calls. If you want to free up memory, you can close the connection.

## Additional resources

- [Hugging Face datasets](https://huggingface.co/datasets)
- [Hugging Face tasks](https://huggingface.co/tasks)
