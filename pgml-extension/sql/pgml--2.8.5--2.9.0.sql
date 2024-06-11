-- src/api.rs:613
-- pgml::api::rank
CREATE  FUNCTION pgml."rank"(
	"transformer" TEXT, /* &str */
	"query" TEXT, /* &str */
	"documents" TEXT[], /* alloc::vec::Vec<&str> */
	"kwargs" jsonb DEFAULT '{}' /* pgrx::datum::json::JsonB */
) RETURNS TABLE (
	"corpus_id" bigint,  /* i64 */
	"score" double precision,  /* f64 */
	"text" TEXT  /* core::option::Option<alloc::string::String> */
)
IMMUTABLE STRICT PARALLEL SAFE 
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'rank_wrapper';
