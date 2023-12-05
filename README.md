<p align="center">
  <a href="https://postgresml.org/">
    <img src="https://postgresml.org/dashboard/static/images/owl_gradient.svg" width="175" alt="PostgresML">
  </a>
</p>
  
<h2 align="center">
  <a href="https://postgresml.org/">
    <svg version="1.1"
        xmlns="http://www.w3.org/2000/svg"
        xmlns:xlink="http://www.w3.org/1999/xlink"
        width="200" height="50"
    >
        <text font-size="32" x="20" y="32">
            <tspan fill="white" style="mix-blend-mode: difference;">Postgres</tspan><tspan fill="dodgerblue">ML</tspan>
        </text>
    </svg>
  </a>
</h2>

<p align="center">
    Generative AI and Simple ML with 
    <a href="https://www.postgresql.org/" target="_blank">PostgreSQL</a>
</p>

<p align="center">
    <img alt="CI" src="https://github.com/postgresml/postgresml/actions/workflows/ci.yml/badge.svg" />
    <a href="https://discord.gg/DmyJP3qJ7U" target="_blank">
        <img src="https://img.shields.io/discord/1013868243036930099" alt="Join our Discord!" />
    </a>
</p>


# Table of contents
- [Introduction](#introduction)
- [Installation](#installation)
- [Getting started](#getting-started)
- [Natural Language Processing](#nlp-tasks)
    - [Text Classification](#text-classification)
    - [Zero-Shot Classification](#zero-shot-classification)
    - [Token Classification](#token-classification)
    - [Translation](#translation)
    - [Summarization](#summarization)
    - [Question Answering](#question-answering)
    - [Text Generation](#text-generation)
    - [Text-to-Text Generation](#text-to-text-generation)
    - [Fill-Mask](#fill-mask)
- [Vector Database](#vector-database)
<!-- - [Regression](#regression)
- [Classification](#classification) -->

# Introduction
PostgresML is a machine learning extension for PostgreSQL that enables you to perform training and inference on text and tabular data using SQL queries. With PostgresML, you can seamlessly integrate machine learning models into your PostgreSQL database and harness the power of cutting-edge algorithms to process data efficiently.

## Text Data
- Perform natural language processing (NLP) tasks like sentiment analysis, question and answering, translation, summarization and text generation
- Access 1000s of state-of-the-art language models like GPT-2, GPT-J, GPT-Neo from :hugs: HuggingFace model hub
- Fine tune large language models (LLMs) on your own text data for different tasks
- Use your existing PostgreSQL database as a vector database by generating embeddings from text stored in the database.

**Translation**

*SQL query*

```sql
SELECT pgml.transform(
    'translation_en_to_fr',
    inputs => ARRAY[
        'Welcome to the future!',
        'Where have you been all this time?'
    ]
) AS french;
```
*Result*

```sql
                         french                                 
------------------------------------------------------------

[
    {"translation_text": "Bienvenue à l'avenir!"},
    {"translation_text": "Où êtes-vous allé tout ce temps?"}
]
```



**Sentiment Analysis**
*SQL query*

```sql
SELECT pgml.transform(
    task   => 'text-classification',
    inputs => ARRAY[
        'I love how amazingly simple ML has become!', 
        'I hate doing mundane and thankless tasks. ☹️'
    ]
) AS positivity;
```
*Result*
```sql
                    positivity
------------------------------------------------------
[
    {"label": "POSITIVE", "score": 0.9995759129524232}, 
    {"label": "NEGATIVE", "score": 0.9903519749641418}
]
```

## Tabular data
- [47+ classification and regression algorithms](https://postgresml.org/docs/training/algorithm_selection)
- [8 - 40X faster inference than HTTP based model serving](https://postgresml.org/blog/postgresml-is-8x-faster-than-python-http-microservices)
- [Millions of transactions per second](https://postgresml.org/blog/scaling-postgresml-to-one-million-requests-per-second)
- [Horizontal scalability](https://github.com/postgresml/pgcat)


**Training a classification model**

*Training*
```sql
SELECT * FROM pgml.train(
    'Handwritten Digit Image Classifier',
    algorithm => 'xgboost',
    'classification',
    'pgml.digits',
    'target'
);
```

*Inference*
```sql
SELECT pgml.predict(
    'My Classification Project', 
    ARRAY[0.1, 2.0, 5.0]
) AS prediction;
```

# Installation
PostgresML installation consists of three parts: PostgreSQL database, Postgres extension for machine learning and a dashboard app. The extension provides all the machine learning functionality and can be used independently using any SQL IDE. The dashboard app provides an easy to use interface for writing SQL notebooks, performing and tracking ML experiments and ML models.

## Serverless Cloud

If you want to check out the functionality without the hassle of Docker, [sign up for a free PostgresML account](https://postgresml.org/signup). You'll get a free database in seconds, with access to GPUs and state of the art LLMs.

## Docker

```
docker run \
    -it \
    -v postgresml_data:/var/lib/postgresql \
    -p 5433:5432 \
    -p 8000:8000 \
    ghcr.io/postgresml/postgresml:2.7.12 \
    sudo -u postgresml psql -d postgresml
```

For more details, take a look at our [Quick Start with Docker](https://postgresml.org/docs/resources/developer-docs/quick-start-with-docker) documentation.

# Getting Started

## Option 1

- On the cloud console click on the **Dashboard** button to connect to your instance with a SQL notebook, or connect directly with tools listed below.
- On local installation, go to dashboard app at `http://localhost:8000/` to use SQL notebooks.

## Option 2

- Use any of these popular tools to connect to PostgresML and write SQL queries
  - <a href="https://superset.apache.org/" target="_blank">Apache Superset</a>
  - <a href="https://dbeaver.io/" target="_blank">DBeaver</a>
  - <a href="https://www.jetbrains.com/datagrip/" target="_blank">Data Grip</a>
  - <a href="https://eggerapps.at/postico2/" target="_blank">Postico 2</a>
  - <a href="https://popsql.com/" target="_blank">Popsql</a>
  - <a href="https://www.tableau.com/" target="_blank">Tableau</a>
  - <a href="https://powerbi.microsoft.com/en-us/" target="_blank">PowerBI</a>
  - <a href="https://jupyter.org/" target="_blank">Jupyter</a>
  - <a href="https://code.visualstudio.com/" target="_blank">VSCode</a>

## Option 3

- Connect directly to the database with your favorite programming language
  - C++: <a href="https://www.tutorialspoint.com/postgresql/postgresql_c_cpp.htm" target="_blank">libpqxx</a>
  - C#: <a href="https://github.com/npgsql/npgsql" target="_blank">Npgsql</a>,<a href="https://github.com/DapperLib/Dapper" target="_blank">Dapper</a>, or <a href="https://github.com/dotnet/efcore" target="_blank">Entity Framework Core</a>
  - Elixir: <a href="https://github.com/elixir-ecto/ecto" target="_blank">ecto</a> or <a href="https://github.com/elixir-ecto/postgrex" target="_blank">Postgrex</a>
  - Go: <a href="https://github.com/jackc/pgx" target="_blank">pgx</a>, <a href="https://github.com/go-pg/pg" target="_blank">pg</a> or <a href="https://github.com/uptrace/bun" target="_blank">Bun</a>
  - Haskell: <a href="https://hackage.haskell.org/package/postgresql-simple" target="_blank">postgresql-simple</a>
  - Java & Scala: <a href="https://jdbc.postgresql.org/" target="_blank">JDBC</a> or <a href="https://github.com/slick/slick" target="_blank">Slick</a> 
  - Julia: <a href="https://github.com/iamed2/LibPQ.jl" target="_blank">LibPQ.jl</a> 
  - Lua: <a href="https://github.com/leafo/pgmoon" target="_blank">pgmoon</a>
  - Node: <a href="https://github.com/brianc/node-postgres" target="_blank">node-postgres</a>, <a href="https://github.com/vitaly-t/pg-promise" target="_blank">pg-promise</a>, or <a href="https://sequelize.org/" target="_blank">Sequelize</a>
  - Perl: <a href="https://github.com/bucardo/dbdpg" target="_blank">DBD::Pg</a>
  - PHP: <a href="https://laravel.com/" target="_blank">Laravel</a> or <a href="https://www.php.net/manual/en/book.pgsql.php" target="_blank">PHP</a> 
  - Python: <a href="https://github.com/psycopg/psycopg2/" target="_blank">psycopg2</a>, <a href="https://www.sqlalchemy.org/" target="_blank">SQLAlchemy</a>, or <a href="https://www.djangoproject.com/" target="_blank">Django</a>
  - R: <a href="https://github.com/r-dbi/DBI" target="_blank">DBI</a> or <a href="https://github.com/ankane/dbx" target="_blank">dbx</a>
  - Ruby: <a href="https://github.com/ged/ruby-pg" target="_blank">pg</a> or <a href="https://rubyonrails.org/" target="_blank">Rails</a>
  - Rust: <a href="https://crates.io/crates/postgres" target="_blank">postgres</a>, <a href="https://github.com/launchbadge/sqlx" target="_blank">SQLx</a> or <a href="https://github.com/diesel-rs/diesel" target="_blank">Diesel</a>
  - Swift: <a href="https://github.com/vapor/postgres-nio" target="_blank">PostgresNIO</a> or <a href="https://github.com/codewinsdotcom/PostgresClientKit" target="_blank">PostgresClientKit</a> 
  - ... open a PR to add your favorite language and connector.

# NLP Tasks

PostgresML integrates 🤗 Hugging Face Transformers to bring state-of-the-art NLP models into the data layer. There are tens of thousands of pre-trained models with pipelines to turn raw text in your database into useful results. Many state of the art deep learning architectures have been published and made available from Hugging Face <a href= "https://huggingface.co/models" target="_blank">model hub</a>.

You can call different NLP tasks and customize using them using the following SQL query.

```sql
SELECT pgml.transform(
    task   => TEXT OR JSONB,     -- Pipeline initializer arguments
    inputs => TEXT[] OR BYTEA[], -- inputs for inference
    args   => JSONB              -- (optional) arguments to the pipeline.
)
```
## Text Classification

Text classification involves assigning a label or category to a given text. Common use cases include sentiment analysis, natural language inference, and the assessment of grammatical correctness.

![text classification](pgml-cms/docs/images/text-classification.png)

### Sentiment Analysis
Sentiment analysis is a type of natural language processing technique that involves analyzing a piece of text to determine the sentiment or emotion expressed within it. It can be used to classify a text as positive, negative, or neutral, and has a wide range of applications in fields such as marketing, customer service, and political analysis.

*Basic usage*
```sql
SELECT pgml.transform(
    task   => 'text-classification',
    inputs => ARRAY[
        'I love how amazingly simple ML has become!', 
        'I hate doing mundane and thankless tasks. ☹️'
    ]
) AS positivity;
```
*Result*
```json
[
    {"label": "POSITIVE", "score": 0.9995759129524232}, 
    {"label": "NEGATIVE", "score": 0.9903519749641418}
]
```
The default <a href="https://huggingface.co/distilbert-base-uncased-finetuned-sst-2-english" target="_blank">model</a> used for text classification is a fine-tuned version of DistilBERT-base-uncased that has been specifically optimized for the Stanford Sentiment Treebank dataset (sst2).


*Using specific model*

To use one of the over 19,000 models available on Hugging Face, include the name of the desired model and `text-classification` task as a JSONB object in the SQL query. For example, if you want to use a RoBERTa <a href="https://huggingface.co/models?pipeline_tag=text-classification" target="_blank">model</a> trained on around 40,000 English tweets and that has POS (positive), NEG (negative), and NEU (neutral) labels for its classes, include this information in the JSONB object when making your query.

```sql
SELECT pgml.transform(
    inputs => ARRAY[
        'I love how amazingly simple ML has become!', 
        'I hate doing mundane and thankless tasks. ☹️'
    ],
    task  => '{"task": "text-classification", 
              "model": "finiteautomata/bertweet-base-sentiment-analysis"
             }'::JSONB
) AS positivity;
```
*Result*
```json
[
    {"label": "POS", "score": 0.992932200431826}, 
    {"label": "NEG", "score": 0.975599765777588}
]
```

*Using industry specific model*

By selecting a model that has been specifically designed for a particular industry, you can achieve more accurate and relevant text classification. An example of such a model is <a href="https://huggingface.co/ProsusAI/finbert" target="_blank">FinBERT</a>, a pre-trained NLP model that has been optimized for analyzing sentiment in financial text. FinBERT was created by training the BERT language model on a large financial corpus, and fine-tuning it to specifically classify financial sentiment. When using FinBERT, the model will provide softmax outputs for three different labels: positive, negative, or neutral.

```sql
SELECT pgml.transform(
    inputs => ARRAY[
        'Stocks rallied and the British pound gained.', 
        'Stocks making the biggest moves midday: Nvidia, Palantir and more'
    ],
    task => '{"task": "text-classification", 
              "model": "ProsusAI/finbert"
             }'::JSONB
) AS market_sentiment;
```

*Result*
```json
[
    {"label": "positive", "score": 0.8983612656593323}, 
    {"label": "neutral", "score": 0.8062630891799927}
]
```

### Natural Language Inference (NLI)
NLI, or Natural Language Inference, is a type of model that determines the relationship between two texts. The model takes a premise and a hypothesis as inputs and returns a class, which can be one of three types:
- Entailment: This means that the hypothesis is true based on the premise.
- Contradiction: This means that the hypothesis is false based on the premise.
- Neutral: This means that there is no relationship between the hypothesis and the premise.

The GLUE dataset is the benchmark dataset for evaluating NLI models. There are different variants of NLI models, such as Multi-Genre NLI, Question NLI, and Winograd NLI.

If you want to use an NLI model, you can find them on the :hugs: Hugging Face model hub. Look for models with "mnli".

```sql
SELECT pgml.transform(
    inputs => ARRAY[
        'A soccer game with multiple males playing. Some men are playing a sport.'
    ],
    task => '{"task": "text-classification", 
              "model": "roberta-large-mnli"
             }'::JSONB
) AS nli;
```
*Result*
```json
[
    {"label": "ENTAILMENT", "score": 0.98837411403656}
]
```
### Question Natural Language Inference (QNLI)
The QNLI task involves determining whether a given question can be answered by the information in a provided document. If the answer can be found in the document, the label assigned is "entailment". Conversely, if the answer cannot be found in the document, the label assigned is "not entailment".

If you want to use an QNLI model, you can find them on the :hugs: Hugging Face model hub. Look for models with "qnli".

```sql
SELECT pgml.transform(
    inputs => ARRAY[
        'Where is the capital of France?, Paris is the capital of France.'
    ],
    task => '{"task": "text-classification", 
              "model": "cross-encoder/qnli-electra-base"
             }'::JSONB
) AS qnli;
```

*Result*
```json
[
    {"label": "LABEL_0", "score": 0.9978110194206238}
]
```

### Quora Question Pairs (QQP)
The Quora Question Pairs model is designed to evaluate whether two given questions are paraphrases of each other. This model takes the two questions and assigns a binary value as output. LABEL_0 indicates that the questions are paraphrases of each other and LABEL_1 indicates that the questions are not paraphrases. The benchmark dataset used for this task is the Quora Question Pairs dataset within the GLUE benchmark, which contains a collection of question pairs and their corresponding labels.

If you want to use an QQP model, you can find them on the :hugs: Hugging Face model hub. Look for models with `qqp`.

```sql
SELECT pgml.transform(
    inputs => ARRAY[
        'Which city is the capital of France?, Where is the capital of France?'
    ],
    task => '{"task": "text-classification", 
              "model": "textattack/bert-base-uncased-QQP"
             }'::JSONB
) AS qqp;
```

*Result*
```json
[
    {"label": "LABEL_0", "score": 0.9988721013069152}
]
```

### Grammatical Correctness
Linguistic Acceptability is a task that involves evaluating the grammatical correctness of a sentence. The model used for this task assigns one of two classes to the sentence, either "acceptable" or "unacceptable". LABEL_0 indicates acceptable and LABEL_1 indicates unacceptable. The benchmark dataset used for training and evaluating models for this task is the Corpus of Linguistic Acceptability (CoLA), which consists of a collection of texts along with their corresponding labels. 

If you want to use a grammatical correctness model, you can find them on the :hugs: Hugging Face model hub. Look for models with `cola`.

```sql
SELECT pgml.transform(
    inputs => ARRAY[
        'I will walk to home when I went through the bus.'
    ],
    task => '{"task": "text-classification", 
              "model": "textattack/distilbert-base-uncased-CoLA"
             }'::JSONB
) AS grammatical_correctness;
```
*Result*
```json
[
    {"label": "LABEL_1", "score": 0.9576480388641356}
]
```

## Zero-Shot Classification
Zero Shot Classification is a task where the model predicts a class that it hasn't seen during the training phase. This task leverages a pre-trained language model and is a type of transfer learning. Transfer learning involves using a model that was initially trained for one task in a different application. Zero Shot Classification is especially helpful when there is a scarcity of labeled data available for the specific task at hand.

![zero-shot classification](pgml-cms/docs/images/zero-shot-classification.png)

In the example provided below, we will demonstrate how to classify a given sentence into a class that the model has not encountered before. To achieve this, we make use of `args` in the SQL query, which allows us to provide `candidate_labels`. You can customize these labels to suit the context of your task. We will use `facebook/bart-large-mnli` model.

Look for models with `mnli` to use a zero-shot classification model on the :hugs: Hugging Face model hub.

```sql
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
*Result*

```json
[
    {
        "labels": ["urgent", "phone", "computer", "not urgent", "tablet"], 
        "scores": [0.503635, 0.47879, 0.012600, 0.002655, 0.002308], 
        "sequence": "I have a problem with my iphone that needs to be resolved asap!!"
    }
]
```
## Token Classification
Token classification is a task in natural language understanding, where labels are assigned to certain tokens in a text. Some popular subtasks of token classification include Named Entity Recognition (NER) and Part-of-Speech (PoS) tagging. NER models can be trained to identify specific entities in a text, such as individuals, places, and dates. PoS tagging, on the other hand, is used to identify the different parts of speech in a text, such as nouns, verbs, and punctuation marks.

![token classification](pgml-cms/docs/images/token-classification.png)

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
*Result*
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
*Result*
```json
[[
    {"end": 1,  "word": "i",         "index": 1, "score": 0.999, "start": 0,  "entity": "PRON"},
    {"end": 6,  "word": "live",      "index": 2, "score": 0.998, "start": 2,  "entity": "VERB"},
    {"end": 9,  "word": "in",        "index": 3, "score": 0.999, "start": 7,  "entity": "ADP"},
    {"end": 19, "word": "amsterdam", "index": 4, "score": 0.998, "start": 10, "entity": "PROPN"}, 
    {"end": 20, "word": ".",         "index": 5, "score": 0.999, "start": 19, "entity": "PUNCT"}
]]
```
## Translation
Translation is the task of converting text written in one language into another language.

![translation](pgml-cms/docs/images/translation.png)

You have the option to select from over 2000 models available on the Hugging Face <a href="https://huggingface.co/models?pipeline_tag=translation" target="_blank">hub</a> for translation.

```sql
select pgml.transform(
    inputs => array[
            	'How are you?'
    ],
	task => '{"task": "translation", 
              "model": "Helsinki-NLP/opus-mt-en-fr"
    }'::JSONB	
);
```
*Result*
```json
[
    {"translation_text": "Comment allez-vous ?"}
]
```
## Summarization
Summarization involves creating a condensed version of a document that includes the important information while reducing its length. Different models can be used for this task, with some models extracting the most relevant text from the original document, while other models generate completely new text that captures the essence of the original content.

![summarization](pgml-cms/docs/images/summarization.png)

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
*Result*
```json
[
    {"summary_text": " Paris is the capital and most populous city of France, with an estimated population of 2,175,601 residents as of 2018 . The city is the centre and seat of government of the region and province of Île-de-France, or Paris Region . Paris Region has an estimated 18 percent of the population of France as of 2017 ."}
    ]
```
You can control the length of summary_text by passing `min_length` and `max_length` as arguments to the SQL query.

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
## Question Answering
Question Answering models are designed to retrieve the answer to a question from a given text, which can be particularly useful for searching for information within a document. It's worth noting that some question answering models are capable of generating answers even without any contextual information.

![question answering](pgml-cms/docs/images/question-answering.png)

```sql
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
*Result*

```json
{
    "end"   :  39, 
    "score" :  0.9538117051124572, 
    "start" :  31, 
    "answer": "İstanbul"
}
```
<!-- ## Table Question Answering
![table question answering](pgml-cms/docs/images/table-question-answering.png) -->

## Text Generation
Text generation is the task of producing new text, such as filling in incomplete sentences or paraphrasing existing text. It has various use cases, including code generation and story generation. Completion generation models can predict the next word in a text sequence, while text-to-text generation models are trained to learn the mapping between pairs of texts, such as translating between languages. Popular models for text generation include GPT-based models, T5, T0, and BART. These models can be trained to accomplish a wide range of tasks, including text classification, summarization, and translation.

![text generation](pgml-cms/docs/images/text-generation.png)

```sql
SELECT pgml.transform(
    task => 'text-generation',
    inputs => ARRAY[
        'Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone'
    ]
) AS answer;
```
*Result*

```json
[
    [
        {"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone, and eight for the Dragon-lords in their halls of blood.\n\nEach of the guild-building systems is one-man"}
    ]
]
```

To use a specific model from :hugs: model hub, pass the model name along with task name in task.

```sql
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
*Result*
```json
[
    [{"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone.\n\nThis place has a deep connection to the lore of ancient Elven civilization. It is home to the most ancient of artifacts,"}]
]
```
To make the generated text longer, you can include the argument `max_length` and specify the desired maximum length of the text.

```sql
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
*Result*
```json
[
    [{"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone, Three for the Dwarfs and the Elves, One for the Gnomes of the Mines, and Two for the Elves of Dross.\"\n\nHobbits: The Fellowship is the first book of J.R.R. Tolkien's story-cycle, and began with his second novel - The Two Towers - and ends in The Lord of the Rings.\n\n\nIt is a non-fiction novel, so there is no copyright claim on some parts of the story but the actual text of the book is copyrighted by author J.R.R. Tolkien.\n\n\nThe book has been classified into two types: fantasy novels and children's books\n\nHobbits: The Fellowship is the first book of J.R.R. Tolkien's story-cycle, and began with his second novel - The Two Towers - and ends in The Lord of the Rings.It"}]
]
```
If you want the model to generate more than one output, you can specify the number of desired output sequences by including the argument `num_return_sequences` in the arguments.

```sql
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
*Result*
```json
[
    [
        {"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone, and Thirteen for the human-men in their hall of fire.\n\nAll of us, our families, and our people"}, 
        {"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone, and the tenth for a King! As each of these has its own special story, so I have written them into the game."}, 
        {"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone… What's left in the end is your heart's desire after all!\n\nHans: (Trying to be brave)"}
    ]
]
```
Text generation typically utilizes a greedy search algorithm that selects the word with the highest probability as the next word in the sequence. However, an alternative method called beam search can be used, which aims to minimize the possibility of overlooking hidden high probability word combinations. Beam search achieves this by retaining the num_beams most likely hypotheses at each step and ultimately selecting the hypothesis with the highest overall probability. We set `num_beams > 1` and `early_stopping=True` so that generation is finished when all beam hypotheses reached the EOS token.

```sql
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

*Result*
```json
[[
    {"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone, Nine for the Dwarves in their caverns of ice, Ten for the Elves in their caverns of fire, Eleven for the"}
]]
```
Sampling methods involve selecting the next word or sequence of words at random from the set of possible candidates, weighted by their probabilities according to the language model. This can result in more diverse and creative text, as well as avoiding repetitive patterns. In its most basic form, sampling means randomly picking the next word $w_t$ according to its conditional probability distribution: 
$$ w_t \approx P(w_t|w_{1:t-1})$$


However, the randomness of the sampling method can also result in less coherent or inconsistent text, depending on the quality of the model and the chosen sampling parameters such as temperature, top-k, or top-p. Therefore, choosing an appropriate sampling method and parameters is crucial for achieving the desired balance between creativity and coherence in generated text.

You can pass `do_sample = True` in the arguments to use sampling methods. It is recommended to alter `temperature` or `top_p` but not both.

*Temperature*
```sql
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
*Result*
```json
[[{"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone, and Thirteen for the Giants and Men of S.A.\n\nThe First Seven-Year Time-Traveling Trilogy is"}]]
```
*Top p*

```sql
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
*Result*
```json
[[{"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone, Four for the Elves of the forests and fields, and Three for the Dwarfs and their warriors.\" ―Lord Rohan [src"}]]
```
## Text-to-Text Generation
Text-to-text generation methods, such as T5, are neural network architectures designed to perform various natural language processing tasks, including summarization, translation, and question answering. T5 is a transformer-based architecture pre-trained on a large corpus of text data using denoising autoencoding. This pre-training process enables the model to learn general language patterns and relationships between different tasks, which can be fine-tuned for specific downstream tasks. During fine-tuning, the T5 model is trained on a task-specific dataset to learn how to perform the specific task.
![text-to-text](pgml-cms/docs/images/text-to-text-generation.png)

*Translation*
```sql
SELECT pgml.transform(
    task => '{
        "task" : "text2text-generation"
    }'::JSONB,
    inputs => ARRAY[
        'translate from English to French: I''m very happy'
    ]
) AS answer;
```

*Result*
```json
[
    {"generated_text": "Je suis très heureux"}
]
```
Similar to other tasks, we can specify a model for text-to-text generation.

```sql
SELECT pgml.transform(
    task => '{
        "task" : "text2text-generation",
        "model" : "bigscience/T0"
    }'::JSONB,
    inputs => ARRAY[
        'Is the word ''table'' used in the same meaning in the two previous sentences? Sentence A: you can leave the books on the table over there. Sentence B: the tables in this book are very hard to read.'

    ]
) AS answer;

```
## Fill-Mask
Fill-mask refers to a task where certain words in a sentence are hidden or "masked", and the objective is to predict what words should fill in those masked positions. Such models are valuable when we want to gain statistical insights about the language used to train the model.
![fill mask](pgml-cms/docs/images/fill-mask.png)

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
*Result*
```json
[
    {"score": 0.679, "token": 812,   "sequence": "Paris is the capital of France.",    "token_str": " capital"}, 
    {"score": 0.051, "token": 32357, "sequence": "Paris is the birthplace of France.", "token_str": " birthplace"}, 
    {"score": 0.038, "token": 1144,  "sequence": "Paris is the heart of France.",      "token_str": " heart"}, 
    {"score": 0.024, "token": 29778, "sequence": "Paris is the envy of France.",       "token_str": " envy"}, 
    {"score": 0.022, "token": 1867,  "sequence": "Paris is the Capital of France.",    "token_str": " Capital"}]
```

# Vector Database
A vector database is a type of database that stores and manages vectors, which are mathematical representations of data points in a multi-dimensional space. Vectors can be used to represent a wide range of data types, including images, text, audio, and numerical data. It is designed to support efficient searching and retrieval of vectors, using methods such as nearest neighbor search, clustering, and indexing. These methods enable applications to find vectors that are similar to a given query vector, which is useful for tasks such as image search, recommendation systems, and natural language processing.

PostgresML enhances your existing PostgreSQL database to be used as a vector database by generating embeddings from text stored in your tables. To generate embeddings, you can use the `pgml.embed` function, which takes a transformer name and a text value as input. This function automatically downloads and caches the transformer for future reuse, which saves time and resources.

Using a vector database involves three key steps: creating embeddings, indexing your embeddings using different algorithms, and querying the index using embeddings for your queries. Let's break down each step in more detail.

## Step 1: Creating embeddings using transformers
To create embeddings for your data, you first need to choose a transformer that can generate embeddings from your input data. Some popular transformer options include BERT, GPT-2, and T5. Once you've selected a transformer, you can use it to generate embeddings for your data.

In the following section, we will demonstrate how to use PostgresML to generate embeddings for a dataset of tweets commonly used in sentiment analysis. To generate the embeddings, we will use the `pgml.embed` function, which will generate an embedding for each tweet in the dataset. These embeddings will then be inserted into a table called tweet_embeddings.
```sql
SELECT pgml.load_dataset('tweet_eval', 'sentiment');

SELECT * 
FROM pgml.tweet_eval
LIMIT 10;

CREATE TABLE tweet_embeddings AS
SELECT text, pgml.embed('distilbert-base-uncased', text) AS embedding 
FROM pgml.tweet_eval;

SELECT * from tweet_embeddings limit 2;
```

*Result*

|text|embedding|
|----|---------|
|"QT @user In the original draft of the 7th book, Remus Lupin survived the Battle of Hogwarts. #HappyBirthdayRemusLupin"|{-0.1567948312,-0.3149209619,0.2163394839,..}|
|"Ben Smith / Smith (concussion) remains out of the lineup Thursday, Curtis #NHL #SJ"|{-0.0701668188,-0.012231146,0.1304316372,.. }|


## Step 2: Indexing your embeddings using different algorithms
After you've created embeddings for your data, you need to index them using one or more indexing algorithms. There are several different types of indexing algorithms available, including B-trees, k-nearest neighbors (KNN), and approximate nearest neighbors (ANN). The specific type of indexing algorithm you choose will depend on your use case and performance requirements. For example, B-trees are a good choice for range queries, while KNN and ANN algorithms are more efficient for similarity searches.

On small datasets (<100k rows), a linear search that compares every row to the query will give sub-second results, which may be fast enough for your use case. For larger datasets, you may want to consider various indexing strategies offered by additional extensions.

- <a href="https://www.postgresql.org/docs/current/cube.html" target="_blank">Cube</a> is a built-in extension that provides a fast indexing strategy for finding similar vectors. By default it has an arbitrary limit of 100 dimensions, unless Postgres is compiled with a larger size.
- <a href="https://github.com/pgvector/pgvector" target="_blank">PgVector</a> supports embeddings up to 2000 dimensions out of the box, and provides a fast indexing strategy for finding similar vectors.

When indexing your embeddings, it's important to consider the trade-offs between accuracy and speed. Exact indexing algorithms like B-trees can provide precise results, but may not be as fast as approximate indexing algorithms like KNN and ANN. Similarly, some indexing algorithms may require more memory or disk space than others.

In the following, we are creating an index on the tweet_embeddings table using the ivfflat algorithm for indexing. The ivfflat algorithm is a type of hybrid index that combines an Inverted File (IVF) index with a Flat (FLAT) index.

The index is being created on the embedding column in the tweet_embeddings table, which contains vector embeddings generated from the original tweet dataset. The `vector_cosine_ops` argument specifies the indexing operation to use for the embeddings. In this case, it's using the `cosine similarity` operation, which is a common method for measuring similarity between vectors.

By creating an index on the embedding column, the database can quickly search for and retrieve records that are similar to a given query vector. This can be useful for a variety of machine learning applications, such as similarity search or recommendation systems.

```sql
CREATE INDEX ON tweet_embeddings USING ivfflat (embedding vector_cosine_ops);
```
## Step 3: Querying the index using embeddings for your queries
Once your embeddings have been indexed, you can use them to perform queries against your database. To do this, you'll need to provide a query embedding that represents the query you want to perform. The index will then return the closest matching embeddings from your database, based on the similarity between the query embedding and the stored embeddings.

```sql
WITH query AS (
    SELECT pgml.embed('distilbert-base-uncased', 'Star Wars christmas special is on Disney')::vector AS embedding
)
SELECT * FROM items, query ORDER BY items.embedding <-> query.embedding LIMIT 5;
```

*Result*
|text|
|----|
|Happy Friday with Batman animated Series 90S forever!|
|"Fri Oct 17, Sonic Highways is on HBO tonight, Also new episode of  Girl Meets World on Disney"|
|tfw the 2nd The Hunger Games movie is on Amazon Prime but not the 1st one I didn't watch|
|5 RT's if you want the next episode of twilight princess tomorrow|
|Jurassic Park is BACK! New Trailer for the 4th Movie, Jurassic World -|


<!-- ## Sentence Similarity
Sentence Similarity involves determining the degree of similarity between two texts. To accomplish this, Sentence similarity models convert the input texts into vectors (embeddings) that encapsulate semantic information, and then measure the proximity (or similarity) between the vectors. This task is especially beneficial for tasks such as information retrieval and clustering/grouping.
![sentence similarity](pgml-cms/docs/images/sentence-similarity.png)

<!-- ## Conversational -->
<!-- # Regression
# Classification -->




