---
description: Task oof creating a condensed version of a document
---

# Summarization

Summarization involves creating a condensed version of a document that includes the important information while reducing its length. Different models can be used for this task, with some models extracting the most relevant text from the original document, while other models generate completely new text that captures the essence of the original content.

```sql
select pgml.transform(
	task => '{"task": "summarization", 
              "model": "sshleifer/distilbart-cnn-12-6"
    }'::JSONB,
	inputs => array[
	'Paris is the capital and most populous city of France, with an estimated population of 2,175,601 residents as of 2018, in an area of more than 105 square kilometres (41 square miles). The City of Paris is the centre and seat of government of the region and province of Île-de-France, or Paris Region, which has an estimated population of 12,174,880, or about 18 percent of the population of France as of 2017.'
	]
);
```

_Result_

```json
[
    {"summary_text": " Paris is the capital and most populous city of France, with an estimated population of 2,175,601 residents as of 2018 . The city is the centre and seat of government of the region and province of Île-de-France, or Paris Region . Paris Region has an estimated 18 percent of the population of France as of 2017 ."}
    ]
```

You can control the length of summary\_text by passing `min_length` and `max_length` as arguments to the SQL query.

```sql
select pgml.transform(
	task => '{"task": "summarization", 
              "model": "sshleifer/distilbart-cnn-12-6"
    }'::JSONB,
	inputs => array[
	'Paris is the capital and most populous city of France, with an estimated population of 2,175,601 residents as of 2018, in an area of more than 105 square kilometres (41 square miles). The City of Paris is the centre and seat of government of the region and province of Île-de-France, or Paris Region, which has an estimated population of 12,174,880, or about 18 percent of the population of France as of 2017.'
	],
	args => '{
            "min_length" : 20,
            "max_length" : 70
	}'::JSONB
);
```

```json
[
    {"summary_text": " Paris is the capital and most populous city of France, with an estimated population of 2,175,601 residents as of 2018 . City of Paris is centre and seat of government of the region and province of Île-de-France, or Paris Region, which has an estimated 12,174,880, or about 18 percent"
    }  
]
```
