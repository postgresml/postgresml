\timing on

SELECT pgml.embed('Alibaba-NLP/gte-base-en-v1.5', 'hi mom', '{"trust_remote_code": true}');
SELECT pgml.embed('Alibaba-NLP/gte-base-en-v1.5', 'hi mom', '{"device": "cuda", "trust_remote_code": true}');
SELECT pgml.embed('Alibaba-NLP/gte-base-en-v1.5', 'hi mom', '{"device": "cpu", "trust_remote_code": true}');
SELECT pgml.embed('hkunlp/instructor-xl', 'hi mom', '{"instruction": "Encode it with love"}');
SELECT pgml.embed('mixedbread-ai/mxbai-embed-large-v1', 'test', '{"prompt": "test prompt: "}');
SELECT pgml.embed('sentence-transformers/all-MiniLM-L6-v2', 'hi mom');
