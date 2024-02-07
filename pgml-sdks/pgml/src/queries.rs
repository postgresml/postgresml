/////////////////////////////
// CREATE TABLE QUERIES /////
/////////////////////////////
pub const CREATE_COLLECTIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS pgml.collections (
  id serial8 PRIMARY KEY, 
  created_at timestamp NOT NULL DEFAULT now(), 
  name text NOT NULL, 
  active BOOLEAN DEFAULT TRUE, 
  project_id int8 NOT NULL REFERENCES pgml.projects ON DELETE CASCADE ON UPDATE CASCADE DEFERRABLE INITIALLY DEFERRED,
  UNIQUE (name)
);
"#;

pub const CREATE_PIPELINES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS %s (
  id serial8 PRIMARY KEY,
  name text NOT NULL,
  created_at timestamp NOT NULL DEFAULT now(), 
  model_id int8 NOT NULL REFERENCES pgml.models ON DELETE CASCADE ON UPDATE CASCADE DEFERRABLE INITIALLY DEFERRED,
  splitter_id int8 NOT NULL REFERENCES pgml.splitters ON DELETE CASCADE ON UPDATE CASCADE DEFERRABLE INITIALLY DEFERRED,
  active BOOLEAN NOT NULL DEFAULT TRUE,
  parameters jsonb NOT NULL DEFAULT '{}',
  UNIQUE (name)
);
"#;

pub const CREATE_DOCUMENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS %s (
  id serial8 PRIMARY KEY,
  created_at timestamp NOT NULL DEFAULT now(),
  source_uuid uuid NOT NULL,
  metadata jsonb NOT NULL DEFAULT '{}',
  text text NOT NULL,
  UNIQUE (source_uuid)
);
"#;

pub const CREATE_SPLITTERS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS pgml.splitters (
  id serial8 PRIMARY KEY,
  created_at timestamp NOT NULL DEFAULT now(),
  name text NOT NULL, 
  parameters jsonb NOT NULL DEFAULT '{}',
  project_id int8 NOT NULL REFERENCES pgml.projects ON DELETE CASCADE ON UPDATE CASCADE DEFERRABLE INITIALLY DEFERRED
);
"#;

pub const CREATE_CHUNKS_TABLE: &str = r#"CREATE TABLE IF NOT EXISTS %s (
  id serial8 PRIMARY KEY, created_at timestamp NOT NULL DEFAULT now(), 
  document_id int8 NOT NULL REFERENCES %s ON DELETE CASCADE ON UPDATE CASCADE DEFERRABLE INITIALLY DEFERRED, 
  splitter_id int8 NOT NULL REFERENCES pgml.splitters ON DELETE CASCADE ON UPDATE CASCADE DEFERRABLE INITIALLY DEFERRED, 
  chunk_index int8 NOT NULL, 
  chunk text NOT NULL,
  UNIQUE (document_id, splitter_id, chunk_index)
);
"#;

pub const CREATE_EMBEDDINGS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS %s (
  id serial8 PRIMARY KEY, 
  created_at timestamp NOT NULL DEFAULT now(), 
  chunk_id int8 NOT NULL REFERENCES %s ON DELETE CASCADE ON UPDATE CASCADE DEFERRABLE INITIALLY DEFERRED, 
  embedding vector(%d) NOT NULL,
  UNIQUE (chunk_id)
);
"#;

pub const CREATE_DOCUMENTS_TSVECTORS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS %s (
  id serial8 PRIMARY KEY, 
  created_at timestamp NOT NULL DEFAULT now(), 
  document_id int8 NOT NULL REFERENCES %s ON DELETE CASCADE ON UPDATE CASCADE DEFERRABLE INITIALLY DEFERRED, 
  configuration text NOT NULL, 
  ts tsvector,
  UNIQUE (configuration, document_id)
);
"#;

/////////////////////////////
// CREATE INDICES ///////////
/////////////////////////////
pub const CREATE_INDEX: &str = r#"
CREATE INDEX %d IF NOT EXISTS %s ON %s (%d);
"#;

pub const CREATE_INDEX_USING_GIN: &str = r#"
CREATE INDEX %d IF NOT EXISTS %s ON %s USING GIN (%d);
"#;

pub const CREATE_INDEX_USING_IVFFLAT: &str = r#"
CREATE INDEX %d IF NOT EXISTS %s ON %s USING ivfflat (%d);
"#;

/////////////////////////////
// Other Big Queries ////////
/////////////////////////////
pub const GENERATE_TSVECTORS: &str = r#"
INSERT INTO %s (document_id, configuration, ts) 
SELECT 
  id, 
  '%d' configuration, 
  to_tsvector('%d', text) ts 
FROM 
  %s
ON CONFLICT (document_id, configuration) DO UPDATE SET ts = EXCLUDED.ts;
"#;

