-- src/api.rs:72
-- pgml::api::debug_info
CREATE  FUNCTION pgml."debug_info"() RETURNS void
STRICT
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'debug_info_wrapper';
