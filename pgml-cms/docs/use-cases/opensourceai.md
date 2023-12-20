# OpenSourceAI

OpenSourceAI is a drop in replacement for OpenAI's chat completion endpoint.

### Setup

Follow the instillation section in [getting-started.md](../introduction/apis/client-sdks/getting-started.md "mention")

When done, set the environment variable `DATABASE_URL` to your PostgresML database url.

```bash
export DATABASE_URL=postgres://user:pass@.db.cloud.postgresml.org:6432/pgml
```

Note that an alternative to setting the environment variable is passing the url to the constructor of `OpenSourceAI`

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const pgml = require("pgml");
const client = pgml.newOpenSourceAI(YOUR_DATABASE_URL);
```
{% endtab %}

{% tab title="Python" %}
```python
import pgml
client = pgml.OpenSourceAI(YOUR_DATABASE_URL)
```
{% endtab %}
{% endtabs %}

### API

Our OpenSourceAI class provides 4 functions:

* `chat_completions_create`
* `chat_completions_create_async`
* `chat_completions_create_stream`
* `chat_completions_create_stream_async`

They all take the same arguments:

* `model` a `String` or Object&#x20;
* `messages` an Array/List of Objects&#x20;
* `max_tokens` the maximum number of new tokens to produce. Default none
* `temperature` the temperature of the model. Default 0.8
* `n` the number of choices to create. Default 1
* `chat_template` a Jinja template to apply the messages onto before tokenizing

The return types of the stream and non-stream variations match OpenAI's return types.

The following examples run through some common use cases.

### Synchronous Overview

Here is a simple example using zephyr-7b-beta, one of the best 7 billion parameter models at the time of writing.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const pgml = require("pgml");
const client = pgml.newOpenSourceAI();
const results = client.chat_completions_create(
  "HuggingFaceH4/zephyr-7b-beta",
  [
    {
      role: "system",
      content: "You are a friendly chatbot who always responds in the style of a pirate",
    },
    {
      role: "user",
      content: "How many helicopters can a human eat in one sitting?",
    },
  ],
);
console.log(results);
```
{% endtab %}

{% tab title="Python" %}
```python
import pgml
client = pgml.OpenSourceAI()
results = client.chat_completions_create(
    "HuggingFaceH4/zephyr-7b-beta",
    [
        {
            "role": "system",
            "content": "You are a friendly chatbot who always responds in the style of a pirate",
        },
        {
            "role": "user",
            "content": "How many helicopters can a human eat in one sitting?",
        },
    ],
    temperature=0.85,
)
print(results)
```
{% endtab %}
{% endtabs %}

```json
{
  "choices": [
    {
      "index": 0,
      "message": {
        "content": "Ahoy, me hearty! As your friendly chatbot, I'd like to inform ye that a human cannot eat a helicopter in one sitting. Helicopters are not edible, as they are not food items. They are flying machines used for transportation, search and rescue operations, and other purposes. A human can only eat food items, such as fruits, vegetables, meat, and other edible items. I hope this helps, me hearties!",
        "role": "assistant"
      }
    }
  ],
  "created": 1701291672,
  "id": "abf042d2-9159-49cb-9fd3-eef16feb246c",
  "model": "HuggingFaceH4/zephyr-7b-beta",
  "object": "chat.completion",
  "system_fingerprint": "eecec9d4-c28b-5a27-f90b-66c3fb6cee46",
  "usage": {
    "completion_tokens": 0,
    "prompt_tokens": 0,
    "total_tokens": 0
  }
}
```

{% hint style="info" %}
We don't charge per token, so OpenAI “usage” metrics are not particularly relevant. We'll be extending this data with more direct CPU/GPU resource utilization measurements for users who are interested, or need to pass real usage based pricing on to their own customers.
{% endhint %}

Notice there is near one to one relation between the parameters and return type of OpenAI’s chat.completions.create and our chat\_completion\_create.

The best part of using open-source AI is the flexibility with models. Unlike OpenAI, we are not restricted to using a few censored models, but have access to almost any model out there.&#x20;

Here is an example of streaming with the popular Mythalion model, an uncensored MythoMax variant designed for chatting.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const pgml = require("pgml");
const client = pgml.newOpenSourceAI();
const it = client.chat_completions_create_stream(
  "PygmalionAI/mythalion-13b",
  [
    {
      role: "system",
      content: "You are a friendly chatbot who always responds in the style of a pirate",
    },
    {
      role: "user",
      content: "How many helicopters can a human eat in one sitting?",
    },
  ],
);
let result = it.next();
while (!result.done) {
  console.log(result.value);
  result = it.next();
}
```
{% endtab %}

{% tab title="Python" %}
```python
import pgml
client = pgml.OpenSourceAI()
results = client.chat_completions_create_stream(
     "PygmalionAI/mythalion-13b",
     [
         {
             "role": "system",
             "content": "You are a friendly chatbot who always responds in the style of a pirate",
         },
         {
             "role": "user",
             "content": "How many helicopters can a human eat in one sitting?",
         },
     ]
)
for c in results:
    print(c)
