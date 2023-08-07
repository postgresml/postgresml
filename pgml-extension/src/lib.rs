extern crate blas;
extern crate linfa_linear;
extern crate openblas_src;
extern crate serde;
extern crate signal_hook;

use pgrx::*;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod api;
pub mod bindings;
pub mod config;
pub mod metrics;
pub mod orm;
pub mod vectors;

#[cfg(not(feature = "use_as_lib"))]
pg_module_magic!();

extension_sql_file!("../sql/schema.sql", name = "schema");

#[cfg(not(feature = "use_as_lib"))]
#[pg_guard]
pub extern "C" fn _PG_init() {
    orm::project::init();
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;

    #[test]
    fn test_migration_file_exists() {
        let paths = std::fs::read_dir("./sql").unwrap();
        for path in paths {
            let path = path.unwrap().path().display().to_string();
            if path.contains(VERSION) {
                return;
            }
        }

        panic!("Migration file for version {} not found", VERSION);
    }
}

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
