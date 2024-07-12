---
featured: true
tags: [engineering, product]
description: >-
  Quickly and easily transition from the confines of the OpenAI APIs to higher
  quality embeddings and unrestricted text generation models.
image: ".gitbook/assets/blog_image_switch_kit.png"
---

# Introducing the OpenAI Switch Kit: Move from closed to open-source AI in minutes

<div align="left">

<figure><img src=".gitbook/assets/image.png" alt="Author" width="100"><figcaption></figcaption></figure>

</div>

Cassandra Stumer and Silas Marvin

December 1, 2023

### Introduction

Last week's whirlwind of events with OpenAI CEO and founder Sam Altman stirred up quite a buzz in the industry. The whole deal left many of us scratching our heads about where OpenAI is headed. Between the corporate drama, valid worries about privacy and transparency, and ongoing issues around model performance, censorship, and the use of marketing scare tactics; it's no wonder there's a growing sense of dissatisfaction and distrust in proprietary models.

On the bright side, the open-source realm has emerged as a potent contender, not just in reaction to OpenAI's shortcomings but as a genuine advancement in its own right. We're all about making the benefits of open-source models accessible to as many folks as possible. So, we've made switching from OpenAI to open-source as easy as possible with a drop-in replacement. It lets users specify any model they’d like in just a few lines of code. We call it the OpenAI Switch Kit. Read on to learn more about why we think you’ll like it, or just try it now and see what you think.

### Is switching to open-source AI right for you?

We think so. Open-source models have made remarkable strides, not only catching up to proprietary counterparts but also surpassing them across multiple domains. The advantages are clear:

* **Performance & reliability:** Open-source models are increasingly comparable or superior across a wide range of tasks and performance metrics. Mistral and Llama-based models, for example, are easily faster than GPT 4. Reliability is another concern you may reconsider leaving in the hands of OpenAI. OpenAI’s API has suffered from several recent outages, and their rate limits can interrupt your app if there is a surge in usage. Open-source models enable greater control over your model’s latency, scalability and availability. Ultimately, the outcome of greater control is that your organization can produce a more dependable integration and a highly reliable production application.
* **Safety & privacy:** Open-source models are the clear winner when it comes to security sensitive AI applications. There are [enormous risks](https://www.infosecurity-magazine.com/news-features/chatgpts-datascraping-scrutiny/) associated with transmitting private data to external entities such as OpenAI. By contrast, open-source models retain sensitive information within an organization's own cloud environments. The data never has to leave your premises, so the risk is bypassed altogether – it’s enterprise security by default. At PostgresML, we offer such private hosting of LLM’s in your own cloud.
* **Model censorship:** A growing number of experts inside and outside of leading AI companies argue that model restrictions have gone too far. The Atlantic recently published an [article on AI’s “Spicy-Mayo Problem'' ](https://www.theatlantic.com/ideas/archive/2023/11/ai-safety-regulations-uncensored-models/676076/) which delves into the issues surrounding AI censorship. The titular example describes a chatbot refusing to return commands asking for a “dangerously spicy” mayo recipe. Censorship can affect baseline performance, and in the case of apps for creative work such as Sudowrite, unrestricted open-source models can actually be a key differentiating value for users.
* **Flexibility & customization:** Closed-source models like GPT3.5 Turbo are fine for generalized tasks, but leave little room for customization. Fine-tuning is highly restricted. Additionally, the headwinds at OpenAI have exposed the [dangerous reality of AI vendor lock-in](https://techcrunch.com/2023/11/21/openai-dangers-vendor-lock-in/). Open-source models such as MPT-7B, Llama V2 and Mistral 7B are designed with extensive flexibility for fine tuning, so organizations can create custom specifications and optimize model performance for their unique needs. This level of customization and flexibility opens the door for advanced techniques like DPO, PPO LoRa and more.

### Try it now

The Switch Kit is an open-source AI SDK that provides a drop in replacement for OpenAI’s chat completion endpoint.

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const korvus = require("korvus");
const client = korvus.newOpenSourceAI();
const results = client.chat_completions_create(
      "meta-llama/Meta-Llama-3-8B-Instruct",
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
import korvus
client = korvus.OpenSourceAI()
results = client.chat_completions_create(
    "meta-llama/Meta-Llama-3-8B-Instruct",
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
        "content": "Me matey, ya landed in me treasure trove o' riddles! But sorry to say, me lads, humans cannot eat helicopters in a single setting, for helicopters are mechanical devices and not food items. So there's no quantity to answer this one! Ahoy there, any other queries ye'd like to raise? Me hearty, we're always at yer service!",
        "role": "assistant"
      }
    }
  ],
  "created": 1701291672,
  "id": "abf042d2-9159-49cb-9fd3-eef16feb246c",
  "model": "meta-llama/Meta-Llama-3-8B-Instruct",
  "object": "chat.completion",
  "system_fingerprint": "eecec9d4-c28b-5a27-f90b-66c3fb6cee46",
  "usage": {
    "completion_tokens": 0,
    "prompt_tokens": 0,
    "total_tokens": 0
  }
}
```

!!! info

We don't charge per token, so OpenAI “usage” metrics are not particularly relevant. We'll be extending this data with more direct CPU/GPU resource utilization measurements for users who are interested, or need to pass real usage based pricing on to their own customers.

!!!

The above is an example using our open-source AI SDK with Meta-Llama-3-8B-Instruct, an incredibly popular and highly efficient 8 billion parameter model.

Notice there is near one to one relation between the parameters and return type of OpenAI’s `chat.completions.create` and our `chat_completion_create`.

Here is an example of streaming:

{% tabs %}
{% tab title="JavaScript" %}
```javascript
const korvus = require("korvus");
const client = korvus.newOpenSourceAI();
const it = client.chat_completions_create_stream(
      "meta-llama/Meta-Llama-3-8B-Instruct",
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
import korvus
client = korvus.OpenSourceAI()
results = client.chat_completions_create_stream(
     "meta-llama/Meta-Llama-3-8B-Instruct",
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
for c in results:
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
  "model": "meta-llama/Meta-Llama-3-8B-Instruct",
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
  "model": "meta-llama/Meta-Llama-3-8B-Instruct",
  "object": "chat.completion.chunk",
  "system_fingerprint": "f366d657-75f9-9c33-8e57-1e6be2cf62f3"
}
```

!!! info

We have truncated the output to two items

!!!

We also have asynchronous versions of the create and `create_stream` functions relatively named `create_async` and `create_stream_async`. Checkout [our documentation](https://postgresml.org/docs/guides/opensourceai) for a complete guide of the open-source AI SDK including guides on how to specify custom models.

PostgresML is free and open source. To run the above examples yourself [create an account](https://postgresml.org/signup), install korvus, and get running!

### Why use open-source models on PostgresML?

PostgresML is a complete MLOps platform in a simple PostgreSQL extension. It’s the tool our team wished they’d had scaling MLOps at Instacart during its peak years of growth. You can host your database with us or locally. However you want to engage, we know from experience that it’s better to bring your ML workload to the database rather than bringing the data to the codebase.

Fundamentally, PostgresML enables PostgreSQL to act as a GPU-powered AI application database — where you can both save models and index data. That eliminates the need for the myriad of separate services you have to tie together for your ML workflow. pgml + pgvector create a complete ML platform (vector DB, model store, inference service, open-source LLMs) all within open-source extensions for PostgreSQL. That takes a lot of the complexity out of your infra, and it's ultimately faster for your users.

We're bullish on the power of in-database and open-source ML/AI, and we’re excited for you to see the power of this approach yourself. You can try it out in our serverless database for $0, with usage based billing starting at just five cents an hour per GB GPU cache. You can even mess with it for free on our homepage.

As always, let us know what you think. Get in touch via email or on our Discord if you have any questions or feedback.