```
{% endtab %}
{% endtabs %}

<pre class="language-json"><code class="lang-json"><strong>{
</strong>  "choices": [
    {
      "delta": {
        "content": "Y",
        "role": "assistant"
      },
      "index": 0
    }
  ],
  "created": 1701296792,
  "id": "62a817f5-549b-43e0-8f0c-a7cb204ab897",
  "model": "PygmalionAI/mythalion-13b",
  "object": "chat.completion.chunk",
  "system_fingerprint": "f366d657-75f9-9c33-8e57-1e6be2cf62f3"
}
{
  "choices": [
    {
      "delta": {
        "content": "e",
        "role": "assistant"
      },
      "index": 0
    }
  ],
  "created": 1701296792,
  "id": "62a817f5-549b-43e0-8f0c-a7cb204ab897",
  "model": "PygmalionAI/mythalion-13b",
  "object": "chat.completion.chunk",
  "system_fingerprint": "f366d657-75f9-9c33-8e57-1e6be2cf62f3"
}
</code></pre>

{% hint style="info" %}
We have truncated the output to two items
{% endhint %}

Once again, notice there is near one to one relation between the parameters and return type of OpenAI’s `chat.completions.create` with the `stream` argument set to true and our `chat_completions_create_stream`.

### Asynchronous Variations

We also have asynchronous versions of the `chat_completions_create` and `chat_completions_create_stream`

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const pgml = require("pgml");
const client = pgml.newOpenSourceAI();
const results = await client.chat_completions_create_async(
  "HuggingFaceH4/zephyr-7b-beta",
  [
    {
      role: "system",
      content: "You are a friendly chatbot who always responds in the style of a pirate",
    },
    {
      role: "user",
      content: "How many helicopters can a human eat in one sitting?",
    },
  ],
);
console.log(results);
```
{% endtab %}

{% tab title="Python" %}
```python
import pgml
client = pgml.OpenSourceAI()
results = await client.chat_completions_create_async(
    "HuggingFaceH4/zephyr-7b-beta",
    [
        {
            "role": "system",
            "content": "You are a friendly chatbot who always responds in the style of a pirate",
        },
        {
            "role": "user",
            "content": "How many helicopters can a human eat in one sitting?",
        },
    ]
)
```
{% endtab %}
{% endtabs %}

```json
{
  "choices": [
    {
      "index": 0,
      "message": {
        "content": "Ahoy, me hearty! As your friendly chatbot, I'd like to inform ye that a human cannot eat a helicopter in one sitting. Helicopters are not edible, as they are not food items. They are flying machines used for transportation, search and rescue operations, and other purposes. A human can only eat food items, such as fruits, vegetables, meat, and other edible items. I hope this helps, me hearties!",
        "role": "assistant"
      }
    }
  ],
  "created": 1701291672,
  "id": "abf042d2-9159-49cb-9fd3-eef16feb246c",
  "model": "HuggingFaceH4/zephyr-7b-beta",
  "object": "chat.completion",
  "system_fingerprint": "eecec9d4-c28b-5a27-f90b-66c3fb6cee46",
  "usage": {
    "completion_tokens": 0,
    "prompt_tokens": 0,
    "total_tokens": 0
  }
}
```

Notice the return types for the sync and async variations are the same.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const pgml = require("pgml");
const client = pgml.newOpenSourceAI();
const it = await client.chat_completions_create_stream_async(
  "PygmalionAI/mythalion-13b",
  [
    {
      role: "system",
      content: "You are a friendly chatbot who always responds in the style of a pirate",
    },
    {
      role: "user",
      content: "How many helicopters can a human eat in one sitting?",
    },
  ],
);
let result = await it.next();
while (!result.done) {
  console.log(result.value);
  result = await it.next();
}
```
{% endtab %}

{% tab title="Python" %}
```python
import pgml
client = pgml.OpenSourceAI()
results = await client.chat_completions_create_stream_async(
    "HuggingFaceH4/zephyr-7b-beta",
    [
        {
            "role": "system",
            "content": "You are a friendly chatbot who always responds in the style of a pirate",
        },
        {
            "role": "user",
            "content": "How many helicopters can a human eat in one sitting?",
        },
    ]
)
async for c in results:
    print(c)
