
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

  CREATE SCHEMA "{db_name}_{schema}";

  IMPORT FOREIGN SCHEMA "{schema}"
  FROM SERVER "{db_name}"
  INTO "{db_name}_{schema}";
