---
description: Task where the model predicts a class that it hasn't seen during the training.
---

# Zero-shot Classification

Zero Shot Classification is a task where the model predicts a class that it hasn't seen during the training phase. This task leverages a pre-trained language model and is a type of transfer learning. Transfer learning involves using a model that was initially trained for one task in a different application. Zero Shot Classification is especially helpful when there is a scarcity of labeled data available for the specific task at hand.

In the example provided below, we will demonstrate how to classify a given sentence into a class that the model has not encountered before. To achieve this, we make use of `args` in the SQL query, which allows us to provide `candidate_labels`. You can customize these labels to suit the context of your task. We will use `facebook/bart-large-mnli` model.

Look for models with `mnli` to use a zero-shot classification model on the :hugs: Hugging Face model hub.

```postgresql
SELECT pgml.transform(
    inputs => ARRAY[
        'I have a problem with my iphone that needs to be resolved asap!!'
    ],
    task => '{
                "task": "zero-shot-classification", 
                "model": "facebook/bart-large-mnli"
             }'::JSONB,
    args => '{
                "candidate_labels": ["urgent", "not urgent", "phone", "tablet", "computer"]
             }'::JSONB
) AS zero_shot;
```

_Result_

```json
[
    {
        "labels": ["urgent", "phone", "computer", "not urgent", "tablet"], 
        "scores": [0.503635, 0.47879, 0.012600, 0.002655, 0.002308], 
        "sequence": "I have a problem with my iphone that needs to be resolved asap!!"
    }
]
```
