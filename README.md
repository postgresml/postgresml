<div align="center">
   <picture>
     <source media="(prefers-color-scheme: dark)" srcset="https://github.com/user-attachments/assets/5d5510da-6014-4cf3-849f-566050e053da">
     <source media="(prefers-color-scheme: light)" srcset="https://github.com/user-attachments/assets/aea1c38a-15bf-4270-8365-3d5e6311f5fc">
     <img alt="Logo" src="" width="520">
   </picture>
</div>

<p align="center">
   <p align="center"><b>Generative AI and Simple ML with PostgreSQL.</b></p>
</p>

<p align="center">
| <a href="https://postgresml.org/docs/"><b>Documentation</b></a> | <a href="https://postgresml.org/blog"><b>Blog</b></a> | <a href="https://discord.gg/DmyJP3qJ7U"><b>Discord</b></a> |
</p>

---
PostgresML is a complete ML/AI platform built inside PostgreSQL. Our operating principle is:

Move models to the database, rather than constantly moving data to the models.

Data for ML & AI systems is inherently larger and more dynamic than the models. It's more efficient, manageable and reliable to move models to the database, rather than continuously moving data to the models.


<b> Table of contents </b>
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
- [LLM Fine-tuning](#llm-fine-tuning)
    - [Text Classification - 2 classes](#text-classification-2-classes)
    - [Text Classification - 9 classes](#text-classification-9-classes)
    - [Conversation](#conversation)
<!-- - [Regression](#regression)
- [Classification](#classification) -->

## Text Data
- Perform natural language processing (NLP) tasks like sentiment analysis, question and answering, translation, summarization and text generation
- Access 1000s of state-of-the-art language models like GPT-2, GPT-J, GPT-Neo from :hugs: HuggingFace model hub
- Fine tune large language models (LLMs) on your own text data for different tasks
- Use your existing PostgreSQL database as a vector database by generating embeddings from text stored in the database.

**Translation**

*SQL query*

```postgresql
SELECT pgml.transform(
    'translation_en_to_fr',
    inputs => ARRAY[
        'Welcome to the future!',
        'Where have you been all this time?'
    ]
) AS french;
```
*Result*

```postgresql
                         french                                 
------------------------------------------------------------

[
    {"translation_text": "Bienvenue à l'avenir!"},
    {"translation_text": "Où êtes-vous allé tout ce temps?"}
]
```

**Sentiment Analysis**
*SQL query*

```postgresql
SELECT pgml.transform(
    task   => 'text-classification',
    inputs => ARRAY[
        'I love how amazingly simple ML has become!', 
        'I hate doing mundane and thankless tasks. ☹️'
    ]
) AS positivity;
```
*Result*
```postgresql
                    positivity
------------------------------------------------------
[
    {"label": "POSITIVE", "score": 0.9995759129524232}, 
    {"label": "NEGATIVE", "score": 0.9903519749641418}
]
```

## Tabular data
- [47+ classification and regression algorithms](https://postgresml.org/docs/open-source/pgml/api/pgml.train)
- [8 - 40X faster inference than HTTP based model serving](https://postgresml.org/blog/postgresml-is-8x-faster-than-python-http-microservices)
- [Millions of transactions per second](https://postgresml.org/blog/scaling-postgresml-to-one-million-requests-per-second)
- [Horizontal scalability](https://postgresml.org/docs/open-source/pgcat/)

**Training a classification model**

*Training*
```postgresql
SELECT * FROM pgml.train(
    'Handwritten Digit Image Classifier',
    algorithm => 'xgboost',
    'classification',
    'pgml.digits',
    'target'
);
```

*Inference*
```postgresql
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

For more details, take a look at our [Quick Start with Docker](https://postgresml.org/docs/open-source/pgml/developers/quick-start-with-docker) documentation.

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

```postgresql
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
```postgresql
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

```postgresql
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

```postgresql
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

```postgresql
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

```postgresql
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

```postgresql
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

```postgresql
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

```postgresql
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

```postgresql
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
```postgresql
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

```postgresql
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

```postgresql
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

```postgresql
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

```postgresql
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
*Result*
```json
[
    [{"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone.\n\nThis place has a deep connection to the lore of ancient Elven civilization. It is home to the most ancient of artifacts,"}]
]
```
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
*Result*
```json
[
    [{"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone, Three for the Dwarfs and the Elves, One for the Gnomes of the Mines, and Two for the Elves of Dross.\"\n\nHobbits: The Fellowship is the first book of J.R.R. Tolkien's story-cycle, and began with his second novel - The Two Towers - and ends in The Lord of the Rings.\n\n\nIt is a non-fiction novel, so there is no copyright claim on some parts of the story but the actual text of the book is copyrighted by author J.R.R. Tolkien.\n\n\nThe book has been classified into two types: fantasy novels and children's books\n\nHobbits: The Fellowship is the first book of J.R.R. Tolkien's story-cycle, and began with his second novel - The Two Towers - and ends in The Lord of the Rings.It"}]
]
```
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
*Result*
```json
[[{"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone, and Thirteen for the Giants and Men of S.A.\n\nThe First Seven-Year Time-Traveling Trilogy is"}]]
```
*Top p*

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
*Result*
```json
[[{"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone, Four for the Elves of the forests and fields, and Three for the Dwarfs and their warriors.\" ―Lord Rohan [src"}]]
```
## Text-to-Text Generation
Text-to-text generation methods, such as T5, are neural network architectures designed to perform various natural language processing tasks, including summarization, translation, and question answering. T5 is a transformer-based architecture pre-trained on a large corpus of text data using denoising autoencoding. This pre-training process enables the model to learn general language patterns and relationships between different tasks, which can be fine-tuned for specific downstream tasks. During fine-tuning, the T5 model is trained on a task-specific dataset to learn how to perform the specific task.
![text-to-text](pgml-cms/docs/images/text-to-text-generation.png)

*Translation*
```postgresql
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

```postgresql
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

```postgresql
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
```postgresql
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

```postgresql
CREATE INDEX ON tweet_embeddings USING ivfflat (embedding vector_cosine_ops);
```
## Step 3: Querying the index using embeddings for your queries
Once your embeddings have been indexed, you can use them to perform queries against your database. To do this, you'll need to provide a query embedding that represents the query you want to perform. The index will then return the closest matching embeddings from your database, based on the similarity between the query embedding and the stored embeddings.

```postgresql
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

# LLM Fine-tuning 

In this section, we will provide a step-by-step walkthrough for fine-tuning a Language Model (LLM) for differnt tasks.

## Prerequisites

1. Ensure you have the PostgresML extension installed and configured in your PostgreSQL database. You can find installation instructions for PostgresML in the official documentation.

2. Obtain a Hugging Face API token to push the fine-tuned model to the Hugging Face Model Hub. Follow the instructions on the [Hugging Face website](https://huggingface.co/settings/tokens) to get your API token.

## Text Classification 2 Classes

### 1. Loading the Dataset

To begin, create a table to store your dataset. In this example, we use the 'imdb' dataset from Hugging Face. IMDB dataset contains three splits: train (25K rows), test (25K rows) and unsupervised (50K rows). In train and test splits, negative class has label 0 and positive class label 1. All rows in unsupervised split has a label of -1. 
```postgresql
SELECT pgml.load_dataset('imdb');
```

### 2. Prepare dataset for fine-tuning

We will create a view of the dataset by performing the following operations:

- Add a new text column named "class" that has positive and negative classes. 
- Shuffled view of the dataset to ensure randomness in the distribution of data.
- Remove all the unsupervised splits that have label = -1.

```postgresql
CREATE VIEW pgml.imdb_shuffled_view AS
SELECT
    label,
    CASE WHEN label = 0 THEN 'negative'
         WHEN label = 1 THEN 'positive'
         ELSE 'neutral'
    END AS class,
    text
FROM pgml.imdb
WHERE label != -1
ORDER BY RANDOM();
```

### 3 Exploratory Data Analysis (EDA) on Shuffled Data

Before splitting the data into training and test sets, it's essential to perform exploratory data analysis (EDA) to understand the distribution of labels and other characteristics of the dataset. In this section, we'll use the `pgml.imdb_shuffled_view` to explore the shuffled data.

#### 3.1 Distribution of Labels

To analyze the distribution of labels in the shuffled dataset, you can use the following SQL query:

```postgresql
-- Count the occurrences of each label in the shuffled dataset
pgml=# SELECT
    class,
    COUNT(*) AS label_count
FROM pgml.imdb_shuffled_view
GROUP BY class
ORDER BY class;

  class   | label_count
----------+-------------
 negative |       25000
 positive |       25000
(2 rows)
```

This query provides insights into the distribution of labels, helping you understand the balance or imbalance of classes in your dataset.

#### 3.2 Sample Records
To get a glimpse of the data, you can retrieve a sample of records from the shuffled dataset:

```postgresql
-- Retrieve a sample of records from the shuffled dataset
pgml=# SELECT LEFT(text,100) AS text, class
FROM pgml.imdb_shuffled_view
LIMIT 5;
                                                 text                                                 |  class
------------------------------------------------------------------------------------------------------+----------
 This is a VERY entertaining movie. A few of the reviews that I have read on this forum have been wri | positive
 This is one of those movies where I wish I had just stayed in the bar.<br /><br />The film is quite  | negative
 Barbershop 2: Back in Business wasn't as good as it's original but was just as funny. The movie itse | negative
 Umberto Lenzi hits new lows with this recycled trash. Janet Agren plays a lady who is looking for he | negative
 I saw this movie last night at the Phila. Film festival. It was an interesting and funny movie that  | positive
(5 rows)

Time: 101.985 ms
```

This query allows you to inspect a few records to understand the structure and content of the shuffled data.

#### 3.3 Additional Exploratory Analysis
Feel free to explore other aspects of the data, such as the distribution of text lengths, word frequencies, or any other features relevant to your analysis. Performing EDA is crucial for gaining insights into your dataset and making informed decisions during subsequent steps of the workflow.

### 4. Splitting Data into Training and Test Sets

Create views for training and test data by splitting the shuffled dataset. In this example, 80% is allocated for training, and 20% for testing. We will use `pgml.imdb_test_view` in [section 6](#6-inference-using-fine-tuned-model) for batch predictions using the finetuned model.

```postgresql
-- Create a view for training data
CREATE VIEW pgml.imdb_train_view AS
SELECT *
FROM pgml.imdb_shuffled_view
LIMIT (SELECT COUNT(*) * 0.8 FROM pgml.imdb_shuffled_view);

-- Create a view for test data
CREATE VIEW pgml.imdb_test_view AS
SELECT *
FROM pgml.imdb_shuffled_view
OFFSET (SELECT COUNT(*) * 0.8 FROM pgml.imdb_shuffled_view);
```

### 5. Fine-Tuning the Language Model

Now, fine-tune the Language Model for text classification using the created training view. In the following sections, you will see a detailed explanation of different parameters used during fine-tuning. Fine-tuned model is pushed to your public Hugging Face Hub periodically. A new repository will be created under your username using your project name (`imdb_review_sentiment` in this case). You can also choose to push the model to a private repository by setting `hub_private_repo: true` in training arguments.

```postgresql
SELECT pgml.tune(
    'imdb_review_sentiment',
    task => 'text-classification',
    relation_name => 'pgml.imdb_train_view',
    model_name => 'distilbert-base-uncased',
    test_size => 0.2,
    test_sampling => 'last',
    hyperparams => '{
        "training_args" : {
            "learning_rate": 2e-5,
            "per_device_train_batch_size": 16,
            "per_device_eval_batch_size": 16,
            "num_train_epochs": 20,
            "weight_decay": 0.01,
            "hub_token" : "YOUR_HUB_TOKEN", 
            "push_to_hub" : true
        },
        "dataset_args" : { "text_column" : "text", "class_column" : "class" }
    }'
);
```

* project_name ('imdb_review_sentiment'): The project_name parameter specifies a unique name for your fine-tuning project. It helps identify and organize different fine-tuning tasks within the PostgreSQL database. In this example, the project is named 'imdb_review_sentiment,' reflecting the sentiment analysis task on the IMDb dataset. You can check `pgml.projects` for list of projects.

* task ('text-classification'): The task parameter defines the nature of the machine learning task to be performed. In this case, it's set to 'text-classification,' indicating that the fine-tuning is geared towards training a model for text classification.

* relation_name ('pgml.imdb_train_view'): The relation_name parameter identifies the training dataset to be used for fine-tuning. It specifies the view or table containing the training data. In this example, 'pgml.imdb_train_view' is the view created from the shuffled IMDb dataset, and it serves as the source for model training.

* model_name ('distilbert-base-uncased'): The model_name parameter denotes the pre-trained language model architecture to be fine-tuned. In this case, 'distilbert-base-uncased' is selected. DistilBERT is a distilled version of BERT, and the 'uncased' variant indicates that the model does not differentiate between uppercase and lowercase letters.

* test_size (0.2): The test_size parameter determines the proportion of the dataset reserved for testing during fine-tuning. In this example, 20% of the dataset is set aside for evaluation, helping assess the model's performance on unseen data.

* test_sampling ('last'): The test_sampling parameter defines the strategy for sampling test data from the dataset. In this case, 'last' indicates that the most recent portion of the data, following the specified test size, is used for testing. Adjusting this parameter might be necessary based on your specific requirements and dataset characteristics.

#### 5.1 Dataset Arguments (dataset_args)
The dataset_args section allows you to specify critical parameters related to your dataset for language model fine-tuning.

* text_column: The name of the column containing the text data in your dataset. In this example, it's set to "text."
* class_column: The name of the column containing the class labels in your dataset. In this example, it's set to "class."

#### 5.2 Training Arguments (training_args)
Fine-tuning a language model requires careful consideration of training parameters in the training_args section. Below is a subset of training args that you can pass to fine-tuning. You can find an exhaustive list of parameters in Hugging Face documentation on [TrainingArguments](https://huggingface.co/docs/transformers/main_classes/trainer#transformers.TrainingArguments).

* learning_rate: The learning rate for the training. It controls the step size during the optimization process. Adjust based on your model's convergence behavior.
* per_device_train_batch_size: The batch size per GPU for training. This parameter controls the number of training samples utilized in one iteration. Adjust based on your available GPU memory.
* per_device_eval_batch_size: The batch size per GPU for evaluation. Similar to per_device_train_batch_size, but used during model evaluation.
* num_train_epochs: The number of training epochs. An epoch is one complete pass through the entire training dataset. Adjust based on the model's convergence and your dataset size.
* weight_decay: L2 regularization term for weight decay. It helps prevent overfitting. Adjust based on the complexity of your model.
* hub_token: Your Hugging Face API token to push the fine-tuned model to the Hugging Face Model Hub. Replace "YOUR_HUB_TOKEN" with the actual token.
* push_to_hub: A boolean flag indicating whether to push the model to the Hugging Face Model Hub after fine-tuning.

#### 5.3 Monitoring
During training, metrics like loss, gradient norm will be printed as info and also logged in pgml.logs table. Below is a snapshot of such output.

```json
INFO:  {
    "loss": 0.3453,
    "grad_norm": 5.230295181274414,
    "learning_rate": 1.9e-05,
    "epoch": 0.25,
    "step": 500,
    "max_steps": 10000,
    "timestamp": "2024-03-07 01:59:15.090612"
}
INFO:  {
    "loss": 0.2479,
    "grad_norm": 2.7754225730895996,
    "learning_rate": 1.8e-05,
    "epoch": 0.5,
    "step": 1000,
    "max_steps": 10000,
    "timestamp": "2024-03-07 02:01:12.064098"
}
INFO:  {
    "loss": 0.223,
    "learning_rate": 1.6000000000000003e-05,
    "epoch": 1.0,
    "step": 2000,
    "max_steps": 10000,
    "timestamp": "2024-03-07 02:05:08.141220"
}
```

Once the training is completed, model will be evaluated against the validation dataset. You will see the below in the client terminal. Accuracy on the evaluation dataset is 0.934 and F1-score is 0.93. 

```json
INFO:  {
    "train_runtime": 2359.5335,
    "train_samples_per_second": 67.81,
    "train_steps_per_second": 4.238,
    "train_loss": 0.11267969808578492,
    "epoch": 5.0,
    "step": 10000,
    "max_steps": 10000,
    "timestamp": "2024-03-07 02:36:38.783279"
}
INFO:  {
    "eval_loss": 0.3691485524177551,
    "eval_f1": 0.9343711842996372,
    "eval_accuracy": 0.934375,
    "eval_runtime": 41.6167,
    "eval_samples_per_second": 192.23,
    "eval_steps_per_second": 12.014,
    "epoch": 5.0,
    "step": 10000,
    "max_steps": 10000,
    "timestamp": "2024-03-07 02:37:31.762917"
}
```

Once the training is completed, you can check query pgml.logs table using the model_id or by finding the latest model on the project. 

```bash
pgml: SELECT logs->>'epoch' AS epoch, logs->>'step' AS step, logs->>'loss' AS loss FROM pgml.logs WHERE model_id = 993 AND jsonb_exists(logs, 'loss');
 epoch | step  |  loss
-------+-------+--------
 0.25  | 500   | 0.3453
 0.5   | 1000  | 0.2479
 0.75  | 1500  | 0.223
 1.0   | 2000  | 0.2165
 1.25  | 2500  | 0.1485
 1.5   | 3000  | 0.1563
 1.75  | 3500  | 0.1559
 2.0   | 4000  | 0.142
 2.25  | 4500  | 0.0816
 2.5   | 5000  | 0.0942
 2.75  | 5500  | 0.075
 3.0   | 6000  | 0.0883
 3.25  | 6500  | 0.0432
 3.5   | 7000  | 0.0426
 3.75  | 7500  | 0.0444
 4.0   | 8000  | 0.0504
 4.25  | 8500  | 0.0186
 4.5   | 9000  | 0.0265
 4.75  | 9500  | 0.0248
 5.0   | 10000 | 0.0284
```

During training, model is periodically uploaded to Hugging Face Hub. You will find the model at `https://huggingface.co/<username>/<project_name>`. An example model that was automatically pushed to Hugging Face Hub is [here](https://huggingface.co/santiadavani/imdb_review_sentiement).

### 6. Inference using fine-tuned model
Now, that we have fine-tuned model on Hugging Face Hub, we can use [`pgml.transform`](/docs/open-source/pgml/api/pgml.transform) to perform real-time predictions as well as batch predictions. 

**Real-time predictions**

Here is an example pgml.transform call for real-time predictions on the newly minted LLM fine-tuned on IMDB review dataset.
```postgresql
 SELECT pgml.transform(
  task   => '{
    "task": "text-classification",
    "model": "santiadavani/imdb_review_sentiement"
  }'::JSONB,
  inputs => ARRAY[
    'I would not give this movie a rating, its not worthy. I watched it only because I am a Pfieffer fan. ',
    'This movie was sooooooo good! It was hilarious! There are so many jokes that you can just watch the'
  ]
);
                                               transform
--------------------------------------------------------------------------------------------------------
 [{"label": "negative", "score": 0.999561846256256}, {"label": "positive", "score": 0.986771047115326}]
(1 row)

Time: 175.264 ms
```

**Batch predictions**

```postgresql
pgml=# SELECT
    LEFT(text, 100) AS truncated_text,
    class,
    predicted_class[0]->>'label' AS predicted_class,
    (predicted_class[0]->>'score')::float AS score
FROM (
    SELECT
        LEFT(text, 100) AS text,
        class,
        pgml.transform(
            task => '{
                "task": "text-classification",
                "model": "santiadavani/imdb_review_sentiement"
            }'::JSONB,
            inputs => ARRAY[text]
        ) AS predicted_class
    FROM pgml.imdb_test_view
    LIMIT 2
) AS subquery;
                                            truncated_text                                            |  class   | predicted_class |       score
------------------------------------------------------------------------------------------------------+----------+-----------------+--------------------
 I wouldn't give this movie a rating, it's not worthy. I watched it only because I'm a Pfieffer fan.  | negative | negative        | 0.9996490478515624
 This movie was sooooooo good! It was hilarious! There are so many jokes that you can just watch the  | positive | positive        | 0.9972313046455384

 Time: 1337.290 ms (00:01.337)
 ```

## 7. Restarting Training from a Previous Trained Model

Sometimes, it's necessary to restart the training process from a previously trained model. This can be advantageous for various reasons, such as model fine-tuning, hyperparameter adjustments, or addressing interruptions in the training process. `pgml.tune` provides a seamless way to restart training while leveraging the progress made in the existing model. Below is a guide on how to restart training using a previous model as a starting point:

### Define the Previous Model

Specify the name of the existing model you want to use as a starting point. This is achieved by setting the `model_name` parameter in the `pgml.tune` function. In the example below, it is set to 'santiadavani/imdb_review_sentiement'.

```postgresql
model_name => 'santiadavani/imdb_review_sentiement',
```

### Adjust Hyperparameters
Fine-tune hyperparameters as needed for the restarted training process. This might include modifying learning rates, batch sizes, or training epochs. In the example below, hyperparameters such as learning rate, batch sizes, and epochs are adjusted.

```postgresql
"training_args": {
    "learning_rate": 2e-5,
    "per_device_train_batch_size": 16,
    "per_device_eval_batch_size": 16,
    "num_train_epochs": 1,
    "weight_decay": 0.01,
    "hub_token": "",
    "push_to_hub": true
},
```

### Ensure Consistent Dataset Configuration
Confirm that the dataset configuration remains consistent, including specifying the same text and class columns as in the previous training. This ensures compatibility between the existing model and the restarted training process.

```postgresql
"dataset_args": {
    "text_column": "text",
    "class_column": "class"
},
```

### Run the pgml.tune Function
Execute the `pgml.tune` function with the updated parameters to initiate the training restart. The function will leverage the existing model and adapt it based on the adjusted hyperparameters and dataset configuration.

```postgresql
SELECT pgml.tune(
    'imdb_review_sentiement',
    task => 'text-classification',
    relation_name => 'pgml.imdb_train_view',
    model_name => 'santiadavani/imdb_review_sentiement',
    test_size => 0.2,
    test_sampling => 'last',
    hyperparams => '{
        "training_args": {
            "learning_rate": 2e-5,
            "per_device_train_batch_size": 16,
            "per_device_eval_batch_size": 16,
            "num_train_epochs": 1,
            "weight_decay": 0.01,
            "hub_token": "YOUR_HUB_TOKEN",
            "push_to_hub": true
        },
        "dataset_args": { "text_column": "text", "class_column": "class" }
    }'
);
```

By following these steps, you can effectively restart training from a previously trained model, allowing for further refinement and adaptation of the model based on new requirements or insights. Adjust parameters as needed for your specific use case and dataset.

## 8. Hugging Face Hub vs. PostgresML as Model Repository
We utilize the Hugging Face Hub as the primary repository for fine-tuning Large Language Models (LLMs). Leveraging the HF hub offers several advantages:

* The HF repository serves as the platform for pushing incremental updates to the model during the training process. In the event of any disruptions in the database connection, you have the flexibility to resume training from where it was left off.
* If you prefer to keep the model private, you can push it to a private repository within the Hugging Face Hub. This ensures that the model is not publicly accessible by setting the parameter hub_private_repo to true.
* The pgml.transform function, designed around utilizing models from the Hugging Face Hub, can be reused without any modifications.

However, in certain scenarios, pushing the model to a central repository and pulling it for inference may not be the most suitable approach. To address this situation, we save all the model weights and additional artifacts, such as tokenizer configurations and vocabulary, in the pgml.files table at the end of the training process. It's important to note that as of the current writing, hooks to use models directly from pgml.files in the pgml.transform function have not been implemented. We welcome Pull Requests (PRs) from the community to enhance this functionality.

## Text Classification 9 Classes

### 1. Load and Shuffle the Dataset
In this section, we begin by loading the FinGPT sentiment analysis dataset using the `pgml.load_dataset` function. The dataset is then processed and organized into a shuffled view (pgml.fingpt_sentiment_shuffled_view), ensuring a randomized order of records. This step is crucial for preventing biases introduced by the original data ordering and enhancing the training process.

```postgresql
-- Load the dataset
SELECT pgml.load_dataset('FinGPT/fingpt-sentiment-train');

-- Create a shuffled view
CREATE VIEW pgml.fingpt_sentiment_shuffled_view AS
SELECT * FROM pgml."FinGPT/fingpt-sentiment-train" ORDER BY RANDOM();
```

### 2. Explore Class Distribution
Once the dataset is loaded and shuffled, we delve into understanding the distribution of sentiment classes within the data. By querying the shuffled view, we obtain valuable insights into the number of instances for each sentiment class. This exploration is essential for gaining a comprehensive understanding of the dataset and its inherent class imbalances.

```postgresql
-- Explore class distribution
SELECTpgml=# SELECT
    output,
    COUNT(*) AS class_count
FROM pgml.fingpt_sentiment_shuffled_view
GROUP BY output
ORDER BY output;

       output        | class_count
---------------------+-------------
 mildly negative     |        2108
 mildly positive     |        2548
 moderately negative |        2972
 moderately positive |        6163
 negative            |       11749
 neutral             |       29215
 positive            |       21588
 strong negative     |         218
 strong positive     |         211

```

### 3. Create Training and Test Views
To facilitate the training process, we create distinct views for training and testing purposes. The training view (pgml.fingpt_sentiment_train_view) contains 80% of the shuffled dataset, enabling the model to learn patterns and associations. Simultaneously, the test view (pgml.fingpt_sentiment_test_view) encompasses the remaining 20% of the data, providing a reliable evaluation set to assess the model's performance.

```postgresql
-- Create a view for training data (e.g., 80% of the shuffled records)
CREATE VIEW pgml.fingpt_sentiment_train_view AS
SELECT *
FROM pgml.fingpt_sentiment_shuffled_view
LIMIT (SELECT COUNT(*) * 0.8 FROM pgml.fingpt_sentiment_shuffled_view);

-- Create a view for test data (remaining 20% of the shuffled records)
CREATE VIEW pgml.fingpt_sentiment_test_view AS
SELECT *
FROM pgml.fingpt_sentiment_shuffled_view
OFFSET (SELECT COUNT(*) * 0.8 FROM pgml.fingpt_sentiment_shuffled_view);

```

### 4. Fine-Tune the Model for 9 Classes
In the final section, we kick off the fine-tuning process using the `pgml.tune` function. The model will be internally configured for sentiment analysis with 9 classes. The training is executed on the 80% of the train view and evaluated on the remaining 20% of the train view. The test view is reserved for evaluating the model's accuracy after training is completed. Please note that the option `hub_private_repo: true` is used to push the model to a private Hugging Face repository. 

```postgresql
-- Fine-tune the model for 9 classes without HUB token
SELECT pgml.tune(
    'fingpt_sentiement',
    task => 'text-classification',
    relation_name => 'pgml.fingpt_sentiment_train_view',
    model_name => 'distilbert-base-uncased',
    test_size => 0.2,
    test_sampling => 'last',
    hyperparams => '{
        "training_args": {
            "learning_rate": 2e-5,
            "per_device_train_batch_size": 16,
            "per_device_eval_batch_size": 16,
            "num_train_epochs": 5,
            "weight_decay": 0.01,
            "hub_token" : "YOUR_HUB_TOKEN",
            "push_to_hub": true,
            "hub_private_repo": true
        },
        "dataset_args": { "text_column": "input", "class_column": "output" }
    }'
);

```

## Conversation

In this section, we will discuss conversational task using state-of-the-art NLP techniques. Conversational AI has garnered immense interest and significance in recent years due to its wide range of applications, from virtual assistants to customer service chatbots and beyond.

### Understanding the Conversation Task

At the core of conversational AI lies the conversation task, a fundamental NLP problem that involves processing and generating human-like text-based interactions. Let's break down this task into its key components:

- **Input:** The input to the conversation task typically consists of a sequence of conversational turns, often represented as text. These turns can encompass a dialogue between two or more speakers, capturing the flow of communication over time.

- **Model:** Central to the conversation task is the NLP model, which is trained to understand the nuances of human conversation and generate appropriate responses. These models leverage sophisticated transformer based architectures like Llama2, Mistral, GPT etc., empowered by large-scale datasets and advanced training techniques.

- **Output:** The ultimate output of the conversation task is the model's response to the input conversation. This response aims to be contextually relevant, coherent, and engaging, reflecting a natural human-like interaction.

### Versatility of the Conversation Task

What makes the conversation task truly remarkable is its remarkable versatility. Beyond its traditional application in dialogue systems, the conversation task can be adapted to solve several NLP problems by tweaking the input representation or task formulation.

- **Text Classification:** By providing individual utterances with corresponding labels, the conversation task can be repurposed for tasks such as sentiment analysis, intent detection, or topic classification.

    **Input:**
    - System: Chatbot: "Hello! How can I assist you today?"
    - User: "I'm having trouble connecting to the internet."

    **Model Output (Text Classification):**
    - Predicted Label: Technical Support
    - Confidence Score: 0.85

- **Token Classification:** Annotating the conversation with labels for specific tokens or phrases enables applications like named entity recognition within conversational text.

    **Input:**
    - System: Chatbot: "Please describe the issue you're facing in detail."
    - User: "I can't access any websites, and the Wi-Fi indicator on my router is blinking."

    **Model Output (Token Classification):**
    - User's Description: "I can't access any websites, and the Wi-Fi indicator on my router is blinking."
    - Token Labels:
    - "access" - Action
    - "websites" - Entity (Location)
    - "Wi-Fi" - Entity (Technology)
    - "indicator" - Entity (Device Component)
    - "blinking" - State

- **Question Answering:** Transforming conversational exchanges into a question-answering format enables extracting relevant information and providing concise answers, akin to human comprehension and response.

    **Input:**
    - System: Chatbot: "How can I help you today?"
    - User: "What are the symptoms of COVID-19?"

    **Model Output (Question Answering):**
    - Answer: "Common symptoms of COVID-19 include fever, cough, fatigue, shortness of breath, loss of taste or smell, and body aches."

### Fine-tuning Llama2-7b model using LoRA
In this section, we will explore how to fine-tune the Llama2-7b-chat large language model for the financial sentiment data discussed in the previous [section](#text-classification-9-classes) utilizing the pgml.tune function and employing the LoRA approach.  LoRA is a technique that enables efficient fine-tuning of large language models by only updating a small subset of the model's weights during fine-tuning, while keeping the majority of the weights frozen. This approach can significantly reduce the computational requirements and memory footprint compared to traditional full model fine-tuning.

```postgresql
SELECT pgml.tune(
    'fingpt-llama2-7b-chat',
    task => 'conversation',
    relation_name => 'pgml.fingpt_sentiment_train_view',
    model_name => 'meta-llama/Llama-2-7b-chat-hf',
    test_size => 0.8,
    test_sampling => 'last',
    hyperparams => '{
        "training_args" : {
            "learning_rate": 2e-5,
            "per_device_train_batch_size": 4,
            "per_device_eval_batch_size": 4,
            "num_train_epochs": 1,
            "weight_decay": 0.01,
            "hub_token" : "HF_TOKEN", 
            "push_to_hub" : true,
            "optim" : "adamw_bnb_8bit",
            "gradient_accumulation_steps" : 4,
            "gradient_checkpointing" : true
        },
        "dataset_args" : { "system_column" : "instruction", "user_column" : "input", "assistant_column" : "output" },
        "lora_config" : {"r": 2, "lora_alpha" : 4, "lora_dropout" : 0.05, "bias": "none", "task_type": "CAUSAL_LM"},
        "load_in_8bit" : false,
        "token" : "HF_TOKEN"
    }'
);
```
Let's break down each argument and its significance:

1. **Model Name (`model_name`):**
   - This argument specifies the name or identifier of the base model that will be fine-tuned. In the context of the provided query, it refers to the pre-trained model "meta-llama/Llama-2-7b-chat-hf."

2. **Task (`task`):**
   - Indicates the specific task for which the model is being fine-tuned. In this case, it's set to "conversation," signifying that the model will be adapted to process conversational data.

3. **Relation Name (`relation_name`):**
   - Refers to the name of the dataset or database relation containing the training data used for fine-tuning. In the provided query, it's set to "pgml.fingpt_sentiment_train_view."

4. **Test Size (`test_size`):**
   - Specifies the proportion of the dataset reserved for testing, expressed as a fraction. In the example, it's set to 0.8, indicating that 80% of the data will be used for training, and the remaining 20% will be held out for testing.

5. **Test Sampling (`test_sampling`):**
   - Determines the strategy for sampling the test data. In the provided query, it's set to "last," indicating that the last portion of the dataset will be used for testing.

6. **Hyperparameters (`hyperparams`):**
   - This argument encapsulates a JSON object containing various hyperparameters essential for the fine-tuning process. Let's break down its subcomponents:
     - **Training Args (`training_args`):** Specifies parameters related to the training process, including learning rate, batch size, number of epochs, weight decay, optimizer settings, and other training configurations.
     - **Dataset Args (`dataset_args`):** Provides arguments related to dataset processing, such as column names for system responses, user inputs, and assistant outputs.
     - **LORA Config (`lora_config`):** Defines settings for the LORA (Learned Optimizer and Rate Adaptation) algorithm, including parameters like the attention radius (`r`), LORA alpha (`lora_alpha`), dropout rate (`lora_dropout`), bias, and task type.
     - **Load in 8-bit (`load_in_8bit`):** Determines whether to load data in 8-bit format, which can be beneficial for memory and performance optimization.
     - **Token (`token`):** Specifies the Hugging Face token required for accessing private repositories and pushing the fine-tuned model to the Hugging Face Hub.

7. **Hub Private Repo (`hub_private_repo`):**
   - This optional parameter indicates whether the fine-tuned model should be pushed to a private repository on the Hugging Face Hub. In the provided query, it's set to `true`, signifying that the model will be stored in a private repository.

### Training Args:

Expanding on the `training_args` within the `hyperparams` argument provides insight into the specific parameters governing the training process of the model. Here's a breakdown of the individual training arguments and their significance:

- **Learning Rate (`learning_rate`):**
   - Determines the step size at which the model parameters are updated during training. A higher learning rate may lead to faster convergence but risks overshooting optimal solutions, while a lower learning rate may ensure more stable training but may take longer to converge.

- **Per-device Train Batch Size (`per_device_train_batch_size`):**
   - Specifies the number of training samples processed in each batch per device during training. Adjusting this parameter can impact memory usage and training speed, with larger batch sizes potentially accelerating training but requiring more memory.

- **Per-device Eval Batch Size (`per_device_eval_batch_size`):**
   - Similar to `per_device_train_batch_size`, this parameter determines the batch size used for evaluation (validation) during training. It allows for efficient evaluation of the model's performance on validation data.

- **Number of Train Epochs (`num_train_epochs`):**
   - Defines the number of times the entire training dataset is passed through the model during training. Increasing the number of epochs can improve model performance up to a certain point, after which it may lead to overfitting.

- **Weight Decay (`weight_decay`):**
   - Introduces regularization by penalizing large weights in the model, thereby preventing overfitting. It helps to control the complexity of the model and improve generalization to unseen data.

- **Hub Token (`hub_token`):**
   - A token required for authentication when pushing the fine-tuned model to the Hugging Face Hub or accessing private repositories. It ensures secure communication with the Hub platform.

- **Push to Hub (`push_to_hub`):**
   - A boolean flag indicating whether the fine-tuned model should be uploaded to the Hugging Face Hub after training. Setting this parameter to `true` facilitates sharing and deployment of the model for wider usage.

- **Optimizer (`optim`):**
   - Specifies the optimization algorithm used during training. In the provided query, it's set to "adamw_bnb_8bit," indicating the use of the AdamW optimizer with gradient clipping and 8-bit quantization.

- **Gradient Accumulation Steps (`gradient_accumulation_steps`):**
   - Controls the accumulation of gradients over multiple batches before updating the model's parameters. It can help mitigate memory constraints and stabilize training, especially with large batch sizes.

- **Gradient Checkpointing (`gradient_checkpointing`):**
    - Enables gradient checkpointing, a memory-saving technique that trades off compute for memory during backpropagation. It allows training of larger models or with larger batch sizes without running out of memory.

Each of these training arguments plays a crucial role in shaping the training process, ensuring efficient convergence, regularization, and optimization of the model for the specific task at hand. Adjusting these parameters appropriately is essential for achieving optimal model performance.

### LORA Args:

Expanding on the `lora_config` within the `hyperparams` argument provides clarity on its role in configuring the LORA (Learned Optimizer and Rate Adaptation) algorithm:

- **Attention Radius (`r`):**
   - Specifies the radius of the attention window for the LORA algorithm. It determines the range of tokens considered for calculating attention weights, allowing the model to focus on relevant information while processing conversational data.

- **LORA Alpha (`lora_alpha`):**
   - Controls the strength of the learned regularization term in the LORA algorithm. A higher alpha value encourages sparsity in attention distributions, promoting selective attention and enhancing interpretability.

- **LORA Dropout (`lora_dropout`):**
   - Defines the dropout rate applied to the LORA attention scores during training. Dropout introduces noise to prevent overfitting and improve generalization by randomly zeroing out a fraction of attention weights.

- **Bias (`bias`):**
   - Determines whether bias terms are included in the LORA attention calculation. Bias terms can introduce additional flexibility to the attention mechanism, enabling the model to learn more complex relationships between tokens.

- **Task Type (`task_type`):**
   - Specifies the type of task for which the LORA algorithm is applied. In this context, it's set to "CAUSAL_LM" for causal language modeling, indicating that the model predicts the next token based on the previous tokens in the sequence.

Configuring these LORA arguments appropriately ensures that the attention mechanism of the model is optimized for processing conversational data, allowing it to capture relevant information and generate coherent responses effectively.

### Dataset Args:

Expanding on the `dataset_args` within the `hyperparams` argument provides insight into its role in processing the dataset:

- **System Column (`system_column`):**
   - Specifies the name or identifier of the column containing system responses (e.g., prompts or instructions) within the dataset. This column is crucial for distinguishing between different types of conversational turns and facilitating model training.

- **User Column (`user_column`):**
   - Indicates the column containing user inputs or queries within the dataset. These inputs form the basis for the model's understanding of user intentions, sentiments, or requests during training and inference.

- **Assistant Column (`assistant_column`):**
   - Refers to the column containing assistant outputs or responses generated by the model during training. These outputs serve as targets for the model to learn from and are compared against the actual responses during evaluation to assess model performance.

Configuring these dataset arguments ensures that the model is trained on the appropriate input-output pairs, enabling it to learn from the conversational data and generate contextually relevant responses.

Once the fine-tuning is completed, you will see the model in your Hugging Face repository (example: https://huggingface.co/santiadavani/fingpt-llama2-7b-chat). Since we are using LoRA to fine tune the model we only save the adapter weights (~2MB) instead of all the 7B weights (14GB) in Llama2-7b model.  

## Inference
For inference, we will be utilizing the [OpenSourceAI](https://postgresml.org/docs/open-source/korvus/guides/opensourceai) class from the [pgml SDK](https://postgresml.org/docs/open-source/korvus/). Here's an example code snippet:

```python
import pgml

database_url = "DATABASE_URL"

client = pgml.OpenSourceAI(database_url)

results = client.chat_completions_create(
    {
        "model" : "santiadavani/fingpt-llama2-7b-chat",
        "token" : "TOKEN",
        "load_in_8bit": "true",
        "temperature" : 0.1,
        "repetition_penalty" : 1.5,
    },
    [
        {
            "role" : "system",
            "content" : "What is the sentiment of this news? Please choose an answer from {strong negative/moderately negative/mildly negative/neutral/mildly positive/moderately positive/strong positive}.",
        },
        {
            "role": "user",
            "content": "Starbucks says the workers violated safety policies while workers said they'd never heard of the policy before and are alleging retaliation.",
        },
    ]
)

print(results)
```

In this code snippet, we first import the pgml module and create an instance of the OpenSourceAI class, providing the necessary database URL. We then call the chat_completions_create method, specifying the model we want to use (in this case, "santiadavani/fingpt-llama2-7b-chat"), along with other parameters such as the token, whether to load the model in 8-bit precision, the temperature for sampling, and the repetition penalty.

The chat_completions_create method takes two arguments: a dictionary containing the model configuration and a list of dictionaries representing the chat conversation. In this example, the conversation consists of a system prompt asking for the sentiment of a given news snippet, and a user message containing the news text.

The results are:

```json
{
    "choices": [
        {
            "index": 0,
            "message": {
                "content": " Moderately negative ",
                "role": "assistant"
            }
        }
    ],
    "created": 1711144872,
    "id": "b663f701-db97-491f-b186-cae1086f7b79",
    "model": "santiadavani/fingpt-llama2-7b-chat",
    "object": "chat.completion",
    "system_fingerprint": "e36f4fa5-3d0b-e354-ea4f-950cd1d10787",
    "usage": {
        "completion_tokens": 0,
        "prompt_tokens": 0,
        "total_tokens": 0
    }
}
```

This dictionary contains the response from the language model, `santiadavani/fingpt-llama2-7b-chat`, for the given news text.

The key information in the response is:

1. `choices`: A list containing the model's response. In this case, there is only one choice.
2. `message.content`: The actual response from the model, which is " Moderately negative".
3. `model`: The name of the model used, "santiadavani/fingpt-llama2-7b-chat".
4. `created`: A timestamp indicating when the response was generated.
5. `id`: A unique identifier for this response.
6. `object`: Indicates that this is a "chat.completion" object.
7. `usage`: Information about the token usage for this response, although all values are 0 in this case.

So, the language model has analyzed the news text **_Starbucks says the workers violated safety policies while workers said they'd never heard of the policy before and are alleging retaliation._** and determined that the sentiment expressed in this text is **_Moderately negative_**
