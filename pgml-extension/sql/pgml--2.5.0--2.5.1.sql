-- src/api.rs:575
-- pgml::api::embed
CREATE  FUNCTION pgml."embed"(
	"transformer" TEXT, /* &str */
	"inputs" TEXT[], /* alloc::vec::Vec<&str> */
	"kwargs" jsonb DEFAULT '{}' /* pgrx::datum::json::JsonB */
) RETURNS real[][] /* alloc::vec::Vec<alloc::vec::Vec<f32>> */
IMMUTABLE STRICT PARALLEL SAFE 
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'embed_batch_wrapper';

-- src/api.rs:584
-- pgml::api::chunk
CREATE  FUNCTION pgml."chunk"(
	"splitter" TEXT, /* &str */
	"text" TEXT, /* &str */
	"kwargs" jsonb DEFAULT '{}' /* pgrx::datum::json::JsonB */
) RETURNS TABLE (
	"chunk_index" bigint,  /* i64 */
	"chunk" TEXT  /* alloc::string::String */
)
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'chunk_wrapper';
