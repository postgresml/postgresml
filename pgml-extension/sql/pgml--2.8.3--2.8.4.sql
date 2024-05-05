ALTER TYPE pgml.task RENAME VALUE 'cluster' TO 'clustering';
ALTER TYPE pgml.task ADD VALUE IF NOT EXISTS 'decomposition';

ALTER TYPE pgml.algorithm ADD VALUE IF NOT EXISTS 'pca';

-- pgml::api::decompose
CREATE FUNCTION pgml."decompose"(
    "project_name" TEXT, /* alloc::string::String */
    "vector" FLOAT4[] /* Vec<f32> */
) RETURNS FLOAT4[] /* Vec<f32> */
    IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'decompose_wrapper';
