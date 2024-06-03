---
description: Task to fill words in a sentence that are hidden
---

# Fill-Mask

Fill-Mask is a task where certain words in a sentence are hidden or "masked", and the objective for the model is to predict what words should fill in those masked positions. Such models are valuable when we want to gain statistical insights about the language used to train the model.

## Example

{% tabs %}
{% tab title="SQL" %}

```postgresql
SELECT pgml.transform(
    task => '{
        "task" : "fill-mask"
    }'::JSONB,
    inputs => ARRAY[
        'Paris is the &lt;mask&gt; of France.'

    ]
) AS answer;
```

{% endtab %}

{% tab title="Result" %}

```json
[
  {
    "score": 0.6811484098434448,
    "token": 812,
    "sequence": "Paris is the capital of France.",
    "token_str": " capital"
  },
  {
    "score": 0.050908513367176056,
    "token": 32357,
    "sequence": "Paris is the birthplace of France.",
    "token_str": " birthplace"
  },
  {
    "score": 0.03812871500849724,
    "token": 1144,
    "sequence": "Paris is the heart of France.",
    "token_str": " heart"
  },
  {
    "score": 0.024047480896115303,
    "token": 29778,
    "sequence": "Paris is the envy of France.",
    "token_str": " envy"
  },
  {
    "score": 0.022767696529626846,
    "token": 1867,
    "sequence": "Paris is the Capital of France.",
    "token_str": " Capital"
  }
]
```

{% endtab %}
{% endtabs %}

### Additional resources

- [Hugging Face documentation](https://huggingface.co/tasks/fill-mask)
