# LLMs

PostgresML integrates [🤗 Hugging Face Transformers](https://huggingface.co/transformers) to bring state-of-the-art models into the data layer. There are tens of thousands of pre-trained models with pipelines to turn raw inputs into useful results. Many state of the art deep learning architectures have been published and made available for download. You will want to browse all the [models](https://huggingface.co/models) available to find the perfect solution for your [dataset](https://huggingface.co/dataset) and [task](https://huggingface.co/tasks). For instance, with PostgresML you can:

* Perform natural language processing (NLP) tasks like sentiment analysis, question and answering, translation, summarization and text generation
* Access 1000s of state-of-the-art language models like GPT-2, GPT-J, GPT-Neo from :hugs: HuggingFace model hub
* Fine tune large language models (LLMs) on your own text data for different tasks
* Use your existing PostgreSQL database as a vector database by generating embeddings from text stored in the database.

See [pgml.transform](/docs/open-source/pgml/api/pgml.transform "mention") for examples of using transformers or [pgml.tune](/docs/open-source/pgml/api/pgml.tune "mention") for fine tuning.

## Supported tasks

PostgresML currently supports most LLM tasks for Natural Language Processing available on Hugging Face:

| Task                                                    | Name | Description |
|---------------------------------------------------------|-------------|---------|
| [Fill mask](fill-mask.md)                | `key-mask` | Fill in the blank in a sentence. |
| [Question answering](question-answering.md)             | `question-answering` | Answer a question based on a context. |
| [Summarization](summarization.md)                       | `summarization` | Summarize a long text. |
| [Text classification](text-classification.md)           | `text-classification` | Classify a text as positive or negative. |
| [Text generation](text-generation.md)                   | `text-generation` | Generate text based on a prompt. |
| [Text-to-text generation](text-to-text-generation.md)   | `text-to-text-generation` | Generate text based on an instruction in the prompt. |
| [Token classification](token-classification.md)         | `token-classification` | Classify tokens in a text. |
| [Translation](translation.md)                           | `translation` | Translate text from one language to another. |
| [Zero-shot classification](zero-shot-classification.md) | `zero-shot-classification` | Classify a text without training data. |
| Conversational                                          | `conversational` | Engage in a conversation with the model, e.g. chatbot. |

## Structured inputs

Both versions of the `pgml.transform()` function also support structured inputs, formatted with JSON. Structured inputs are used with the conversational task, e.g. to differentiate between the system and user prompts. Simply replace the text array argument with an array of JSONB objects.


## Additional resources

- [Hugging Face datasets](https://huggingface.co/datasets)
- [Hugging Face tasks](https://huggingface.co/tasks)
