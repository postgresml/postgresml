---
description: Stream generated text from state of the art models.
---

# pgml.transform_stream

`pgml.transform_stream` mirrors `pgml.transform` with two caveats:
- It returns a `SETOF JSONB` instead of `JSONB`.
- It only works with the `text-generation` task.

The `pgml.transform_stream` function is overloaded and can be used to chat with messages or complete text.

## Chat

Use this for conversational AI applications or when you need to provide instructions and maintain context.

### API

```postgresql
pgml.transform_stream(
    task JSONB,
    inputs ARRAY[]::JSONB,
    args JSONB
)
```

| Argument | Description |
|----------|-------------|
| task | The task object with required keys of `task` and `model`. |
| inputs | The input chat messages. | 
| args | The additional arguments for the model. |

A simple example using `meta-llama/Meta-Llama-3.1-8B-Instruct`:

```postgresql
SELECT pgml.transform_stream(
    task => '{
        "task": "conversational",
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
["I"]
["'m"]
[" so"]
[" glad"]
[" you"]
[" asked"]
["!"]
[" I"]
["'m"]
[" a"]
...
```
Results have been truncated for sanity.

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
SELECT pgml.transform_stream(
    task => '{
        "task": "conversational",
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
["I"]
["'m"]
[" so"]
[" glad"]
[" you"]
[" asked"]
["!"]
[" I"]
["'m"]
[" a"]
```

## Completion

Use this for simpler text-generation tasks like completing sentences or generating content based on a prompt.

### API

```postgresql
pgml.transform_stream(
    task JSONB,
    input text,
    args JSONB
)
```
| Argument | Description |
|----------|-------------|
| task | The task object with required keys of `task` and `model`. |
| input | The text to complete. | 
| args | The additional arguments for the model. |

A simple example using `meta-llama/Meta-Llama-3.1-8B-Instruct`:

```postgresql
SELECT pgml.transform_stream(
    task => '{
        "task": "text-generation",
        "model": "meta-llama/Meta-Llama-3.1-8B-Instruct"
    }'::JSONB,
    input => 'Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone'
) AS answer;
```

_Result_

```json
[","]
[" Nine"]
[" for"]
[" Mort"]
["al"]
[" Men"]
[" doomed"]
[" to"]
[" die"]
[","]
[" One"]
[" for"]
[" the"]
[" Dark"]
[" Lord"]
[" on"]
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
SELECT pgml.transform_stream(
    task => '{
        "task": "text-generation",
        "model": "meta-llama/Meta-Llama-3.1-8B-Instruct"
    }'::JSONB,
    input => 'Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone',
    args => '{
        "max_tokens": 10,
        "temperature": 0.75,
        "seed": 10
    }'::JSONB
) AS answer;
```

_Result_

```json
[","]
[" Nine"]
[" for"]
[" Mort"]
["al"]
[" Men"]
[" doomed"]
[" to"]
[" die"]
[","]
```
