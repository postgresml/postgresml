# Fine Tuning

Pre-trained models allow you to get up and running quickly, but you can likely improve performance on your dataset by fine tuning them. Normally, you'll bring your own data to the party, but for these examples we'll use datasets published on Hugging Face. Make sure you've installed the required data dependencies detailed in [setup](/docs/user_guides/transformers/setup).

## Translation Example
The [Helsinki-NLP](https://huggingface.co/Helsinki-NLP) organization provides more than a thousand pre-trained models to translate between different language pairs. These can be further fine tuned on additional datasets with domain specific vocabulary. Researchers have also created large collections of documents that have been manually translated across languages by experts for training data. 

### Prepare the data
The [kde4](https://huggingface.co/datasets/kde4) dataset contains many language pairs. Subsets can be loaded into your Postgres instance with a call to `pgml.load_dataset`, or you may wish to create your own fine tuning dataset with vocabulary specific to your domain.

```postgresql
SELECT pgml.load_dataset('kde4', kwargs => '{"lang1": "en", "lang2": "es"}');
```

You can view the newly loaded data in your Postgres database:

=== "SQL"

```postgresql
SELECT * FROM pgml.kde4 LIMIT 5;
```

=== "Result"

```postgresql
id  |
translation

-----+-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
99  | {"en": "If you wish to manipulate the DOM tree in any way you will have to use an external script to do so.", "es": "Si desea manipular el árbol DOM deberá utilizar un script externo para hacerlo."}
100 | {"en": "Credits", "es": "Créditos"}
101 | {"en": "The domtreeviewer plugin is Copyright & copy; 2001 The Kafka Team/ Andreas Schlapbach kde-kafka@master. kde. org schlpbch@unibe. ch", "es": "Derechos de autor de la extensión domtreeviewer & copy;. 2001. El equipo de Kafka/ Andreas Schlapbach kde-kafka@master. kde. org schlpbch@unibe. ch."}
102 | {"en": "Josef Weidendorfer Josef. Weidendorfer@gmx. de", "es": "Josef Weidendorfer Josef. Weidendorfer@gmx. de"}
103 | {"en": "ROLES_OF_TRANSLATORS", "es": "Rafael Osuna rosuna@wol. es Traductor"}
(5 rows)
```

===

This huggingface dataset stores the data as language key pairs in a JSON document. To use it with PostgresML, we'll need to provide a `VIEW` that structures the data into more primitively typed columns.

=== "SQL"

```postgresql
CREATE OR REPLACE VIEW kde4_en_to_es AS
SELECT translation->>'en' AS "en", translation->>'es' AS "es"
FROM pgml.kde4
LIMIT 10;
```

=== "Result"

```
CREATE VIEW
```

===

Now, we can see the data in more normalized form. The exact column names don't matter for now, we'll specify which one is the target during the training call, and the other one will be used as the input.

=== "SQL"

```postgresql
SELECT * FROM kde4_en_to_es LIMIT 10;
```

=== "Result"

```postgresql
                                            en                                            |                                                   es

--------------------------------------------------------------------------------------------+--------------------------------------------------------------------------
------------------------------
 Lauri Watts                                                                                | Lauri Watts
 & Lauri. Watts. mail;                                                                      | & Lauri. Watts. mail;
 ROLES_OF_TRANSLATORS                                                                       | Rafael Osuna rosuna@wol. es Traductor Miguel Revilla Rodríguez yo@miguelr
evilla. com Traductor
 2006-02-26 3.5.1                                                                           | 2006-02-26 3.5.1
 The Babel & konqueror; plugin gives you quick access to the Babelfish translation service. | La extensión Babel de & konqueror; le permite un acceso rápido al servici
o de traducción de Babelfish.
 KDE                                                                                        | KDE
 kdeaddons                                                                                  | kdeaddons
 konqueror                                                                                  | konqueror
 plugins                                                                                    | extensiones
 babelfish                                                                                  | babelfish
(10 rows)
```

===


### Tune the model
Tuning is very similar to training with PostgresML, although we specify a `model_name` to download from Hugging Face instead of the base `algorithm`.

```postgresql
SELECT pgml.tune(
    'Translate English to Spanish',
    task => 'translation',
    relation_name => 'kde4_en_to_es',
    y_column_name => 'es', -- translate into spanish
    model_name => 'Helsinki-NLP/opus-mt-en-es',
    hyperparams => '{
        "learning_rate": 2e-5,
        "per_device_train_batch_size": 16,
        "per_device_eval_batch_size": 16,
        "num_train_epochs": 1,
        "weight_decay": 0.01,
        "max_length": 128
    }',
    test_size => 0.5,
    test_sampling => 'last'
);
```

### Generate Translations

!!! note

Translations use the `pgml.generate` API since they return `TEXT` rather than numeric values. You may also call `pgml.generate` with a `TEXT[]` for batch processing.

!!!

=== "SQL"

```postgresql

SELECT pgml.generate('Translate English to Spanish', 'I love SQL')
AS spanish;
```

=== "Result"

```postgresql
    spanish
----------------
Me encanta SQL
(1 row)

Time: 126.837 ms
```

===

See the [task documentation](https://huggingface.co/tasks/translation) for more examples, use cases, models and datasets.


## Text Classification Example

DistilBERT is a small, fast, cheap and light Transformer model based on the BERT architecture. It can be fine tuned on specific datasets to learn further nuance between positive and negative examples. For this example, we'll fine tune `distilbert-base-uncased` on the IMBD dataset, which is a list of movie reviews along with a positive or negative label.

Without tuning, DistilBERT classifies every single movie review as `positive`, and has a F<sub>1</sub> score of 0.367, which is about what you'd expect for a relatively useless classifier. However, after training for a single epoch (takes about 10 minutes on an Nvidia 1080 TI), the F<sub>1</sub> jumps to 0.928 which is a huge improvement, indicating DistilBERT can now fairly accurately predict sentiment from IMDB reviews. Further training for another epoch only results in a very minor improvement to 0.931, and the 3rd epoch is flat, also at 0.931 which indicates DistilBERT is unlikely to continue learning more about this particular dataset with additional training. You can view the results of each model, like those trained from scratch, in the dashboard. 

Once our model has been fine tuned on the dataset, it'll be saved and deployed with a Project visible in the Dashboard, just like models built from simpler algorithms.

![Fine Tuning](/dashboard/static/images/dashboard/tuning.png)

### Prepare the data
The IMDB dataset has 50,000 examples of user reviews with positive or negative viewing experiences as the labels, and is split 50/50 into training and evaluation datasets.

```postgresql
SELECT pgml.load_dataset('imdb');
```

You can view the newly loaded data in your Postgres database:

=== "SQL"

```postgresql
SELECT * FROM pgml.imdb LIMIT 1;
```

=== "Result"

```postgresql
                                                                                                                                                            text                                                                                                                                           | label
-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+-------
 This has to be the funniest stand up comedy I have ever seen. Eddie Izzard is a genius, he picks in Brits, Americans and everyone in between. His style is completely natural and completely hilarious. I doubt that anyone could sit through this and not laugh their a** off. Watch, enjoy, it's funny. |     1
(1 row)
```

===

### Tune the model

Tuning has a nearly identical API to training, except you may pass the name of a [model published on Hugging Face](https://huggingface.co/models) to start with, rather than training an algorithm from scratch.

```postgresql
SELECT pgml.tune(
    'IMDB Review Sentiment',
    task => 'text-classification',
    relation_name => 'pgml.imdb',
    y_column_name => 'label',
    model_name => 'distilbert-base-uncased',
    hyperparams => '{
        "learning_rate": 2e-5,
        "per_device_train_batch_size": 16,
        "per_device_eval_batch_size": 16,
        "num_train_epochs": 1,
        "weight_decay": 0.01
    }',
    test_size => 0.5,
    test_sampling => 'last'
);
```

### Make predictions

=== "SQL"

```postgresql
SELECT pgml.predict('IMDB Review Sentiment', 'I love SQL')
AS sentiment;
```

=== "Result"

```
sentiment
-----------
1
(1 row)

Time: 16.681 ms
```

===

The default for predict in a classification problem classifies the statement as one of the labels. In this case, 0 is negative and 1 is positive. If you'd like to check the individual probabilities associated with each class you can use the `predict_proba` API:

=== "SQL"

```postgresql
SELECT pgml.predict_proba('IMDB Review Sentiment', 'I love SQL')
AS sentiment;
```

=== "Result"

```
                sentiment
-------------------------------------------
[0.06266672909259796, 0.9373332858085632]
(1 row)

Time: 18.101 ms
```

===

This shows that there is a 6.26% chance for category 0 (negative sentiment), and a 93.73% chance it's category 1 (positive sentiment).

See the [task documentation](https://huggingface.co/tasks/text-classification) for more examples, use cases, models and datasets.

## Summarization Example
At a high level, summarization uses similar techniques to translation. Both use an input sequence to generate an output sequence. The difference being that summarization extracts the most relevant parts of the input sequence to generate the output.

### Prepare the data
[BillSum](https://huggingface.co/datasets/billsum) is a dataset with training examples that summarize US Congressional and California state bills. You can pass `kwargs` specific to loading datasets, in this case we'll restrict the dataset to California samples:

```postgresql
SELECT pgml.load_dataset('billsum', kwargs => '{"split": "ca_test"}');
```

You can view the newly loaded data in your Postgres database:

=== "SQL"

```postgresql
SELECT * FROM pgml.billsum LIMIT 1;
```

=== "Result"

```
                                        text                                                                                                                                                                                                                                                                                                                                                                    |                                                                                                                                                                                                       summary                                                                                                                                                                                                        |                                                            title
-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+-----------------------------------------------------------------------------------------------------------------------------
The people of the State of California do enact as follows:                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               +| Existing property tax law establishes a veterans’ organization exemption under which property is exempt from taxation if, among other things, that property is used exclusively for charitable purposes and is owned by a veterans’ organization.                                                                                                                                                                   +| An act to amend Section 215.1 of the Revenue and Taxation Code, relating to taxation, to take effect immediately, tax levy.
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        +| This bill would provide that the veterans’ organization exemption shall not be denied to a property on the basis that the property is used for fraternal, lodge, or social club purposes, and would make specific findings and declarations in that regard. The bill would also provide that the exemption shall not apply to any portion of a property that consists of a bar where alcoholic beverages are served.+|
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        +| Section 2229 of the Revenue and Taxation Code requires the Legislature to reimburse local agencies annually for certain property tax revenues lost as a result of any exemption or classification of property for purposes of ad valorem property taxation.                                                                                                                                                         +|
SECTION 1.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               +| This bill would provide that, notwithstanding Section 2229 of the Revenue and Taxation Code, no appropriation is made and the state shall not reimburse local agencies for property tax revenues lost by them pursuant to the bill.                                                                                                                                                                                 +|
The Legislature finds and declares all of the following:                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 +| This bill would take effect immediately as a tax levy.                                                                                                                                                                                                                                                                                                                                                               |
(a) (1) Since 1899 congressionally chartered veterans’ organizations have provided a valuable service to our nation’s returning service members. These organizations help preserve the memories and incidents of the great hostilities fought by our nation, and preserve and strengthen comradeship among members.                                                                                                                                                                                                                                                                                                                                                                                                                      +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(2) These veterans’ organizations also own and manage various properties including lodges, posts, and fraternal halls. These properties act as a safe haven where veterans of all ages and their families can gather together to find camaraderie and fellowship, share stories, and seek support from people who understand their unique experiences. This aids in the healing process for these returning veterans, and ensures their health and happiness.                                                                                                                                                                                                                                                                            +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(b) As a result of congressional chartering of these veterans’ organizations, the United States Internal Revenue Service created a special tax exemption for these organizations under Section 501(c)(19) of the Internal Revenue Code.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(c) Section 501(c)(19) of the Internal Revenue Code and related federal regulations provide for the exemption for posts or organizations of war veterans, or an auxiliary unit or society of, or a trust or foundation for, any such post or organization that, among other attributes, carries on programs to perpetuate the memory of deceased veterans and members of the Armed Forces and to comfort their survivors, conducts programs for religious, charitable, scientific, literary, or educational purposes, sponsors or participates in activities of a patriotic nature, and provides social and recreational activities for their members.                                                                                   +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(d) Section 215.1 of the Revenue and Taxation Code stipulates that all buildings, support and so much of the real property on which the buildings are situated as may be required for the convenient use and occupation of the buildings, used exclusively for charitable purposes, owned by a veterans’ organization that has been chartered by the Congress of the United States, organized and operated for charitable purposes, when the same are used solely and exclusively for the purpose of the organization, if not conducted for profit and no part of the net earnings of which ensures to the benefit of any private individual or member thereof, are exempt from taxation.                                                +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(e) The Chief Counsel of the State Board of Equalization concluded, based on a 1979 appellate court decision, that only parts of American Legion halls are exempt from property taxation and that other parts, such as billiard rooms, card rooms, and similar areas, are not exempt.                                                                                                                                                                                                                                                                                                                                                                                                                                                    +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(f) In a 1994 memorandum, the State Board of Equalization’s legal division further concluded that the areas normally considered eligible for exemptions are the office areas used to counsel veterans and the area used to store veterans’ records, but that the meeting hall and bar found in most of the facilities are not considered used for charitable purposes.                                                                                                                                                                                                                                                                                                                                                                   +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(g) Tax-exempt status is intended to provide economic incentive and support to veterans’ organizations to provide for the social welfare of the community of current and former military personnel.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(h) The State Board of Equalization’s constriction of the tax exemption has resulted in an onerous tax burden on California veteran service organizations posts or halls, hinders the posts’ ability to provide facilities for veterans, and threatens the economic viability of many local organizations.                                                                                                                                                                                                                                                                                                                                                                                                                               +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(i) The charitable activities of a veteran service organizations post or hall are much more than the counseling of veterans. The requirements listed for qualification for the federal tax exemption clearly dictate a need for more than just an office.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(j) Programs to perpetuate the memory of deceased veterans and members of the Armed Forces and to comfort their survivors require the use of facilities for funerals and receptions.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(k) Programs for religious, charitable, scientific, literary, or educational purposes require space for more than 50 attendees.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(l) Activities of a patriotic nature need facilities to accommodate hundreds of people.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(m) Social and recreational activities for members require precisely those areas considered “not used for charitable purposes” by the State Board of Equalization.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(n) The State Board of Equalization’s interpretation of the Revenue and Taxation Code reflects a lack of understanding of the purpose and programs of the veterans service organizations posts or halls and is detrimental to the good works performed in support of our veteran community.

                                                                                    +|
                                                                                    +|                                                                                                                                                                                                                                   (g) Tax-exempt status is intended to provide economic incentive and support to veterans’ organizations to provide for the social welfare of the community of current and former military personnel.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(h) The State Board of Equalization’s constriction of the tax exemption has resulted in an onerous tax burden on California veteran service organizations posts or halls, hinders the posts’ ability to provide facilities for veterans, and threatens the economic viability of many local organizations.                                                                                                                                                                                                                                                                                                                                                                                                                               +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(i) The charitable activities of a veteran service organizations post or hall are much more than the counseling of veterans. The requirements listed for qualification for the federal tax exemption clearly dictate a need for more than just an office.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(j) Programs to perpetuate the memory of deceased veterans and members of the Armed Forces and to comfort their survivors require the use of facilities for funerals and receptions.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(k) Programs for religious, charitable, scientific, literary, or educational purposes require space for more than 50 attendees.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(l) Activities of a patriotic nature need facilities to accommodate hundreds of people.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(m) Social and recreational activities for members require precisely those areas considered “not used for charitable purposes” by the State Board of Equalization.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(n) The State Board of Equalization’s interpretation of the Revenue and Taxation Code reflects a lack of understanding of the purpose and programs of the veterans service organizations posts or halls and is detrimental to the good works performed in support of our veteran community.                                                                                                                                                                                                                                                                                                                                                                                                                                              +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
SECTION 1.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
SEC. 2.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
Section 215.1 of the Revenue and Taxation Code is amended to read:                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
215.1.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(a) All buildings, and so much of the real property on which the buildings are situated as may be required for the convenient use and occupation of the buildings, used exclusively for charitable purposes, owned by a veterans’ organization that has been chartered by the Congress of the United States, organized and operated for charitable purposes, and exempt from federal income tax as an organization described in Section 501(c)(19) of the Internal Revenue Code when the same are used solely and exclusively for the purpose of the organization, if not conducted for profit and no part of the net earnings of which inures to the benefit of any private individual or member thereof, shall be exempt from taxation.+|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(b) The exemption provided for in this section shall apply to the property of all organizations meeting the requirements of this section, subdivision (b) of Section 4 of Article XIII of the California Constitution, and paragraphs (1) to (4), inclusive, (6), and (7) of subdivision (a) of Section 214.                                                                                                                                                                                                                                                                                                                                                                                                                             +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(c) (1) The exemption specified by subdivision (a) shall not be denied to a property on the basis that the property is used for fraternal, lodge, or social club purposes.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(2) With regard to this subdivision, the Legislature finds and declares all of the following:                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(A) The exempt activities of a veterans’ organization as described in subdivision (a) qualitatively differ from the exempt activities of other nonprofit entities that use property for fraternal, lodge, or social club purposes in that the exempt purpose of the veterans’ organization is to conduct programs to perpetuate the memory of deceased veterans and members of the Armed Forces and to comfort their survivors, to conduct programs for religious, charitable, scientific, literary, or educational purposes, to sponsor or participate in activities of a patriotic nature, and to provide social and recreational activities for their members.                                                                        +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(B) In light of this distinction, the use of real property by a veterans’ organization as described in subdivision (a), for fraternal, lodge, or social club purposes is central to that organization’s exempt purposes and activities.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(C) In light of the factors set forth in subparagraphs (A) and (B), the use of real property by a veterans’ organization as described in subdivision (a) for fraternal, lodge, or social club purposes, constitutes the exclusive use of that property for a charitable purpose within the meaning of subdivision (b) of Section 4 of Article XIII of the California Constitution.                                                                                                                                                                                                                                                                                                                                                       +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(d) The exemption provided for in this section shall not apply to any portion of a property that consists of a bar where alcoholic beverages are served. The portion of the property ineligible for the veterans’ organization exemption shall be that area used primarily to prepare and serve alcoholic beverages.                                                                                                                                                                                                                                                                                                                                                                                                                     +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(e) An organization that files a claim for the exemption provided for in this section shall file with the assessor a valid organizational clearance certificate issued pursuant to Section 254.6.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        +|                                                                                                                                                                                                                                                                                                                                                                                                                      |
(f) This exemption shall be known as the “veterans’ organization exemption.”

                                                                                    +|
                                                                                                                                                                                    |
SEC. 2.

                                                                                    +|
                                                                                                                                                                                    |
SEC. 3.

                                                                                    +|
                                                                                                                                                                                    |
Notwithstanding Section 2229 of the Revenue and Taxation Code, no appropriation is made by this act and the state shall not reimburse any local agency for any property tax revenues lost by it pursuant to this act.

                                                                                    +|
                                                                                                                                                                                    |
SEC. 3.

                                                                                    +|
                                                                                                                                                                                    |
SEC. 4.

                                                                                    +|
                                                                                                                                                                                    |
This act provides for a tax levy within the meaning of Article IV of the Constitution and shall go into immediate effect.

                                                                                    |
                                                                                                                                                                                    |
(1 row)
```

===

This dataset has 3 fields, but summarization transformers only take a single input to produce their output. We can create a view that simply omits the `title` from the training data:

```postgresql
CREATE OR REPLACE VIEW billsum_training_data
AS SELECT "text", summary FROM pgml.billsum;
```

Or, it might be interesting to concat the title to the text field to see how relevant it actually is to the bill. If the title of a bill is the first sentence, and doesn't appear in summary, it may indicate that it's a poorly chosen title for the bill:

```postgresql
CREATE OR REPLACE VIEW billsum_training_data
AS SELECT title || '\n' || "text" AS "text", summary FROM pgml.billsum 
LIMIT 10;
```

### Tune the model

Tuning has a nearly identical API to training, except you may pass the name of a [model published on Hugging Face](https://huggingface.co/models) to start with, rather than training an algorithm from scratch. 

```postgresql
SELECT pgml.tune(
    'Legal Summarization',
    task => 'summarization',
    relation_name => 'billsum_training_data',
    y_column_name => 'summary',
    model_name => 'sshleifer/distilbart-xsum-12-1',
    hyperparams => '{
        "learning_rate": 2e-5,
        "per_device_train_batch_size": 2,
        "per_device_eval_batch_size": 2,
        "num_train_epochs": 1,
        "weight_decay": 0.01,
        "max_length": 1024
    }',
    test_size => 0.2,
    test_sampling => 'last'
);
```


### Make predictions

=== "SQL"

```postgresql
SELECT pgml.predict('IMDB Review Sentiment', 'I love SQL') AS sentiment;
```

=== "Result"

```
sentiment
-----------
1
(1 row)

Time: 16.681 ms
```

===

The default for predict in a classification problem classifies the statement as one of the labels. In this case 0 is negative and 1 is positive. If you'd like to check the individual probabilities associated with each class you can use the `predict_proba` API

=== "SQL"

```postgresql
SELECT pgml.predict_proba('IMDB Review Sentiment', 'I love SQL') AS sentiment;
```

=== "Result"

```
                sentiment
-------------------------------------------
[0.06266672909259796, 0.9373332858085632]
(1 row)

Time: 18.101 ms
```

===

This shows that there is a 6.26% chance for category 0 (negative sentiment), and a 93.73% chance it's category 1 (positive sentiment).

See the [task documentation](https://huggingface.co/tasks/text-classification) for more examples, use cases, models and datasets.



## Text Generation

```postgresql
SELECT pgml.load_dataset('bookcorpus', "limit" => 100);

SELECT pgml.tune(
    'GPT Generator',
    task => 'text-generation',
    relation_name => 'pgml.bookcorpus',
    y_column_name => 'text',
    model_name => 'gpt2',
    hyperparams => '{
        "learning_rate": 2e-5,
        "num_train_epochs": 1
    }',
    test_size => 0.2,
    test_sampling => 'last'
);

SELECT pgml.generate('GPT Generator', 'While I wandered weak and weary');
```
