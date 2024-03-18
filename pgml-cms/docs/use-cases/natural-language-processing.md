# Natural Language Processing

PostgresML integrates [🤗 Hugging Face Transformers](https://huggingface.co/transformers) to bring state-of-the-art models into the data layer. There are tens of thousands of pre-trained models with pipelines to turn raw inputs into useful results. Many state of the art deep learning architectures have been published and made available for download. You will want to browse all the [models](https://huggingface.co/models) available to find the perfect solution for your [dataset](https://huggingface.co/dataset) and [task](https://huggingface.co/tasks). For instance, with PostgresML you can:

* Perform natural language processing (NLP) tasks like sentiment analysis, question and answering, translation, summarization and text generation
* Access 1000s of state-of-the-art language models like GPT-2, GPT-J, GPT-Neo from :hugs: HuggingFace model hub
* Fine tune large language models (LLMs) on your own text data for different tasks
* Use your existing PostgreSQL database as a vector database by generating embeddings from text stored in the database.

See [pgml.transform](../api/sql-extension/pgml.transform/ "mention") for examples of using transformers or [pgml.tune.md](../api/sql-extension/pgml.tune.md "mention") for fine tuning.
