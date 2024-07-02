---
description: Task of creating a condensed version of a document.
---

# Summarization

Summarization involves creating a condensed version of a document that includes the important information while reducing its length. Different models can be used for this task, with some models extracting the most relevant text from the original document, while other models generate completely new text that captures the essence of the original content.

## Example

{% tabs %}
{% tab title="SQL" %}

```postgresql
SELECT pgml.transform(
        task => '{
          "task": "summarization", 
          "model": "google/pegasus-xsum"
    }'::JSONB,
        inputs => array[
         'Paris is the capital and most populous city of France, with an estimated population of 2,175,601 residents as of 2018,
         in an area of more than 105 square kilometres (41 square miles). The City of Paris is the centre and seat of government
         of the region and province of Île-de-France, or Paris Region, which has an estimated population of 12,174,880,
         or about 18 percent of the population of France as of 2017.'
        ]
);
```

{% endtab %}
{% tab title="Result" %}

```json
[
  {
    "summary_text": "The City of Paris is the centre and seat of government of the region and province of le-de-France, or Paris Region, which has an estimated population of 12,174,880, or about 18 percent of the population of France as of 2017."
  }
]
```

{% endtab %}
{% endtabs %}

### Additional resources

- [Hugging Face documentation](https://huggingface.co/tasks/summarization)
- [google/pegasus-xsum](https://huggingface.co/google/pegasus-xsum)