pub const GENERATE_TSVECTORS_FOR_DOCUMENT_IDS: &str = r#"
INSERT INTO %s (document_id, configuration, ts) 
SELECT 
  id, 
  '%d' configuration, 
  to_tsvector('%d', text) ts 
FROM 
  %s
WHERE id = ANY ($1)
ON CONFLICT (document_id, configuration) DO NOTHING;
"#;

pub const GENERATE_EMBEDDINGS: &str = r#"
INSERT INTO %s (chunk_id, embedding) 
SELECT 
  chunks.id,
  pgml.embed(
    text => chunk, 
    transformer => $1, 
    kwargs => $2 
  ) 
FROM %s AS chunks
LEFT JOIN %s AS embeddings ON embeddings.chunk_id = chunks.id
WHERE splitter_id = $3
  AND embeddings.id IS NULL
ON CONFLICT (chunk_id) DO NOTHING;
"#;

pub const GENERATE_EMBEDDINGS_FOR_CHUNK_IDS: &str = r#"
INSERT INTO %s (chunk_id, embedding) 
SELECT 
  chunks.id,
  pgml.embed(
    text => chunk, 
    transformer => $1, 
    kwargs => $2 
  ) 
FROM %s AS chunks
LEFT JOIN %s AS embeddings ON embeddings.chunk_id = chunks.id
WHERE splitter_id = $3
  AND chunks.id = ANY ($4)
  AND embeddings.id IS NULL
ON CONFLICT (chunk_id) DO NOTHING;
"#;

pub const EMBED_AND_VECTOR_SEARCH: &str = r#"
WITH pipeline AS (
    SELECT
      model_id
    FROM
      %s
    WHERE
      name = $1
),
model AS (
    SELECT
      hyperparams 
    FROM
      pgml.models 
    WHERE
      id = (SELECT model_id FROM pipeline)
),
embedding AS (
  SELECT 
    pgml.embed(
      transformer => (SELECT hyperparams->>'name' FROM model),
      text => $2,
      kwargs => $3
    )::vector AS embedding
), 
comparison AS (
  SELECT 
    chunk_id, 
    1 - (
      %s.embedding <=> (SELECT embedding FROM embedding)
    ) AS score 
  FROM 
    %s 
  ORDER BY
    %s.embedding <=> (SELECT embedding FROM embedding)
  LIMIT $4
) 
SELECT 
  comparison.score, 
  chunks.chunk, 
  documents.metadata 
FROM 
  comparison 
  INNER JOIN %s chunks ON chunks.id = comparison.chunk_id 
  INNER JOIN %s documents ON documents.id = chunks.document_id 
  ORDER BY 
  comparison.score DESC 
"#;

pub const VECTOR_SEARCH: &str = r#"
SELECT 
  1 - (embeddings.embedding <=> $1::vector) score,
  chunks.chunk, 
  documents.metadata 
FROM 
  %s embeddings
INNER JOIN %s chunks ON chunks.id = embeddings.chunk_id 
INNER JOIN %s documents ON documents.id = chunks.document_id 
ORDER BY 
  embeddings.embedding <=> $1::vector ASC 
LIMIT 
  $2;
"#;

pub const GENERATE_CHUNKS: &str = r#"
WITH splitter as (
    SELECT name, parameters
    FROM pgml.splitters
    WHERE id = $1
), chunks AS (
    SELECT
      documents.id AS document_id,
      pgml.chunk(
        (SELECT name FROM splitter),
        text,
        (SELECT parameters FROM splitter)
      ) AS chunk
    FROM %s AS documents
    LEFT OUTER JOIN %s AS chunks ON chunks.splitter_id = $1
      AND chunks.document_id = documents.id
    WHERE chunks.id IS NULL
)
INSERT INTO %s(document_id, splitter_id, chunk_index, chunk)
SELECT document_id, $1, (chunk).chunk_index, (chunk).chunk
FROM chunks
ON CONFLICT (document_id, splitter_id, chunk_index) DO NOTHING
RETURNING id
"#;

pub const GENERATE_CHUNKS_FOR_DOCUMENT_IDS: &str = r#"
WITH splitter as (
    SELECT name, parameters
    FROM pgml.splitters
    WHERE id = $1
), chunks AS (
    SELECT
      documents.id AS document_id,
      pgml.chunk(
        (SELECT name FROM splitter),
        text,
        (SELECT parameters FROM splitter)
      ) AS chunk
    FROM %s AS documents
    LEFT OUTER JOIN %s AS chunks ON chunks.splitter_id = $1
      AND chunks.document_id = documents.id
    WHERE chunks.id IS NULL
      AND documents.id = any($2)
)
INSERT INTO %s(document_id, splitter_id, chunk_index, chunk)
SELECT document_id, $1, (chunk).chunk_index, (chunk).chunk
FROM chunks
ON CONFLICT (document_id, splitter_id, chunk_index) DO NOTHING
RETURNING id
"#;
