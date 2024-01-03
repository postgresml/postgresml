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
