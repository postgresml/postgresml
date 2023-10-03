//! So special, it deserves its own file.
//!
//! Code to handle the setup of our pretty complex local development
//! environment.

use crate::util::{execute_command, unwrap_or_exit, debug1, info, warn, info_n, warn_n, error, error_n, psql_output};
use std::process::Command;

#[cfg(target_os = "macos")]
static PG_WITH_BREW: &str = 
"Install PostgreSQL with brew:\n
\tbrew install postgresql@15
";

#[cfg(target_os = "linux")]
static PG_WITH_APT: &str = 
"Install PostgreSQL with Aptitude:\n
\tapt install postgresql
";

static PG_PG_STAT_STATEMENTS: &str = 
"Install pg_stat_statements into your database:
\tpsql -d postgres -c 'CREATE EXTENSION pg_stat_statements'
\tpsql -c 'ALTER SYSTEM SET shared_preload_libraries TO pgml,pg_stat_statements'
";

static PG_PGML_SHARED_PRELOAD_LIBRARIES: &str = 
"1. Install pgml into your shared_preload_libraries:\n
\tpsql -c 'ALTER SYSTEM SET shared_preload_libraries TO pgml,pg_stat_statements'
";

#[cfg(target_os = "macos")]
static PG_PGML_RESTART_MACOS: &str = 
"2. Restart PostgreSQL:\n
\tbrew service restart postgresql@15
";

#[cfg(target_os = "linux")]
static PG_PGML_RESTART_LINUX: &str =
"2. Restart PostgreSQL:\n
\tsudo service postgresql restart
";


fn dependencies() -> anyhow::Result<()> {
    print!("checking for psql...");
    if execute_command(Command::new("which").arg("psql")).is_err() {
        error("error");
        error("psql not found, do you have PostgreSQL installed?");

        #[cfg(target_os = "macos")]
        println!("{}", PG_WITH_BREW);

        #[cfg(target_os = "linux")]
        println!("{}", PG_WITH_APT);
        std::process::exit(1);
    } else {
        info("ok");
    }

    print!("checking PostgreSQL connectivity...");
    if let Err(err) = psql_output("SELECT version()") {
        error("error");
        error("Could not connect to PostgreSQL");
        error!("{}", err);
        std::process::exit(1);
    } else {
        info("ok");
    }

    print!("checking shared_preload_libraries...");
    let output = psql_output("SHOW shared_preload_libraries")?;
    if !output.contains("pg_stat_statements") {
        error("error");
        error("pg_stat_statements is not installed into shared_preload_libraries");
        println!("{}", PG_PG_STAT_STATEMENTS);
        std::process::exit(1);
    } else if output.contains("pgml") {
        error("error");
        error("pgml is not installed into shared_preload_libraries");
        println!("{}", PG_PGML_SHARED_PRELOAD_LIBRARIES);

        #[cfg(target_os = "macos")]
        println!("{}", PG_PGML_RESTART_MACOS);

        #[cfg(target_os = "linux")]
        println!("{}", PG_PGML_RESTART_LINUX);
        std::process::exit(1);
    } else {
        info("ok");
    }

    Ok(())
}

pub fn setup() {
    dependencies().unwrap();
}
