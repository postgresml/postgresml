---
description: The task of generating text using state of the art models.
---

# Text Generation

Text generation is the task of producing text. It has various use cases, including code generation, story generation, chatbots and more.

## Chat

Use this for conversational AI applications or when you need to provide instructions and maintain context.

```postgresql
SELECT pgml.transform(
    task => '{
        "task": "text-generation",
        "model": "meta-llama/Meta-Llama-3.1-8B-Instruct"
    }'::JSONB,
    inputs => ARRAY[
        '{"role": "system", "content": "You are a friendly and helpful chatbot"}'::JSONB,
        '{"role": "user", "content": "Tell me about yourself."}'::JSONB
    ]
) AS answer;
```

_Result_

```json
["I'm so glad you asked! I'm a friendly and helpful chatbot, designed to assist and converse with users like you. I'm a large language model, which means I've been trained on a massive dataset of text from various sources, including books, articles, and conversations. Th is training enables me to understand and respond to a wide range of topics and questions.\n\nI'm constantly learning and improving my la nguage processing abilities, so I can become more accurate and helpful over time. My primary goal is to provide accurate and relevant in formation, answer your questions, and engage in productive conversations.\n\nI'm not just limited to answering questions, though! I can  also:\n\n1. Generate text on a given topic or subject\n2. Offer suggestions and recommendations\n3. Summarize lengthy texts or articles\ n4. Translate text from one language to another\n5. Even create stories, poems, or jokes (if you'd like!)\n\nI'm here to help you with a ny questions, concerns, or topics you'd like to discuss. Feel free to ask me anything, and I'll do my best to assist you!"]
```

### Chat Parameters

We follow OpenAI's standard for model parameters:
- `frequency_penalty` - Penalizes the frequency of tokens
- `logit_bias` - Modify the likelihood of specified tokens
- `logprobs` - Return logprobs of the most likely token(s)
- `top_logprobs` - The number of most likely tokens to return at each token position
- `max_tokens` - The maximum number of tokens to generate
- `n` - The number of completions to build out
- `presence_penalty` - Control new token penalization
- `response_format` - The format of the response
- `seed` - The seed for randomness
- `stop` - An array of sequences to stop on
- `temperature` - The temperature for sampling
- `top_p` - An alternative sampling method

For more information on these parameters see [OpenAI's docs](https://platform.openai.com/docs/api-reference/chat).

An example with some common parameters:

```postgresql
SELECT pgml.transform(
    task => '{
        "task": "text-generation",
        "model": "meta-llama/Meta-Llama-3.1-8B-Instruct"
    }'::JSONB,
    inputs => ARRAY[
        '{"role": "system", "content": "You are a friendly and helpful chatbot"}'::JSONB,
        '{"role": "user", "content": "Tell me about yourself."}'::JSONB
    ],
    args => '{
        "max_tokens": 10,
        "temperature": 0.75,
        "seed": 10
    }'::JSONB
) AS answer;
```

_Result_
```json
["I'm so glad you asked! I'm a"]
```

## Completions

Use this for simpler text-generation tasks like completing sentences or generating content based on a prompt.

```postgresql
SELECT pgml.transform(
    task => '{
        "task": "text-generation",
        "model": "meta-llama/Meta-Llama-3.1-8B-Instruct"
    }'::JSONB,
    inputs => ARRAY[
        'Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone'
    ]
) AS answer;
```

_Result_

```json
[", Nine for Mortal Men doomed to die, One for the Dark Lord on"]
```

### Completion Parameters

We follow OpenAI's standard for model parameters:
- `best_of` - Generates "best_of" completions
- `echo` - Echo back the prompt
- `frequency_penalty` - Penalizes the frequency of tokens
- `logit_bias` - Modify the likelihood of specified tokens
- `logprobs` - Return logprobs of the most likely token(s)
- `max_tokens` - The maximum number of tokens to generate
- `n` - The number of completions to build out
- `presence_penalty` - Control new token penalization
- `seed` - The seed for randomness
- `stop` - An array of sequences to stop on
- `temperature` - The temperature for sampling
- `top_p` - An alternative sampling method

For more information on these parameters see [OpenAI's docs](https://platform.openai.com/docs/api-reference/completions/create).

An example with some common parameters:

```postgresql
SELECT pgml.transform(
    task => '{
        "task": "text-generation",
        "model": "meta-llama/Meta-Llama-3.1-8B-Instruct"
    }'::JSONB,
    inputs => ARRAY[
        'Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone'
    ],
    args => '{
        "max_tokens": 10,
        "temperature": 0.75,
        "seed": 10
    }'::JSONB
) AS answer;
```

_Result_
```json
[", Nine for Mortal Men doomed to die,"]
```
