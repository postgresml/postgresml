
  DROP SCHEMA IF EXISTS "{db_name}_{schema}" CASCADE;

  DROP USER MAPPING IF EXISTS
  FOR CURRENT_USER
  SERVER "{db_name}"
  CASCADE;

  DROP SERVER IF EXISTS "{db_name}" CASCADE;