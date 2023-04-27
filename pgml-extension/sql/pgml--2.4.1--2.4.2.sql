CREATE OR REPLACE FUNCTION pgml.activate_venv(venv text)
  RETURNS boolean
  LANGUAGE c
  STRICT
 AS '$libdir/pgml', $function$activate_venv_wrapper$function$;
