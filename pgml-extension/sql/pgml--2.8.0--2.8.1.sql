-- pgml::api::transform_conversational_json
CREATE FUNCTION pgml."transform"(
	"task" jsonb, /* pgrx::datum::json::JsonB */
	"args" jsonb DEFAULT '{}', /* pgrx::datum::json::JsonB */
	"inputs" jsonb[] DEFAULT ARRAY[]::JSONB[], /* Vec<pgrx::datum::json::JsonB> */
	"cache" bool DEFAULT false /* bool */
) RETURNS jsonb /* alloc::string::String */
IMMUTABLE STRICT PARALLEL SAFE 
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'transform_conversational_json_wrapper';

-- pgml::api::transform_conversational_string
CREATE FUNCTION pgml."transform"(
	"task" TEXT, /* alloc::string::String */
	"args" jsonb DEFAULT '{}', /* pgrx::datum::json::JsonB */
	"inputs" jsonb[] DEFAULT ARRAY[]::JSONB[], /* Vec<pgrx::datum::json::JsonB> */
	"cache" bool DEFAULT false /* bool */
) RETURNS jsonb /* alloc::string::String */
IMMUTABLE STRICT PARALLEL SAFE 
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'transform_conversational_string_wrapper';

-- pgml::api::transform_stream_string
DROP FUNCTION IF EXISTS pgml."transform_stream"(text,jsonb,text,boolean);
CREATE FUNCTION pgml."transform_stream"(
	"task" TEXT, /* alloc::string::String */
	"args" jsonb DEFAULT '{}', /* pgrx::datum::json::JsonB */
	"input" TEXT DEFAULT '', /* &str */
	"cache" bool DEFAULT false /* bool */
) RETURNS SETOF jsonb /* pgrx::datum::json::JsonB */
IMMUTABLE STRICT PARALLEL SAFE 
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'transform_stream_string_wrapper';

-- pgml::api::transform_stream_json
DROP FUNCTION IF EXISTS pgml."transform_stream"(jsonb,jsonb,text,boolean);
CREATE FUNCTION pgml."transform_stream"(
	"task" jsonb, /* pgrx::datum::json::JsonB */
	"args" jsonb DEFAULT '{}', /* pgrx::datum::json::JsonB */
	"input" TEXT DEFAULT '', /* &str */
	"cache" bool DEFAULT false /* bool */
) RETURNS SETOF jsonb /* pgrx::datum::json::JsonB */
IMMUTABLE STRICT PARALLEL SAFE 
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'transform_stream_json_wrapper';

-- pgml::api::transform_stream_conversational_json
CREATE FUNCTION pgml."transform_stream"(
	"task" TEXT, /* alloc::string::String */
	"args" jsonb DEFAULT '{}', /* pgrx::datum::json::JsonB */
	"inputs" jsonb[] DEFAULT ARRAY[]::JSONB[], /* Vec<pgrx::datum::json::JsonB> */
	"cache" bool DEFAULT false /* bool */
) RETURNS SETOF jsonb /* pgrx::datum::json::JsonB */
IMMUTABLE STRICT PARALLEL SAFE 
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'transform_stream_conversational_string_wrapper';

-- pgml::api::transform_stream_conversational_string
CREATE FUNCTION pgml."transform_stream"(
	"task" jsonb, /* pgrx::datum::json::JsonB */
	"args" jsonb DEFAULT '{}', /* pgrx::datum::json::JsonB */
	"inputs" jsonb[] DEFAULT ARRAY[]::JSONB[], /* Vec<pgrx::datum::json::JsonB> */
	"cache" bool DEFAULT false /* bool */
) RETURNS SETOF jsonb /* pgrx::datum::json::JsonB */
IMMUTABLE STRICT PARALLEL SAFE 
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'transform_stream_conversational_json_wrapper';
