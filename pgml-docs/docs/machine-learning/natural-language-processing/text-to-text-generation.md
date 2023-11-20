# Text-to-Text Generation

Text-to-text generation methods, such as T5, are neural network architectures designed to perform various natural language processing tasks, including summarization, translation, and question answering. T5 is a transformer-based architecture pre-trained on a large corpus of text data using denoising autoencoding. This pre-training process enables the model to learn general language patterns and relationships between different tasks, which can be fine-tuned for specific downstream tasks. During fine-tuning, the T5 model is trained on a task-specific dataset to learn how to perform the specific task.&#x20;

_Translation_

```sql
SELECT pgml.transform(
    task => '{
        "task" : "text2text-generation"
    }'::JSONB,
    inputs => ARRAY[
        'translate from English to French: I''m very happy'
    ]
) AS answer;
```

_Result_

```json
[
    {"generated_text": "Je suis très heureux"}
]
```

Similar to other tasks, we can specify a model for text-to-text generation.

```sql
SELECT pgml.transform(
    task => '{
        "task" : "text2text-generation",
        "model" : "bigscience/T0"
    }'::JSONB,
    inputs => ARRAY[
        'Is the word ''table'' used in the same meaning in the two previous sentences? Sentence A: you can leave the books on the table over there. Sentence B: the tables in this book are very hard to read.'

    ]
) AS answer;

```
