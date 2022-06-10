SELECT pgml.transform(
    'summarization',
    inputs => ARRAY[
        'In the twenty-third century, the universe is threatened by evil. The only hope for mankind is the Fifth Element, who comes to Earth every five thousand years to protect the humans with four stones of the four elements: fire, water, Earth and air. A Mondoshawan spacecraft is bringing The Fifth Element back to Earth but it is destroyed by the evil Mangalores. However, a team of scientists use the DNA of the remains of the Fifth Element to rebuild the perfect being called Leeloo. She escapes from the laboratory and stumbles upon the taxi driver and former elite commando Major Korben Dallas that helps her to escape from the police. Leeloo tells him that she must meet Father Vito Cornelius to accomplish her mission. Meanwhile, the Evil uses the greedy and cruel Jean-Baptiste Emanuel Zorg and a team of mercenary Mangalores to retrieve the stones and avoid the protection of Leeloo. But the skilled Korben Dallas has fallen in love with Leeloo and decides to help her to retrieve the stones.'
    ]
) AS result;

SELECT pgml.transform(
    'translation_en_to_fr',
    inputs => ARRAY[
        'Welcome to the future!',
        'Where have you been all this time?'
    ]
) AS result;

SELECT pgml.transform(
    '{"model": "roberta-large-mnli"}'::JSONB,
    inputs => ARRAY[
        'I love how amazingly simple ML has become!', 
        'Some models are painfully slow and expensive ☹️'
    ]
) AS result;

SELECT pgml.transform(
    'question-answering',
    inputs => ARRAY[
        '{"question": "Am I dreaming?", "context": "I got a good nights sleep last night, and started a simple tutorial over my cup of morning coffee. The capabilities seem unreal, compared to what I came to expect from the simple SQL standard I studied so long ago. The answer is staring me in the face, and I feel the uncanny call from beyond the screen calling me to check the results."}'
    ]
) AS result;

SELECT pgml.transform(
    '{"model": "roberta-large-mnli"}'::JSONB,
    inputs => ARRAY[
        'I love how amazingly simple ML has become!', 
        'I hate doing mundane and thankless tasks. ☹️'
    ]
) AS result;
SELECT pgml.transform(
    '{"model": "roberta-large-mnli"}'::JSONB,
    inputs => ARRAY[
        'I love how amazingly simple ML has become!', 
        'I hate doing mundane and thankless tasks. ☹️'
    ]
) AS result;
SELECT pgml.transform(
    '{"model": "roberta-large-mnli"}'::JSONB,
    inputs => ARRAY[
        'I love how amazingly simple ML has become!', 
        'I hate doing mundane and thankless tasks. ☹️'
    ]
) AS result;

SELECT pgml.transform(
    'summarization',
    inputs => ARRAY['
        Dominic Cobb is the foremost practitioner of the artistic science of extraction, inserting oneself into a subject''s dreams to obtain hidden information without the subject knowing, a concept taught to him by his professor father-in-law, Dr. Stephen Miles. Dom''s associates are Miles'' former students, who Dom requires as he has given up being the dream architect for reasons he won''t disclose. Dom''s primary associate, Arthur, believes it has something to do with Dom''s deceased wife, Mal, who often figures prominently and violently in those dreams, or Dom''s want to "go home" (get back to his own reality, which includes two young children). Dom''s work is generally in corporate espionage. As the subjects don''t want the information to get into the wrong hands, the clients have zero tolerance for failure. Dom is also a wanted man, as many of his past subjects have learned what Dom has done to them. One of those subjects, Mr. Saito, offers Dom a job he can''t refuse: to take the concept one step further into inception, namely planting thoughts into the subject''s dreams without them knowing. Inception can fundamentally alter that person as a being. Saito''s target is Robert Michael Fischer, the heir to an energy business empire, which has the potential to rule the world if continued on the current trajectory. Beyond the complex logistics of the dream architecture of the case and some unknowns concerning Fischer, the biggest obstacles in success for the team become worrying about one aspect of inception which Cobb fails to disclose to the other team members prior to the job, and Cobb''s newest associate Ariadne''s belief that Cobb''s own subconscious, especially as it relates to Mal, may be taking over what happens in the dreams.
    ']);

SELECT pgml.load_dataset('opus_books', 'en-fr');

SELECT pgml.load_dataset('kde4', kwargs => '{"lang1": "en", "lang2": "es"}');
SELECT pgml.tune(
    'Translate English to Spanish',
    task => 'translation_en_to_es',
    relation_name => 'pgml.kde4',
    y_column_name => 'translation',
    model_name => 'Helsinki-NLP/opus-mt-en-es',
    hyperparams => '{
        "learning_rate": 2e-5,
        "per_device_train_batch_size": 16,
        "per_device_eval_batch_size": 16,
        "num_train_epochs": 1,
        "weight_decay": 0.01,
        "max_length": 128
    }',
    test_size => 0.05,
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
