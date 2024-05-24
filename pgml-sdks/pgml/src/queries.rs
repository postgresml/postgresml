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
  version jsonb NOT NULL DEFAULT '{}'::jsonb,
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
  embedding vector(%d) NOT NULL,
  UNIQUE (chunk_id)
);
"#;

pub const CREATE_CHUNKS_TSVECTORS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS %s (
  id serial8 PRIMARY KEY, 
  created_at timestamp NOT NULL DEFAULT now(), 
  chunk_id int8 NOT NULL REFERENCES %s ON DELETE CASCADE ON UPDATE CASCADE DEFERRABLE INITIALLY DEFERRED, 
  ts tsvector,
  UNIQUE (chunk_id)
);
"#;

pub const CREATE_PIPELINES_SEARCHES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS %s (
  id serial8 PRIMARY KEY,
  created_at timestamp NOT NULL DEFAULT now(), 
  query jsonb
);
"#;

pub const CREATE_PIPELINES_SEARCH_RESULTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS %s (
  id serial8 PRIMARY KEY,
  search_id int8 NOT NULL REFERENCES %s ON DELETE CASCADE,
  document_id int8 NOT NULL REFERENCES %s ON DELETE CASCADE,
  scores jsonb NOT NULL,
  rank integer NOT NULL
);
"#;

