---
description: Task of converting text written in one language into another language.
---

# Translation

Translation is the task of converting text written in one language into another language. You have the option to select from over 2000 models available on the Hugging Face [hub](https://huggingface.co/models?pipeline\_tag=translation) for translation.

```postgresql
select pgml.transform(
    inputs => array[
        'How are you?'
    ],
	task => '{
        "task": "translation", 
        "model": "google-t5/t5-base"
    }'::JSONB	
);
```

_Result_

```json
[
    {"translation_text": "Comment allez-vous ?"}
]
```