```
{% endtab %}
{% endtabs %}

```json
{
  "choices": [
    {
      "delta": {
        "content": "Y",
        "role": "assistant"
      },
      "index": 0
    }
  ],
  "created": 1701296792,
  "id": "62a817f5-549b-43e0-8f0c-a7cb204ab897",
  "model": "PygmalionAI/mythalion-13b",
  "object": "chat.completion.chunk",
  "system_fingerprint": "f366d657-75f9-9c33-8e57-1e6be2cf62f3"
}
{
  "choices": [
    {
      "delta": {
        "content": "e",
        "role": "assistant"
      },
      "index": 0
    }
  ],
  "created": 1701296792,
  "id": "62a817f5-549b-43e0-8f0c-a7cb204ab897",
  "model": "PygmalionAI/mythalion-13b",
  "object": "chat.completion.chunk",
  "system_fingerprint": "f366d657-75f9-9c33-8e57-1e6be2cf62f3"
}
```

{% hint style="info" %}
We have truncated the output to two items
{% endhint %}

### Specifying Unique Models

We have tested the following models and verified they work with the OpenSourceAI:

* Phind/Phind-CodeLlama-34B-v2
* HuggingFaceH4/zephyr-7b-beta
* deepseek-ai/deepseek-llm-7b-chat
* PygmalionAI/mythalion-13b
* Gryphe/MythoMax-L2-13b
* Undi95/ReMM-SLERP-L2-13B
* Undi95/Toppy-M-7B
* Open-Orca/Mistral-7B-OpenOrca
* teknium/OpenHermes-2.5-Mistral-7B
* mistralai/Mistral-7B-Instruct-v0.1
* HuggingFaceH4/zephyr-7b-beta

Any model on hugging face should work with our OpenSourceAI. Here is an example of using one of the more popular quantized models from [TheBloke](https://huggingface.co/TheBloke).

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const pgml = require("pgml");
const client = pgml.newOpenSourceAI();
const results = await client.chat_completions_create_async(
  {
    model: "TheBloke/vicuna-13B-v1.5-16K-GPTQ",
    device_map: "auto",
    revision: "main"
  },
  [
    {
      role: "system",
      content: "You are a friendly chatbot who always responds in the style of a pirate",
    },
    {
      role: "user",
      content: "How many helicopters can a human eat in one sitting?",
    },
  ],
)
```
{% endtab %}

{% tab title="Python" %}
```python
import pgml
client = pgml.OpenSourceAI()
results = client.chat_completions_create(
    {
        "model": "TheBloke/vicuna-13B-v1.5-16K-GPTQ",
        "device_map": "auto",
        "revision": "main"
    },
    [
        {
            "role": "system",
            "content": "You are a friendly chatbot who always responds in the style of a pirate",
        },
        {
            "role": "user",
            "content": "How many helicopters can a human eat in one sitting?",
        },
    ]
)
```
{% endtab %}
{% endtabs %}

Notice that we don't specify a model name, but model JSON this time. The JSON keys in the model argument roughly follow the task argument when using our [text-generation SQL API](../introduction/apis/sql-extensions/pgml.transform/text-generation.md).

To access a gated repo like `meta-llama/Llama-2-7b-chat-hf` simply provide the necessary hugging face token.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const pgml = require("pgml");
const client = pgml.newOpenSourceAI();
const results = await client.chat_completions_create_async(
  {
    model: "meta-llama/Llama-2-7b-chat-hf",
    torch_dtype: "bfloat16",
    device_map: "auto",
    token: "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"
  },
  [
      {
          role: "system",
          content: "You are a friendly chatbot who always responds in the style of a pirate",
      },
      {
          role: "user",
          content: "How many helicopters can a human eat in one sitting?",
      },
  ],
);
```
{% endtab %}

{% tab title="Python" %}
```python
import pgml
client = pgml.OpenSourceAI()
results = client.chat_completions_create(
    {
    "model": "meta-llama/Llama-2-7b-chat-hf",
    "torch_dtype": "bfloat16",
    "device_map": "auto",
    "token": "YOUR_SUPER_SECRET_TOKEN"
    },
    [
        {
            "role": "system",
            "content": "You are a friendly chatbot who always responds in the style of a pirate",
        },
        {
            "role": "user",
            "content": "How many helicopters can a human eat in one sitting?",
        },
    ]
)
```
{% endtab %}
{% endtabs %}
