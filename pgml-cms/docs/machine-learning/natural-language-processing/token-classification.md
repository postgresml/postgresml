---
description: Task where labels are assigned to certain tokens in a text.
---

# Token Classification

Token classification is a task in natural language understanding, where labels are assigned to certain tokens in a text. Some popular subtasks of token classification include Named Entity Recognition (NER) and Part-of-Speech (PoS) tagging. NER models can be trained to identify specific entities in a text, such as individuals, places, and dates. PoS tagging, on the other hand, is used to identify the different parts of speech in a text, such as nouns, verbs, and punctuation marks.

### Named Entity Recognition

Named Entity Recognition (NER) is a task that involves identifying named entities in a text. These entities can include the names of people, locations, or organizations. The task is completed by labeling each token with a class for each named entity and a class named "0" for tokens that don't contain any entities. In this task, the input is text, and the output is the annotated text with named entities.

```sql
SELECT pgml.transform(
    inputs => ARRAY[
        'I am Omar and I live in New York City.'
    ],
    task => 'token-classification'
) as ner;
```

_Result_

```json
[[
    {"end": 9,  "word": "Omar", "index": 3,  "score": 0.997110, "start": 5,  "entity": "I-PER"}, 
    {"end": 27, "word": "New",  "index": 8,  "score": 0.999372, "start": 24, "entity": "I-LOC"}, 
    {"end": 32, "word": "York", "index": 9,  "score": 0.999355, "start": 28, "entity": "I-LOC"}, 
    {"end": 37, "word": "City", "index": 10, "score": 0.999431, "start": 33, "entity": "I-LOC"}
]]
```

### Part-of-Speech (PoS) Tagging

PoS tagging is a task that involves identifying the parts of speech, such as nouns, pronouns, adjectives, or verbs, in a given text. In this task, the model labels each word with a specific part of speech.

Look for models with `pos` to use a zero-shot classification model on the :hugs: Hugging Face model hub.

```sql
select pgml.transform(
	inputs => array [
  	'I live in Amsterdam.'
	],
	task => '{"task": "token-classification", 
              "model": "vblagoje/bert-english-uncased-finetuned-pos"
    }'::JSONB
) as pos;
```

_Result_

```json
[[
    {"end": 1,  "word": "i",         "index": 1, "score": 0.999, "start": 0,  "entity": "PRON"},
    {"end": 6,  "word": "live",      "index": 2, "score": 0.998, "start": 2,  "entity": "VERB"},
    {"end": 9,  "word": "in",        "index": 3, "score": 0.999, "start": 7,  "entity": "ADP"},
    {"end": 19, "word": "amsterdam", "index": 4, "score": 0.998, "start": 10, "entity": "PROPN"}, 
    {"end": 20, "word": ".",         "index": 5, "score": 0.999, "start": 19, "entity": "PUNCT"}
]]
```
