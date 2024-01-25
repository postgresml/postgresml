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
  sdk_version text,
  UNIQUE (name)
);
"#;

pub const PIPELINES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS %s (
  id serial8 PRIMARY KEY,
  name text NOT NULL,
  created_at timestamp NOT NULL DEFAULT now(), 
  active BOOLEAN NOT NULL DEFAULT TRUE,
  schema jsonb NOT NULL,
  UNIQUE (name)
);
"#;

pub const CREATE_DOCUMENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS %s (
  id serial8 PRIMARY KEY,
  created_at timestamp NOT NULL DEFAULT now(),
  source_uuid uuid NOT NULL,
  document jsonb NOT NULL,
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
  chunk_index int8 NOT NULL, 
  chunk text NOT NULL,
  UNIQUE (document_id, chunk_index)
);
"#;

pub const CREATE_EMBEDDINGS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS %s (
  id serial8 PRIMARY KEY, 
  created_at timestamp NOT NULL DEFAULT now(), 
  chunk_id int8 NOT NULL REFERENCES %s ON DELETE CASCADE ON UPDATE CASCADE DEFERRABLE INITIALLY DEFERRED, 
  document_id int8 NOT NULL REFERENCES %s,
  embedding vector(%d) NOT NULL,
  UNIQUE (chunk_id)
);
"#;

pub const CREATE_CHUNKS_TSVECTORS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS %s (
  id serial8 PRIMARY KEY, 
  created_at timestamp NOT NULL DEFAULT now(), 
  chunk_id int8 NOT NULL REFERENCES %s ON DELETE CASCADE ON UPDATE CASCADE DEFERRABLE INITIALLY DEFERRED, 
  document_id int8 NOT NULL REFERENCES %s, 
  ts tsvector,
  UNIQUE (chunk_id)
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

pub const CREATE_INDEX_USING_HNSW: &str = r#"
CREATE INDEX %d IF NOT EXISTS %s on %s using hnsw (%d) %d;
"#;

/////////////////////////////
// Other Big Queries ////////
/////////////////////////////
pub const GENERATE_TSVECTORS_FOR_CHUNK_IDS: &str = r#"
INSERT INTO %s (chunk_id, document_id, ts) 
SELECT 
  id, 
  document_id,
  to_tsvector('%d', chunk) ts 
FROM 
  %s
WHERE id = ANY ($1)
ON CONFLICT (chunk_id) DO UPDATE SET ts = EXCLUDED.ts;
"#;

pub const GENERATE_TSVECTORS: &str = r#"
INSERT INTO %s (chunk_id, document_id, ts) 
SELECT 
  id, 
  document_id,
  to_tsvector('%d', chunk) ts 
FROM 
  %s
WHERE 
  id NOT IN (
    SELECT 
      chunk_id 
    FROM 
      %s
  )
ON CONFLICT (chunk_id) DO NOTHING;
"#;

pub const GENERATE_EMBEDDINGS_FOR_CHUNK_IDS: &str = r#"
INSERT INTO %s (chunk_id, document_id, embedding) 
SELECT 
  id, 
  document_id,
  pgml.embed(
    text => chunk, 
    transformer => $1, 
    kwargs => $2 
  ) 
FROM 
  %s 
WHERE 
  id = ANY ($3)
ON CONFLICT (chunk_id) DO UPDATE SET embedding = EXCLUDED.embedding
"#;

pub const GENERATE_EMBEDDINGS: &str = r#"
INSERT INTO %s (chunk_id, document_id, embedding) 
SELECT 
  id, 
  document_id,
  pgml.embed(
    text => chunk, 
    transformer => $1, 
    kwargs => $2 
  ) 
FROM 
  %s 
WHERE 
  id NOT IN (
    SELECT 
      chunk_id 
    FROM 
      %s
  )
ON CONFLICT (chunk_id) DO NOTHING;
"#;

pub const GENERATE_CHUNKS: &str = r#"
WITH splitter as (
    SELECT
      name,
      parameters
    FROM
      pgml.splitters 
    WHERE
      id = $1
)
INSERT INTO %s(
  document_id, chunk_index, chunk
) 
SELECT 
  document_id, 
  (chunk).chunk_index, 
  (chunk).chunk 
FROM 
  (
    select 
      id AS document_id, 
      pgml.chunk(
        (SELECT name FROM splitter), 
        text, 
        (SELECT parameters FROM splitter)
      ) AS chunk 
    FROM 
      (
        SELECT 
          id, 
          %d as text 
        FROM 
          %s 
        WHERE 
          id NOT IN (
            SELECT 
              document_id 
            FROM 
              %s 
          )
      ) AS documents
  ) chunks 
ON CONFLICT (document_id, chunk_index) DO NOTHING 
"#;

pub const GENERATE_CHUNKS_FOR_DOCUMENT_IDS: &str = r#"
WITH splitter as (
    SELECT
      name,
      parameters
    FROM
      pgml.splitters 
    WHERE
      id = $1
)
INSERT INTO %s(
  document_id, chunk_index, chunk
)
SELECT 
  document_id, 
  (chunk).chunk_index, 
  (chunk).chunk 
FROM 
  (
    SELECT 
      id AS document_id, 
      pgml.chunk(
        (SELECT name FROM splitter), 
        %d, 
        (SELECT parameters FROM splitter)
      ) AS chunk 
    FROM 
      %s WHERE id = ANY($2)
  ) chunks
ON CONFLICT (document_id, chunk_index) DO UPDATE SET chunk = EXCLUDED.chunk 
RETURNING id
"#;
