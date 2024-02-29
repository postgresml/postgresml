-- src/api.rs:317
-- pgml::api::deploy
DROP FUNCTION IF EXISTS pgml."deploy"(BIGINT);
CREATE  FUNCTION pgml."deploy"(
    "model_id" BIGINT /* i64 */
) RETURNS TABLE (
                    "project" TEXT,  /* alloc::string::String */
                    "strategy" TEXT,  /* alloc::string::String */
                    "algorithm" TEXT  /* alloc::string::String */
                )
    LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'deploy_model_wrapper';

DROP FUNCTION IF EXISTS pgml."deploy"(text, pgml.Strategy, pgml.Algorithm);
CREATE  FUNCTION pgml."deploy"(
    "project_name" TEXT, /* &str */
    "strategy" pgml.Strategy, /* pgml::orm::strategy::Strategy */
    "algorithm" pgml.Algorithm DEFAULT NULL /* core::option::Option<pgml::orm::algorithm::Algorithm> */
) RETURNS TABLE (
                    "project" TEXT,  /* alloc::string::String */
                    "strategy" TEXT,  /* alloc::string::String */
                    "algorithm" TEXT  /* alloc::string::String */
                )
    LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'deploy_strategy_wrapper';

ALTER TYPE pgml.strategy ADD VALUE 'specific';

ALTER TYPE pgml.Sampling ADD VALUE 'stratified';

-- src/api.rs:534
-- pgml::api::snapshot
DROP FUNCTION IF EXISTS pgml."snapshot"(text, text, real, pgml.Sampling, jsonb);
CREATE  FUNCTION pgml."snapshot"(
	"relation_name" TEXT, /* &str */
	"y_column_name" TEXT, /* &str */
	"test_size" real DEFAULT 0.25, /* f32 */
	"test_sampling" pgml.Sampling DEFAULT 'stratified', /* pgml::orm::sampling::Sampling */
	"preprocess" jsonb DEFAULT '{}' /* pgrx::datum::json::JsonB */
) RETURNS TABLE (
	"relation" TEXT,  /* alloc::string::String */
	"y_column_name" TEXT  /* alloc::string::String */
)
STRICT
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'snapshot_wrapper';

-- src/api.rs:802
-- pgml::api::tune
DROP FUNCTION IF EXISTS pgml."tune"(text, text, text, text, text, jsonb, real, pgml.Sampling, bool, bool);
CREATE  FUNCTION pgml."tune"(
	"project_name" TEXT, /* &str */
	"task" TEXT DEFAULT NULL, /* core::option::Option<&str> */
	"relation_name" TEXT DEFAULT NULL, /* core::option::Option<&str> */
	"y_column_name" TEXT DEFAULT NULL, /* core::option::Option<&str> */
	"model_name" TEXT DEFAULT NULL, /* core::option::Option<&str> */
	"hyperparams" jsonb DEFAULT '{}', /* pgrx::datum::json::JsonB */
	"test_size" real DEFAULT 0.25, /* f32 */
	"test_sampling" pgml.Sampling DEFAULT 'stratified', /* pgml::orm::sampling::Sampling */
	"automatic_deploy" bool DEFAULT true, /* core::option::Option<bool> */
	"materialize_snapshot" bool DEFAULT false /* bool */
) RETURNS TABLE (
	"status" TEXT,  /* alloc::string::String */
	"task" TEXT,  /* alloc::string::String */
	"algorithm" TEXT,  /* alloc::string::String */
	"deployed" bool  /* bool */
)
PARALLEL SAFE
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'tune_wrapper';

-- src/api.rs:92
-- pgml::api::train
DROP FUNCTION IF EXISTS pgml."train"(text, text, text, text, pgml.Algorithm, jsonb, pgml.Search, jsonb, jsonb, real, pgml.Sampling, pgml.Runtime, bool, bool, jsonb);
CREATE  FUNCTION pgml."train"(
	"project_name" TEXT, /* &str */
	"task" TEXT DEFAULT NULL, /* core::option::Option<&str> */
	"relation_name" TEXT DEFAULT NULL, /* core::option::Option<&str> */
	"y_column_name" TEXT DEFAULT NULL, /* core::option::Option<&str> */
	"algorithm" pgml.Algorithm DEFAULT 'linear', /* pgml::orm::algorithm::Algorithm */
	"hyperparams" jsonb DEFAULT '{}', /* pgrx::datum::json::JsonB */
	"search" pgml.Search DEFAULT NULL, /* core::option::Option<pgml::orm::search::Search> */
	"search_params" jsonb DEFAULT '{}', /* pgrx::datum::json::JsonB */
	"search_args" jsonb DEFAULT '{}', /* pgrx::datum::json::JsonB */
	"test_size" real DEFAULT 0.25, /* f32 */
	"test_sampling" pgml.Sampling DEFAULT 'stratified', /* pgml::orm::sampling::Sampling */
	"runtime" pgml.Runtime DEFAULT NULL, /* core::option::Option<pgml::orm::runtime::Runtime> */
	"automatic_deploy" bool DEFAULT true, /* core::option::Option<bool> */
	"materialize_snapshot" bool DEFAULT false, /* bool */
	"preprocess" jsonb DEFAULT '{}' /* pgrx::datum::json::JsonB */
) RETURNS TABLE (
	"project" TEXT,  /* alloc::string::String */
	"task" TEXT,  /* alloc::string::String */
	"algorithm" TEXT,  /* alloc::string::String */
	"deployed" bool  /* bool */
)
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'train_wrapper';

-- src/api.rs:138
-- pgml::api::train_joint
DROP FUNCTION IF EXISTS pgml."train_joint"(text, text, text, text, pgml.Algorithm, jsonb, pgml.Search, jsonb, jsonb, real, pgml.Sampling, pgml.Runtime, bool, bool, jsonb);
CREATE  FUNCTION pgml."train_joint"(
	"project_name" TEXT, /* &str */
	"task" TEXT DEFAULT NULL, /* core::option::Option<&str> */
	"relation_name" TEXT DEFAULT NULL, /* core::option::Option<&str> */
	"y_column_name" TEXT[] DEFAULT NULL, /* core::option::Option<alloc::vec::Vec<alloc::string::String>> */
	"algorithm" pgml.Algorithm DEFAULT 'linear', /* pgml::orm::algorithm::Algorithm */
	"hyperparams" jsonb DEFAULT '{}', /* pgrx::datum::json::JsonB */
	"search" pgml.Search DEFAULT NULL, /* core::option::Option<pgml::orm::search::Search> */
	"search_params" jsonb DEFAULT '{}', /* pgrx::datum::json::JsonB */
	"search_args" jsonb DEFAULT '{}', /* pgrx::datum::json::JsonB */
	"test_size" real DEFAULT 0.25, /* f32 */
	"test_sampling" pgml.Sampling DEFAULT 'stratified', /* pgml::orm::sampling::Sampling */
	"runtime" pgml.Runtime DEFAULT NULL, /* core::option::Option<pgml::orm::runtime::Runtime> */
	"automatic_deploy" bool DEFAULT true, /* core::option::Option<bool> */
	"materialize_snapshot" bool DEFAULT false, /* bool */
	"preprocess" jsonb DEFAULT '{}' /* pgrx::datum::json::JsonB */
) RETURNS TABLE (
	"project" TEXT,  /* alloc::string::String */
	"task" TEXT,  /* alloc::string::String */
	"algorithm" TEXT,  /* alloc::string::String */
	"deployed" bool  /* bool */
)
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'train_joint_wrapper';
