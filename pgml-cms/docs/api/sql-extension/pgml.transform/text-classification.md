---
description: Task that involves assigning a label or category to a given text.
---

# Text classification

Text classification is a task which includes sentiment analysis, natural language inference, and the assessment of grammatical correctness. It has a wide range of applications in fields such as marketing, customer service, and political analysis.

### Sentiment analysis

Sentiment analysis is a type of natural language processing technique which analyzes a piece of text to determine the sentiment or emotion expressed within. It can be used to classify a text as positive, negative, or neutral.

#### Example

{% tabs %}
{% tab title="SQL" %}

```postgresql
SELECT pgml.transform(
    task   => 'text-classification',
    inputs => ARRAY[
        'I love how amazingly simple ML has become!', 
        'I hate doing mundane and thankless tasks. ☹️'
    ]
) AS positivity;
```

{% endtab %}
{% tab title="Result" %}

```json
[
    {"label": "POSITIVE", "score": 0.9995759129524232}, 
    {"label": "NEGATIVE", "score": 0.9903519749641418}
]
```

{% endtab %}
{% endtabs %}


Currently, the default model used for text classification is a [fine-tuned version](https://huggingface.co/distilbert-base-uncased-finetuned-sst-2-english) of DistilBERT-base-uncased that has been specifically optimized for the [Stanford Sentiment Treebank dataset (sst2)](https://huggingface.co/datasets/stanfordnlp/sst2).

#### Using a specific model

To use one of the [thousands of models]((https://huggingface.co/models?pipeline\_tag=text-classification)) available on Hugging Face, include the name of the desired model and `text-classification` task as a JSONB object in the SQL query.

For example, if you want to use a RoBERTa model trained on around 40,000 English tweets and that has POS (positive), NEG (negative), and NEU (neutral) labels for its classes, include it in the query:

{% tabs %}
{% tab title="SQL" %}

```postgresql
SELECT pgml.transform(
    task  => '{
      "task": "text-classification", 
      "model": "finiteautomata/bertweet-base-sentiment-analysis"
    }'::JSONB,
    inputs => ARRAY[
        'I love how amazingly simple ML has become!', 
        'I hate doing mundane and thankless tasks. ☹️'
    ]
    
) AS positivity;
```

{% endtab %}
{% tab title="Result" %}

```json
[
    {"label": "POS", "score": 0.992932200431826}, 
    {"label": "NEG", "score": 0.975599765777588}
]
```

{% endtab %}
{% endtabs %}



#### Using an industry-specific model

By selecting a model that has been specifically designed for a particular subject, you can achieve more accurate and relevant text classification. An example of such a model is [FinBERT](https://huggingface.co/ProsusAI/finbert), a pre-trained NLP model that has been optimized for analyzing sentiment in financial text. FinBERT was created by training the BERT language model on a large financial corpus, and fine-tuning it to specifically classify financial sentiment. When using FinBERT, the model will provide softmax outputs for three different labels: positive, negative, or neutral.

{% tabs %}
{% tab title="SQL" %}

```postgresql
SELECT pgml.transform(
    task => '{
      "task": "text-classification", 
      "model": "ProsusAI/finbert"
    }'::JSONB,
    inputs => ARRAY[
        'Stocks rallied and the British pound gained.', 
        'Stocks making the biggest moves midday: Nvidia, Palantir and more'
    ]
) AS market_sentiment;
```

{% endtab %}
{% tab title="Result" %}

```json
[
    {"label": "positive", "score": 0.8983612656593323}, 
    {"label": "neutral", "score": 0.8062630891799927}
]
```

{% endtab %}
{% endtabs %}


### Natural Language Inference (NLI)

NLI, or Natural Language Inference, is a type of model that determines the relationship between two texts. The model takes a premise and a hypothesis as inputs and returns a class, which can be one of three types:

| Class | Description |
|-------|-------------|
| Entailment | The hypothesis is true based on the premise. |
| Contradiction | The hypothesis is false based on the premise. |
| Neutral | There is no relationship between the hypothesis and the premise. |


The [GLUE dataset](https://huggingface.co/datasets/nyu-mll/glue) is the benchmark dataset for evaluating NLI models. There are different variants of NLI models, such as Multi-Genre NLI, Question NLI, and Winograd NLI.

If you want to use an NLI model, you can find them on the Hugging Face. When searching for the model, look for models with "mnli" in their name, for example:

{% tabs %}
{% tab title="SQL" %}

```postgresql
SELECT pgml.transform(
    task => '{
      "task": "text-classification", 
      "model": "roberta-large-mnli"
    }'::JSONB,
    inputs => ARRAY[
        'A soccer game with multiple males playing. Some men are playing a sport.'
    ]
) AS nli;
```

{% endtab %}
{% tab title="Result" %}

```json
[
    {"label": "ENTAILMENT", "score": 0.98837411403656}
]
```

{% endtab %}
{% endtabs %}

### Question Natural Language Inference (QNLI)

The QNLI task involves determining whether a given question can be answered by the information in a provided document. If the answer can be found in the document, the label assigned is "entailment". Conversely, if the answer cannot be found in the document, the label assigned is "not entailment".

If you want to use an QNLI model, you can find them on the Hugging Face, by looking for models with "qnli" in their name, for example:

{% tabs %}
{% tab title="SQL" %}

```postgresql
SELECT pgml.transform(
    task => '{
      "task": "text-classification", 
      "model": "cross-encoder/qnli-electra-base"
    }'::JSONB,
    inputs => ARRAY[
        'Where is the capital of France? Paris is the capital of France.'
    ]
) AS qnli;
```

{% endtab %}
{% tab title="Result" %}

```json
[
    {"label": "LABEL_0", "score": 0.9978110194206238}
]
```

{% endtab %}
{% endtabs %}

### Quora Question Pairs (QQP)

The Quora Question Pairs model is designed to evaluate whether two given questions are paraphrases of each other. This model takes the two questions and assigns a binary value as output. `LABEL_0` indicates that the questions are paraphrases of each other and `LABEL_1` indicates that the questions are not paraphrases. The benchmark dataset used for this task is the [Quora Question Pairs](https://huggingface.co/datasets/quora) dataset within the GLUE benchmark, which contains a collection of question pairs and their corresponding labels.

If you want to use an QQP model, you can find them on Hugging Face, by looking for models with `qqp` in their name, for example:

{% tabs %}
{% tab title="SQL" %}

```postgresql
SELECT pgml.transform(
    task => '{
     "task": "text-classification", 
     "model": "textattack/bert-base-uncased-QQP"
    }'::JSONB,
    inputs => ARRAY[
        'Which city is the capital of France? Where is the capital of France?'
    ]
) AS qqp;
```

{% endtab %}
{% tab title="Result" %}

```json
[
    {"label": "LABEL_0", "score": 0.9988721013069152}
]
```

{% endtab %}
{% endtabs %}

### Grammatical correctness

Linguistic Acceptability is a task that involves evaluating the grammatical correctness of a sentence. The model used for this task assigns one of two classes to the sentence, either "acceptable" or "unacceptable". `LABEL_0` indicates acceptable and `LABEL_1` indicates unacceptable. The benchmark dataset used for training and evaluating models for this task is the [Corpus of Linguistic Acceptability (CoLA)](https://huggingface.co/datasets/nyu-mll/glue), which consists of a collection of texts along with their corresponding labels.

If you want to use a grammatical correctness model, you can find them on the Hugging Face. Look for models with "cola" in their name, for example:

{% tabs %}
{% tab title="SQL" %}

```postgresql
SELECT pgml.transform(
    task => '{
      "task": "text-classification", 
      "model": "textattack/distilbert-base-uncased-CoLA"
    }'::JSONB,
    inputs => ARRAY[
        'I will walk to home when I went through the bus.'
    ]
) AS grammatical_correctness;
```

{% endtab %}
{% tab title="Result" %}

```json
[
    {"label": "LABEL_1", "score": 0.9576480388641356}
]
```

{% endtab %}
{% endtabs %}
