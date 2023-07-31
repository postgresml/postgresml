-- src/api.rs:74
-- pgml::api::python_pip_freeze
CREATE FUNCTION pgml."python_pip_freeze"() RETURNS TABLE (
    "package" TEXT  /* alloc::string::String */
)
STRICT
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'python_pip_freeze_wrapper';
