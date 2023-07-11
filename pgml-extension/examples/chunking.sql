--- Chunk text for  LLM embeddings and vectorization.

DROP TABLE IF EXISTS documents CASCADE;
CREATE TABLE documents (
	id BIGSERIAL PRIMARY KEY,
	document TEXT NOT NULL,
	created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

DROP TABLE IF EXISTS splitters CASCADE;
CREATE TABLE splitters (
	id BIGSERIAL PRIMARY KEY,
	splitter VARCHAR NOT NULL DEFAULT 'recursive_character'
);

DROP TABLE IF EXISTS document_chunks CASCADE;
CREATE TABLE document_chunks(
	id BIGSERIAL PRIMARY KEY,
	document_id BIGINT NOT NULL REFERENCES documents(id),
	splitter_id BIGINT NOT NULL REFERENCES splitters(id),
	chunk_index BIGINT NOT NULL,
	chunk VARCHAR
);

INSERT INTO documents VALUES (
	1,
	'It was the best of times, it was the worst of times, it was the age of wisdom, 
	it was the age of foolishness, it was the epoch of belief, it was the epoch of incredulity, it was the season of Light, 
	it was the season of Darkness, it was the spring of hope, it was the winter of despair, we had everything before us, 
	we had nothing before us, we were all going direct to Heaven, we were all going direct the other way—in short, the period was so far like
	the present period, that some of its noisiest authorities insisted on its being received, for good or for evil, in the superlative degree of comparison only.',
	NOW()
);

INSERT INTO splitters VALUES (1, 'recursive_character');

WITH document AS (
	SELECT id, document
	FROM documents
	WHERE id = 1
),

splitter AS (
	SELECT id, splitter
	FROM splitters
	WHERE id = 1
)

INSERT INTO document_chunks SELECT
	nextval('document_chunks_id_seq'::regclass),
	(SELECT id FROM document),
	(SELECT id FROM splitter),
	chunk_index,
	chunk
FROM
	pgml.chunk(
		(SELECT splitter FROM splitter),
		(SELECT document FROM document),
		'{"chunk_size": 2, "chunk_overlap": 2}'
	);

SELECT * FROM document_chunks LIMIT 5;
