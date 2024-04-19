
  CREATE EXTENSION IF NOT EXISTS postgres_fdw;
  CREATE EXTENSION IF NOT EXISTS dblink;

  CREATE SERVER "{db_name}"
  FOREIGN DATA WRAPPER postgres_fdw
  OPTIONS (
    host '{host}',
    port '{port}',
    dbname '{database_name}'
  );

  CREATE USER MAPPING 
  FOR CURRENT_USER 
  SERVER "{db_name}"
  OPTIONS (
    user '{user}',
    password '{password}'
  );

  SELECT * FROM dblink(
    '{db_name}',
    'SELECT pgml.embed(''intfloat/e5-small'', ''test postgresml embedding'') AS embedding'
  ) AS t(embedding real[386]);

  CREATE FUNCTION pgml_embed_e5_small(text) RETURNS real[386] AS $$
    SELECT * FROM dblink(
      '{db_name}',
      'SELECT pgml.embed(''intfloat/e5-small'', ''' || $1 || ''') AS embedding'
    ) AS t(embedding real[386]);
  $$ LANGUAGE SQL;
