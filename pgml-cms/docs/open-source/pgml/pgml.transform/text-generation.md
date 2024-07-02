---
description: Task of producing new text
---

# Text Generation

Text generation is the task of producing new text, such as filling in incomplete sentences or paraphrasing existing text. It has various use cases, including code generation and story generation. Completion generation models can predict the next word in a text sequence, while text-to-text generation models are trained to learn the mapping between pairs of texts, such as translating between languages. Popular models for text generation include GPT-based models, T5, T0, and BART. These models can be trained to accomplish a wide range of tasks, including text classification, summarization, and translation.

```postgresql
SELECT pgml.transform(
    task => 'text-generation',
    inputs => ARRAY[
        'Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone'
    ]
) AS answer;
```

_Result_

```json
[
    [
        {"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone, and eight for the Dragon-lords in their halls of blood.\n\nEach of the guild-building systems is one-man"}
    ]
]
```

### Model from hub

To use a specific model from :hugging: model hub, pass the model name along with task name in task.

```postgresql
SELECT pgml.transform(
    task => '{
        "task" : "text-generation",
        "model" : "gpt2-medium"
    }'::JSONB,
    inputs => ARRAY[
        'Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone'
    ]
) AS answer;
```

_Result_

```json
[
    [{"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone.\n\nThis place has a deep connection to the lore of ancient Elven civilization. It is home to the most ancient of artifacts,"}]
]
```

### Maximum Length

To make the generated text longer, you can include the argument `max_length` and specify the desired maximum length of the text.

```postgresql
SELECT pgml.transform(
    task => '{
        "task" : "text-generation",
        "model" : "gpt2-medium"
    }'::JSONB,
    inputs => ARRAY[
        'Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone'
    ],
    args => '{
			"max_length" : 200
		}'::JSONB 
) AS answer;
```

_Result_

```json
[
    [{"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone, Three for the Dwarfs and the Elves, One for the Gnomes of the Mines, and Two for the Elves of Dross.\"\n\nHobbits: The Fellowship is the first book of J.R.R. Tolkien's story-cycle, and began with his second novel - The Two Towers - and ends in The Lord of the Rings.\n\n\nIt is a non-fiction novel, so there is no copyright claim on some parts of the story but the actual text of the book is copyrighted by author J.R.R. Tolkien.\n\n\nThe book has been classified into two types: fantasy novels and children's books\n\nHobbits: The Fellowship is the first book of J.R.R. Tolkien's story-cycle, and began with his second novel - The Two Towers - and ends in The Lord of the Rings.It"}]
]
```

### Return Sequences

If you want the model to generate more than one output, you can specify the number of desired output sequences by including the argument `num_return_sequences` in the arguments.

```postgresql
SELECT pgml.transform(
    task => '{
        "task" : "text-generation",
        "model" : "gpt2-medium"
    }'::JSONB,
    inputs => ARRAY[
        'Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone'
    ],
    args => '{
			"num_return_sequences" : 3
		}'::JSONB 
) AS answer;
```

_Result_

```json
[
    [
        {"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone, and Thirteen for the human-men in their hall of fire.\n\nAll of us, our families, and our people"}, 
        {"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone, and the tenth for a King! As each of these has its own special story, so I have written them into the game."}, 
        {"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone… What's left in the end is your heart's desire after all!\n\nHans: (Trying to be brave)"}
    ]
]
```

### Beam Search

Text generation typically utilizes a greedy search algorithm that selects the word with the highest probability as the next word in the sequence. However, an alternative method called beam search can be used, which aims to minimize the possibility of overlooking hidden high probability word combinations. Beam search achieves this by retaining the num\_beams most likely hypotheses at each step and ultimately selecting the hypothesis with the highest overall probability. We set `num_beams > 1` and `early_stopping=True` so that generation is finished when all beam hypotheses reached the EOS token.

```postgresql
SELECT pgml.transform(
    task => '{
        "task" : "text-generation",
        "model" : "gpt2-medium"
    }'::JSONB,
    inputs => ARRAY[
        'Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone'
    ],
    args => '{
			"num_beams" : 5,
			"early_stopping" : true
		}'::JSONB 
) AS answer;
```

_Result_

```json
[[
    {"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone, Nine for the Dwarves in their caverns of ice, Ten for the Elves in their caverns of fire, Eleven for the"}
]]
```

Sampling methods involve selecting the next word or sequence of words at random from the set of possible candidates, weighted by their probabilities according to the language model. This can result in more diverse and creative text, as well as avoiding repetitive patterns. In its most basic form, sampling means randomly picking the next word $w\_t$ according to its conditional probability distribution: $$w_t \approx P(w_t|w_{1:t-1})$$

However, the randomness of the sampling method can also result in less coherent or inconsistent text, depending on the quality of the model and the chosen sampling parameters such as temperature, top-k, or top-p. Therefore, choosing an appropriate sampling method and parameters is crucial for achieving the desired balance between creativity and coherence in generated text.

You can pass `do_sample = True` in the arguments to use sampling methods. It is recommended to alter `temperature` or `top_p` but not both.

### _Temperature_

```postgresql
SELECT pgml.transform(
    task => '{
        "task" : "text-generation",
        "model" : "gpt2-medium"
    }'::JSONB,
    inputs => ARRAY[
        'Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone'
    ],
    args => '{
			"do_sample" : true,
			"temperature" : 0.9
		}'::JSONB 
) AS answer;
```

_Result_

```json
[[{"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone, and Thirteen for the Giants and Men of S.A.\n\nThe First Seven-Year Time-Traveling Trilogy is"}]]
```

### _Top p_

```postgresql
SELECT pgml.transform(
    task => '{
        "task" : "text-generation",
        "model" : "gpt2-medium"
    }'::JSONB,
    inputs => ARRAY[
        'Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone'
    ],
    args => '{
			"do_sample" : true,
			"top_p" : 0.8
		}'::JSONB 
) AS answer;
```

_Result_

```json
[[{"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone, Four for the Elves of the forests and fields, and Three for the Dwarfs and their warriors.\" ―Lord Rohan [src"}]]
```
