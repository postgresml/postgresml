---
description: Numeric representation of text
---

# Embeddings

Embeddings are a numeric representation of text. They are used to represent words and sentences as vectors, an array of numbers. Embeddings can be used to find similar pieces of text, by comparing the similarity of the numeric vectors using a distance measure, or they can be used as input features for other machine learning models, since most algorithms can't use text directly.

Many pretrained LLMs can be used to generate embeddings from text within PostgresML. You can browse all the [models](https://huggingface.co/models?library=sentence-transformers) available to find the best solution on Hugging Face.

```sql
SELECT pgml.embed(
    'distilbert-base-uncased', 
    'Star Wars christmas special is on Disney'
    )::vector 
AS embedding
```

_Result_

```json
{
"embedding" : [-0.048401695,-0.20282568,0.2653648,0.12278256,0.24706738, ...]
}
```