pub const CREATE_PIPELINES_SEARCH_EVENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS %s (
  id serial8 PRIMARY KEY,
  created_at timestamp NOT NULL DEFAULT now(), 
  search_result int8 NOT NULL REFERENCES %s ON DELETE CASCADE,
  event jsonb NOT NULL
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
// Inserting Search Events //
/////////////////////////////

// Tag: CRITICAL_QUERY
// Checked: True
// Trigger: Runs whenever a user calls collection.add_search_event
// Required indexes:
// search_results table | "search_results_search_id_rank_index" btree (search_id, rank)
// Used to insert a search event
pub const INSERT_SEARCH_EVENT: &str = r#"
INSERT INTO %s (search_result, event) VALUES ((SELECT id FROM %s WHERE search_id = $1 AND rank = $2), $3)
"#;

/////////////////////////////
// Upserting Documents //////
/////////////////////////////

// Tag: CRITICAL_QUERY
// Checked: True
// Trigger: Runs whenever a user upserts documents
// Required indexes:
// documents table | - "documents_source_uuid_key" UNIQUE CONSTRAINT, btree (source_uuid)
// Used to upsert a document and merge the previous metadata on conflict
// The values of the query and the source_uuid binding are built when used
pub const UPSERT_DOCUMENT_AND_MERGE_METADATA: &str = r#"
WITH prev AS (
  SELECT id, document FROM %s WHERE source_uuid = ANY({binding_parameter})
) INSERT INTO %s (source_uuid, document, version) 
VALUES {values_parameters} 
ON CONFLICT (source_uuid) DO UPDATE SET document = %s.document || EXCLUDED.document, version = EXCLUDED.version 
RETURNING id, (SELECT document FROM prev WHERE prev.id = %s.id) 
"#;

// Tag: CRITICAL_QUERY
// Checked: True
// Trigger: Runs whenever a user upserts documents
// Required indexes:
// - documents table | "documents_source_uuid_key" UNIQUE CONSTRAINT, btree (source_uuid)
// Used to upsert a document and over the previous document on conflict
// The values of the query and the source_uuid binding are built when used
pub const UPSERT_DOCUMENT: &str = r#"
WITH prev AS (
  SELECT id, document FROM %s WHERE source_uuid = ANY({binding_parameter})
) INSERT INTO %s (source_uuid, document, version) 
VALUES {values_parameters} 
ON CONFLICT (source_uuid) DO UPDATE SET document = EXCLUDED.document, version = EXCLUDED.version 
RETURNING id, (SELECT document FROM prev WHERE prev.id = %s.id) 
"#;

/////////////////////////////
// Generaiting TSVectors ////
/////////////////////////////

// Tag: CRITICAL_QUERY
// Checked: True
// Trigger: Runs whenever a pipeline is syncing documents and does full_text_search
// Required indexes:
// - chunks table | "{key}_tsvectors_pkey" PRIMARY KEY, btree (id)
// Used to generate tsvectors for specific chunks
pub const GENERATE_TSVECTORS_FOR_CHUNK_IDS: &str = r#"
INSERT INTO %s (chunk_id, ts) 
SELECT 
  id, 
  to_tsvector('%d', chunk) ts 
FROM 
  %s
WHERE id = ANY ($1)
ON CONFLICT (chunk_id) DO UPDATE SET ts = EXCLUDED.ts;
"#;

// Tag: CRITICAL_QUERY
// Checked: True
// Trigger: Runs whenever a pipeline is resyncing and does full_text_search
// Required indexes: None
// Used to generate tsvectors for an entire collection
pub const GENERATE_TSVECTORS: &str = r#"
INSERT INTO %s (chunk_id, ts) 
SELECT 
  id, 
  to_tsvector('%d', chunk) ts 
FROM 
  %s chunks
ON CONFLICT (chunk_id) DO UPDATE SET ts = EXCLUDED.ts;
"#;

/////////////////////////////
// Generaiting Embeddings ///
/////////////////////////////

// Tag: CRITICAL_QUERY
// Checked: True
// Trigger: Runs whenver a pipeline is syncing documents and does semantic_search
// Required indexes:
// - chunks table | "{key}_chunks_pkey" PRIMARY KEY, btree (id)
// Used to generate embeddings for specific chunks
pub const GENERATE_EMBEDDINGS_FOR_CHUNK_IDS: &str = r#"
INSERT INTO %s (chunk_id, embedding) 
SELECT 
  unnest(array_agg(id)), 
  pgml.embed(
    inputs => array_agg(chunk), 
    transformer => $1, 
    kwargs => $2 
  ) 
FROM 
  %s 
WHERE 
  id = ANY ($3)
ON CONFLICT (chunk_id) DO UPDATE SET embedding = EXCLUDED.embedding
"#;

// Tag: CRITICAL_QUERY
// Checked: True
// Trigger: Runs whenever a pipeline is resyncing and does semantic_search
// Required indexes: None
// Used to generate embeddings for an entire collection
pub const GENERATE_EMBEDDINGS: &str = r#"
INSERT INTO %s (chunk_id, embedding) 
SELECT 
  id, 
  pgml.embed(
    text => chunk, 
    transformer => $1, 
    kwargs => $2 
  ) 
FROM 
  %s 
ON CONFLICT (chunk_id) DO UPDATE set embedding = EXCLUDED.embedding;
"#;

/////////////////////////////
// Generating Chunks ///////
/////////////////////////////

// Tag: CRITICAL_QUERY
// Checked: False
// Used to generate chunks for a specific documents with a splitter
pub const GENERATE_CHUNKS_FOR_DOCUMENT_IDS_WITH_SPLITTER: &str = r#"
WITH splitter AS (
    SELECT
        name,
        parameters
    FROM
        pgml.splitters
    WHERE
        id = $1
),
new AS (
    SELECT
        documents.id AS document_id,
        pgml.chunk ((
            SELECT
                name
            FROM splitter), %d, (
            SELECT
                parameters
            FROM splitter)) AS chunk_t
FROM
    %s AS documents
    WHERE
        id = ANY ($2)
),
del AS (
    DELETE FROM %s chunks
    WHERE chunk_index > (
            SELECT
                MAX((chunk_t).chunk_index)
            FROM
                new
            WHERE
                new.document_id = chunks.document_id
            GROUP BY
                new.document_id)
        AND chunks.document_id = ANY (
            SELECT
                document_id
            FROM
                new))
    INSERT INTO %s (document_id, chunk_index, chunk)
SELECT
    new.document_id,
    (chunk_t).chunk_index,
    (chunk_t).chunk
FROM
    new
    LEFT OUTER JOIN %s chunks ON chunks.document_id = new.document_id
    AND chunks.chunk_index = (chunk_t).chunk_index
WHERE (chunk_t).chunk <> COALESCE(chunks.chunk, '')
ON CONFLICT (document_id, chunk_index)
    DO UPDATE SET
        chunk = EXCLUDED.chunk
RETURNING
    id;
"#;

// Tag: CRITICAL_QUERY
// Checked: True
// Trigger: Runs whenver a pipeline is syncing documents and the key does not have a splitter
// Required indexes:
// - documents table | "documents_pkey" PRIMARY KEY, btree (id)
// - chunks table | "{key}_pipeline_chunk_document_id_index" btree (document_id)
// Used to generate chunks for a specific documents without a splitter
// This query just copies the document key into the chunk
pub const GENERATE_CHUNKS_FOR_DOCUMENT_IDS: &str = r#"
INSERT INTO %s(
    document_id, chunk_index, chunk
)
SELECT 
    documents.id,
    1,
    %d
FROM %s documents
LEFT OUTER JOIN %s chunks ON chunks.document_id = documents.id
WHERE documents.%d <> COALESCE(chunks.chunk, '')
  AND documents.id = ANY($1)
ON CONFLICT (document_id, chunk_index) DO UPDATE SET chunk = EXCLUDED.chunk 
RETURNING id
"#;

// Tag: CRITICAL_QUERY
// Checked: False
// Used to generate chunks for an entire collection with a splitter
pub const GENERATE_CHUNKS_WITH_SPLITTER: &str = r#"
WITH splitter AS (
    SELECT
        name,
        parameters
    FROM
        pgml.splitters
    WHERE
        id = $1
),
new AS (
    SELECT
        documents.id AS document_id,
        pgml.chunk ((
            SELECT
                name
            FROM splitter), %d, (
            SELECT
                parameters
            FROM splitter)) AS chunk_t
FROM
    %s AS documents
),
del AS (
    DELETE FROM %s chunks
    WHERE chunk_index > (
            SELECT
                MAX((chunk_t).chunk_index)
            FROM
                new
            WHERE
                new.document_id = chunks.document_id
            GROUP BY
                new.document_id)
        AND chunks.document_id = ANY (
            SELECT
                document_id
            FROM
                new))
INSERT INTO %s (document_id, chunk_index, chunk)
SELECT
    new.document_id,
    (chunk_t).chunk_index,
    (chunk_t).chunk
FROM
    new
ON CONFLICT (document_id, chunk_index)
    DO UPDATE SET
        chunk = EXCLUDED.chunk;
"#;

// Tag: CRITICAL_QUERY
// Trigger: Runs whenever a pipeline is resyncing
// Required indexes: None
// Checked: True
// Used to generate chunks for an entire collection
pub const GENERATE_CHUNKS: &str = r#"
INSERT INTO %s (
    document_id, chunk_index, chunk
)
SELECT
    id,
    1,
    %d
FROM %s
ON CONFLICT (document_id, chunk_index) DO UPDATE SET chunk = EXCLUDED.chunk
RETURNING id
"#;
