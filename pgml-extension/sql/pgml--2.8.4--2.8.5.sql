DROP FUNCTION pgml."embed"(TEXT,TEXT[],JSONB);
-- pgml::api::embed
CREATE OR REPLACE FUNCTION pgml."embed"(
	"transformer" TEXT, /* &str */
	"inputs" TEXT[], /* alloc::vec::Vec<&str> */
	"kwargs" jsonb DEFAULT '{}' /* pgrx::datum::json::JsonB */
) RETURNS SETOF real[] /* alloc::vec::Vec<f32> */
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'embed_batch_wrapper';
