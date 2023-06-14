-- Exit on error (psql)
-- \set ON_ERROR_STOP true
\timing on


SELECT pgml.load_dataset('kde4', kwargs => '{"lang1": "en", "lang2": "es"}');
CREATE OR REPLACE VIEW kde4_en_to_es AS
SELECT translation->>'en' AS "en", translation->>'es' AS "es"
FROM pgml.kde4
LIMIT 10;
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

SELECT pgml.load_dataset('imdb');
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
SELECT pgml.predict('IMDB Review Sentiment', 'I love SQL');

SELECT pgml.load_dataset('squad_v2');
SELECT pgml.tune(
    'SQuAD Q&A v2',
    'question-answering',
    'pgml.squad_v2',
    'answers',
    'deepset/roberta-base-squad2',
    hyperparams => '{
        "evaluation_strategy": "epoch",
        "learning_rate": 2e-5,
        "per_device_train_batch_size": 16,
        "per_device_eval_batch_size": 16,
        "num_train_epochs": 1,
        "weight_decay": 0.01,
        "max_length": 384,
        "stride": 128
    }',
    test_size => 11873,
    test_sampling => 'last'
);


SELECT pgml.load_dataset('billsum', kwargs => '{"split": "ca_test"}');
CREATE OR REPLACE VIEW billsum_training_data
AS SELECT title || '\n' || text AS text, summary FROM pgml.billsum;
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
        "max_input_length": 1024,
        "max_summary_length": 128
    }',
    test_size => 0.01,
    test_sampling => 'last'
);
