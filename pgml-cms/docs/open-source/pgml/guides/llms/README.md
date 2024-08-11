# LLMs

## Supported tasks

PostgresML currently supports most LLM tasks for Natural Language Processing available on Hugging Face:

| Task                                                    | Name | Description |
|---------------------------------------------------------|-------------|---------|
| [Fill mask](../guides/llms/fill-mask.md)                | `key-mask` | Fill in the blank in a sentence. |
| [Question answering](../guides/llms/question-answering.md)             | `question-answering` | Answer a question based on a context. |
| [Summarization](../guides/llms/summarization.md)                       | `summarization` | Summarize a long text. |
| [Text classification](../guides/llms/text-classification.md)           | `text-classification` | Classify a text as positive or negative. |
| [Text generation](../guides/llms/text-generation.md)                   | `text-generation` | Generate text based on a prompt. |
| [Text-to-text generation](../guides/llms/text-to-text-generation.md)   | `text-to-text-generation` | Generate text based on an instruction in the prompt. |
| [Token classification](../guides/llms/token-classification.md)         | `token-classification` | Classify tokens in a text. |
| [Translation](../guides/llms/translation.md)                           | `translation` | Translate text from one language to another. |
| [Zero-shot classification](../guides/llms/zero-shot-classification.md) | `zero-shot-classification` | Classify a text without training data. |
| Conversational                                          | `conversational` | Engage in a conversation with the model, e.g. chatbot. |

## Structured inputs

Both versions of the `pgml.transform()` function also support structured inputs, formatted with JSON. Structured inputs are used with the conversational task, e.g. to differentiate between the system and user prompts. Simply replace the text array argument with an array of JSONB objects.


## Additional resources

- [Hugging Face datasets](https://huggingface.co/datasets)
- [Hugging Face tasks](https://huggingface.co/tasks)
