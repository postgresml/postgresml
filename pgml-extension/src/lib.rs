extern crate blas;
extern crate linfa_linear;
extern crate openblas_src;
extern crate serde;
extern crate signal_hook;

use pgrx::*;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod api;
pub mod bindings;
pub mod metrics;
pub mod orm;
pub mod vectors;

pg_module_magic!();

extension_sql_file!("../sql/schema.sql", name = "schema");

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {}

#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec!["shared_preload_libraries = 'pgml'"]
    }
}
