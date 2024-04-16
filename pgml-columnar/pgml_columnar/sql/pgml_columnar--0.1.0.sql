CREATE FUNCTION pgml_columnar_table_am_handler(arg internal) RETURNS table_am_handler
	LANGUAGE C STRICT
	AS 'MODULE_PATHNAME', 'pgml_columnar_table_am';

CREATE ACCESS METHOD pgml_columnar TYPE TABLE HANDLER pgml_columnar_table_am_handler;
