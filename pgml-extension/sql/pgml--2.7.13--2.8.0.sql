-- src/api.rs:691
-- pgml::api::transform_stream
CREATE OR REPLACE FUNCTION pgml."transform_stream"(
	"task" TEXT, /* alloc::string::String */
	"args" jsonb DEFAULT '{}', /* pgrx::datum::json::JsonB */
	"input" TEXT DEFAULT '', /* &str */
	"cache" bool DEFAULT false /* bool */
) RETURNS SETOF TEXT /* alloc::string::String */
IMMUTABLE STRICT PARALLEL SAFE 
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'transform_stream_string_wrapper';

-- src/api.rs:674
-- pgml::api::transform_stream
CREATE OR REPLACE FUNCTION pgml."transform_stream"(
	"task" jsonb, /* pgrx::datum::json::JsonB */
	"args" jsonb DEFAULT '{}', /* pgrx::datum::json::JsonB */
	"input" TEXT DEFAULT '', /* &str */
	"cache" bool DEFAULT false /* bool */
) RETURNS SETOF TEXT /* alloc::string::String */
IMMUTABLE STRICT PARALLEL SAFE 
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'transform_stream_json_wrapper';
