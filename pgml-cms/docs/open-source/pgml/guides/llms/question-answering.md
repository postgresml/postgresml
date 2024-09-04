---
description: Retrieve the answer to a question from a given text.
---

# Question answering

Question answering models are designed to retrieve the answer to a question from a given text, which can be particularly useful for searching for information within a document. It's worth noting that some question answering models are capable of generating answers even without any contextual information.

## Example

{% tabs %}
{% tab title="SQL" %}

```postgresql
SELECT pgml.transform(
    'question-answering',
    inputs => ARRAY[
        '{
            "question": "Where do I live?",
            "context": "My name is Merve and I live in İstanbul."
        }'
    ]
) AS answer;
```

{% endtab %}

{% tab title="Result" %}

```json
{
    "end"   :  39, 
    "score" :  0.9538117051124572, 
    "start" :  31, 
    "answer": "İstanbul"
}
```

{% endtab %}
{% endtabs %}


### Additional resources

- [Hugging Face documentation](https://huggingface.co/tasks/question-answering)
