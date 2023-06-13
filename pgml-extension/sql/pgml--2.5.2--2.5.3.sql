-- src/api.rs:599
-- pgml::api::clear_gpu_cache
CREATE  FUNCTION pgml."clear_gpu_cache"(
    "memory_usage" REAL DEFAULT NULL /* Option<f32> */
) RETURNS bool /* bool */
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'clear_gpu_cache_wrapper';
