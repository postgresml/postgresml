/////////////////////////////
// CREATE TABLE QUERIES /////
/////////////////////////////
pub const CREATE_COLLECTIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS pgml.collections (
  id serial8 PRIMARY KEY, 
  created_at timestamptz NOT NULL DEFAULT now(), 
  name text NOT NULL, 
  active BOOLEAN DEFAULT TRUE, 
  UNIQUE (name)
);
"#;

pub const CREATE_DOCUMENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS %s (
  id serial8 PRIMARY KEY, 
  created_at timestamptz NOT NULL DEFAULT now(), 
  source_uuid uuid NOT NULL, 
  metadata jsonb NOT NULL DEFAULT '{}', 
  text text NOT NULL, 
  UNIQUE (source_uuid)
);
"#;

pub const CREATE_SPLITTERS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS %s (
  id serial8 PRIMARY KEY, created_at timestamptz NOT NULL DEFAULT now(), 
  name text NOT NULL, parameters jsonb NOT NULL DEFAULT '{}'
);
"#;

pub const CREATE_MODELS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS %s (
  id serial8 PRIMARY KEY, created_at timestamptz NOT NULL DEFAULT now(), 
  task text NOT NULL, name text NOT NULL, 
  parameters jsonb NOT NULL DEFAULT '{}'
);
"#;

pub const CREATE_TRANSFORMS_TABLE: &str = r#"CREATE TABLE IF NOT EXISTS %s (
  table_name text PRIMARY KEY, 
  created_at timestamptz NOT NULL DEFAULT now(), 
  task text NOT NULL, 
  splitter_id int8 NOT NULL REFERENCES %s ON DELETE CASCADE ON UPDATE CASCADE DEFERRABLE INITIALLY DEFERRED, 
  model_id int8 NOT NULL REFERENCES %s ON DELETE CASCADE ON UPDATE CASCADE DEFERRABLE INITIALLY DEFERRED, 
  UNIQUE (task, splitter_id, model_id)
);
"#;

pub const CREATE_CHUNKS_TABLE: &str = r#"CREATE TABLE IF NOT EXISTS %s (
  id serial8 PRIMARY KEY, created_at timestamptz NOT NULL DEFAULT now(), 
  document_id int8 NOT NULL REFERENCES %s ON DELETE CASCADE ON UPDATE CASCADE DEFERRABLE INITIALLY DEFERRED, 
  splitter_id int8 NOT NULL REFERENCES %s ON DELETE CASCADE ON UPDATE CASCADE DEFERRABLE INITIALLY DEFERRED, 
  chunk_index int8 NOT NULL, chunk text NOT NULL
);
"#;

pub const CREATE_EMBEDDINGS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS %s (
  id serial8 PRIMARY KEY, 
  created_at timestamptz NOT NULL DEFAULT now(), 
  chunk_id int8 NOT NULL REFERENCES %s ON DELETE CASCADE ON UPDATE CASCADE DEFERRABLE INITIALLY DEFERRED, 
  embedding vector(%d) NOT NULL
);
"#;

/////////////////////////////
// CREATE INDICES ///////////
/////////////////////////////
pub const CREATE_INDEX: &str = r#"
CREATE INDEX CONCURRENTLY IF NOT EXISTS %s ON %s (%d);
"#;

pub const CREATE_INDEX_USING_GIN: &str = r#"
CREATE INDEX CONCURRENTLY IF NOT EXISTS %s ON %s USING GIN (%d);
"#;

pub const CREATE_INDEX_USING_IVFFLAT: &str = r#"
CREATE INDEX CONCURRENTLY IF NOT EXISTS %s ON %s USING ivfflat (%d);
"#;

/////////////////////////////
// Other Big Queries ////////
/////////////////////////////
pub const GENERATE_EMBEDDINGS: &str = r#"
WITH model as (
  SELECT 
    name, 
    parameters 
  from 
    %s 
  where 
    id = $1 
) INSERT INTO %s (chunk_id, embedding) 
SELECT 
  id, 
  pgml.embed(
    text => chunk, 
    transformer => (
      SELECT 
        name 
      FROM 
        model
    ), 
    kwargs => (
      SELECT 
        parameters 
      FROM 
        model
    )
  ) 
FROM 
  %s 
WHERE 
  splitter_id = $2 
  AND id NOT IN (
    SELECT 
      chunk_id 
    from 
      %s
  );
"#;

pub const VECTOR_SEARCH: &str = r#"
WITH query_cte AS (
  SELECT 
    pgml.embed(
      transformer => models.name, 
      text => $1, 
      kwargs => $2 
    )::vector AS query_embedding from %s models where models.id = $3
), 
cte AS (
  SELECT 
    chunk_id, 
    1 - (
      %s.embedding <=> (SELECT query_embedding FROM query_cte)
    ) AS score 
  FROM 
    %s 
) 
SELECT 
  cte.score, 
  chunks.chunk, 
  documents.metadata 
FROM 
  cte 
  INNER JOIN %s chunks ON chunks.id = cte.chunk_id 
  INNER JOIN %s documents ON documents.id = chunks.document_id 
  ORDER BY 
  cte.score DESC 
  LIMIT 
  $4;
"#;

pub const GENERATE_CHUNKS: &str = r#"
WITH splitter as (
  SELECT 
    id, 
    name, 
    parameters 
  FROM 
    %s 
  WHERE 
    id = $1 
) INSERT INTO %s(
  document_id, splitter_id, chunk_index, 
  chunk
) 
SELECT 
  document_id, 
  (
    SELECT 
      id 
    FROM 
      splitter
  ), 
  (chunk).chunk_index, 
  (chunk).chunk 
FROM 
  (
    select 
      id AS document_id, 
      pgml.chunk(
        (
          SELECT 
            name 
          FROM 
            splitter
        ), 
        text, 
        (
          SELECT 
            parameters 
          FROM 
            splitter
        )
      ) AS chunk 
    FROM 
      (
        select 
          id, 
          text 
        from 
          %s 
        WHERE 
          id NOT IN (
            select 
              document_id 
            from 
              %s 
            where 
              splitter_id = (
                select 
                  id 
                from 
                  splitter
              )
          )
      ) as documents
  ) chunks
"#;
