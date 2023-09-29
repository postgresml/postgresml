---
description: Task to fill words in a sentence that are hidden
---

# Fill Mask

Fill-mask refers to a task where certain words in a sentence are hidden or "masked", and the objective is to predict what words should fill in those masked positions. Such models are valuable when we want to gain statistical insights about the language used to train the model.&#x20;

```sql
SELECT pgml.transform(
    task => '{
        "task" : "fill-mask"
    }'::JSONB,
    inputs => ARRAY[
        'Paris is the <mask> of France.'

    ]
) AS answer;
```

_Result_

```json
[
    {"score": 0.679, "token": 812,   "sequence": "Paris is the capital of France.",    "token_str": " capital"}, 
    {"score": 0.051, "token": 32357, "sequence": "Paris is the birthplace of France.", "token_str": " birthplace"}, 
    {"score": 0.038, "token": 1144,  "sequence": "Paris is the heart of France.",      "token_str": " heart"}, 
    {"score": 0.024, "token": 29778, "sequence": "Paris is the envy of France.",       "token_str": " envy"}, 
    {"score": 0.022, "token": 1867,  "sequence": "Paris is the Capital of France.",    "token_str": " Capital"}]
```
