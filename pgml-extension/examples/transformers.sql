-- Exit on error (psql)
-- \set ON_ERROR_STOP true
\timing on

SELECT pgml.embed('Alibaba-NLP/gte-base-en-v1.5', 'hi mom', '{"trust_remote_code": true }');
SELECT pgml.embed('Alibaba-NLP/gte-base-en-v1.5', 'hi mom', '{"trust_remote_code": true, "device": "cuda"}');
SELECT pgml.embed('Alibaba-NLP/gte-base-en-v1.5', 'hi mom', '{"trust_remote_code": true, "device": "cpu"}');
SELECT pgml.embed('hkunlp/instructor-xl', 'hi mom', '{"instruction": "Encode it with love"}');
SELECT pgml.embed('mixedbread-ai/mxbai-embed-large-v1', 'test', '{"prompt": "test prompt: "}');

SELECT pgml.transform(
  task   => '{
    "task": "text-generation",
    "model": "meta-llama/Meta-Llama-3.1-8B-Instruct",
    "token": "hf_123"
  }'::JSONB,
  inputs => ARRAY['AI is going to'],
  args   => '{
    "max_new_tokens": 100,
    "trust_remote_code": true
  }'::JSONB
);

-- BitsAndBytes support
SELECT pgml.transform(
    task => '{
      "task": "text-generation",
      "model": "bigscience/bloom-1b7",
      "device_map": "auto",
      "load_in_4bit": true
    }'::JSONB,
    inputs => ARRAY[
        'Once upon a time,'
    ],
    args => '{"max_new_tokens": 32}'::JSONB
);

SELECT pgml.transform(
    task => '{
       "task": "text-generation",
       "model": "TheBloke/MPT-7B-Storywriter-GGML",
       "model_file": "mpt-7b-storywriter.ggmlv3.q8_0.bin"
     }'::JSONB,
     inputs => ARRAY[
    'Once upon a time,'
     ],
     args => '{"max_new_tokens": 32}'::JSONB
);

-- GGML model support
SELECT pgml.transform(
    task => '{
      "task": "text-generation",
      "model": "marella/gpt-2-ggml"
    }'::JSONB,
    inputs => ARRAY[
        'Once upon a time,',
        'This is the beginning'
    ],
    args => '{"max_new_tokens": 32}'::JSONB
);

-- GPTQ model support
SELECT pgml.transform(
    task => '{
      "task": "text-generation",
      "model": "Qwen/Qwen2.5-7B-Instruct-GPTQ-Int8"
    }'::JSONB,
    inputs => ARRAY[
        'Once upon a time,',
        'This is the beginning'
    ],
    args => '{"max_new_tokens": 32}'::JSONB
);

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
    task => '{
        "task": "summarization",
        "model": "facebook/bart-large-cnn"
    }'::JSONB,
    inputs => ARRAY[
        'Dominic Cobb is the foremost practitioner of the artistic science of extraction, inserting oneself into a subject''s dreams to obtain hidden information without the subject knowing, a concept taught to him by his professor father-in-law, Dr. Stephen Miles. Dom''s associates are Miles'' former students, who Dom requires as he has given up being the dream architect for reasons he won''t disclose. Dom''s primary associate, Arthur, believes it has something to do with Dom''s deceased wife, Mal, who often figures prominently and violently in those dreams, or Dom''s want to "go home" (get back to his own reality, which includes two young children). Dom''s work is generally in corporate espionage. As the subjects don''t want the information to get into the wrong hands, the clients have zero tolerance for failure. Dom is also a wanted man, as many of his past subjects have learned what Dom has done to them. One of those subjects, Mr. Saito, offers Dom a job he can''t refuse: to take the concept one step further into inception, namely planting thoughts into the subject''s dreams without them knowing. Inception can fundamentally alter that person as a being. Saito''s target is Robert Michael Fischer, the heir to an energy business empire, which has the potential to rule the world if continued on the current trajectory. Beyond the complex logistics of the dream architecture of the case and some unknowns concerning Fischer, the biggest obstacles in success for the team become worrying about one aspect of inception which Cobb fails to disclose to the other team members prior to the job, and Cobb''s newest associate Ariadne''s belief that Cobb''s own subconscious, especially as it relates to Mal, may be taking over what happens in the dreams.'
    ]
);

SELECT pgml.transform(
    task => '{"task": "text-classification",
              "model": "finiteautomata/bertweet-base-sentiment-analysis"
             }'::JSONB,
    inputs => ARRAY[
        'I love how amazingly simple ML has become!',
        'I hate doing mundane and thankless tasks. ☹️'
    ]
) AS positivity;

SELECT pgml.transform(
    task   => 'text-classification',
    inputs => ARRAY[
        'I love how amazingly simple ML has become!', 
        'I hate doing mundane and thankless tasks. ☹️'
    ]
) AS positivity;

SELECT pgml.transform(
    inputs => ARRAY[
        'Stocks rallied and the British pound gained.', 
        'Stocks making the biggest moves midday: Nvidia, Palantir and more'
    ],
    task => '{"task": "text-classification", 
              "model": "ProsusAI/finbert"
             }'::JSONB
) AS market_sentiment;

SELECT pgml.transform(
    inputs => ARRAY[
        'I have a problem with my iphone that needs to be resolved asap!!'
    ],
    task => '{"task": "zero-shot-classification", 
              "model": "roberta-large-mnli"
             }'::JSONB,
    args => '{"candidate_labels": ["urgent", "not urgent", "phone", "tablet", "computer"]
             }'::JSONB
) AS zero_shot;

SELECT pgml.transform(
    inputs => ARRAY[
        'Hugging Face is a French company based in New York City.'
    ],
    task => 'token-classification'
);

SELECT pgml.transform(
    'question-answering',
    inputs => ARRAY[
        '{
            "question": "Am I dreaming?",
            "context": "I got a good nights sleep last night and started a simple tutorial over my cup of morning coffee. The capabilities seem unreal, compared to what I came to expect from the simple SQL standard I studied so long ago. The answer is staring me in the face, and I feel the uncanny call from beyond the screen to check the results."
        }'
    ]
) AS answer;

