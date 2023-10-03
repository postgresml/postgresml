//! So special, it deserves its own file.
//!
//! Code to handle the setup of our pretty complex local development
//! environment.

use crate::util::{
    debug1, error, execute_command, info, ok_or_error, print, psql_output, unwrap_or_exit, warn,
};
use std::process::Command;

#[cfg(target_os = "macos")]
static PG_INSTALL: &str = "Install PostgreSQL with brew:\n
\tbrew install postgresql@15
";

#[cfg(target_os = "linux")]
static PG_INSTALL: &str = "Install PostgreSQL with Aptitude:\n
\tapt install postgresql
";

static PG_PG_STAT_STATEMENTS: &str = "Install pg_stat_statements into your database:
\tpsql -d postgres -c 'CREATE EXTENSION pg_stat_statements'
\tpsql -c 'ALTER SYSTEM SET shared_preload_libraries TO pgml,pg_stat_statements'
";

static PG_PGML_SHARED_PRELOAD_LIBRARIES: &str =
    "1. Install pgml into your shared_preload_libraries:\n
\tpsql -c 'ALTER SYSTEM SET shared_preload_libraries TO pgml,pg_stat_statements'
";

#[cfg(target_os = "macos")]
static PG_PGML_RESTART_MACOS: &str = "2. Restart PostgreSQL:\n
\tbrew service restart postgresql@15
";

#[cfg(target_os = "linux")]
static PG_PGML_RESTART_LINUX: &str = "2. Restart PostgreSQL:\n
\tsudo service postgresql restart
";

#[cfg(target_os = "macos")]
static PG_PGVECTOR: &str = "Install pgvector into your PostgreSQL database:\n
\tgit clone --branch v0.5.0 https://github.com/pgvector/pgvector && \\
\tcd pgvector && \\
\techo \"trusted = true\" >> vector.control && \\
\tmake && \\
\tmake install
";

#[cfg(target_os = "linux")]
static PG_PGVECTOR: &str = "Install pgvector into your PostgreSQL database:\n
\tgit clone --branch v0.5.0 https://github.com/pgvector/pgvector && \\
\tcd pgvector && \\
\techo \"trusted = true\" >> vector.control && \\
\tmake && \\
\tsudo make install
";

#[cfg(target_os = "macos")]
static PG_PGML: &str = "To install PostgresML into your PostgreSQL database,
follow the instructions on:

\thttps://postgresml.org/docs/guides/setup/v2/installation
";

#[cfg(target_os = "linux")]
static PG_PGML: &str = "To install PostgresML
into your PostgreSQL database:

1. Add your Aptitude repository into your sources:

\techo \"deb [trusted=yes] https://apt.postgresml.org $(lsb_release -cs) main\" | \\
\tsudo tee -a /etc/apt/sources.list

2. Update Aptitude:

\tsudo apt update

3. Install PostgresML:

\tsudo apt install postgresml-14
";

fn dependencies() -> anyhow::Result<()> {
    ok_or_error!(
        "checking for psql",
        { execute_command(Command::new("which").arg("psql")).is_ok() },
        PG_INSTALL
    );

    ok_or_error!(
        "checking PostgreSQL connectivity",
        {
            if let Err(err) = psql_output("SELECT version()") {
                error!("{}", err);
                false
            } else {
                true
            }
        },
        "Could not connect to PostgreSQL with psql"
    );

    ok_or_error!(
        "checking for pgvector PostgreSQL extension",
        {
            let output = psql_output(
                "
                SELECT
                    name
                FROM
                    pg_available_extensions
                WHERE name = 'vector'
            ",
            )?;
            output.contains("vector")
        },
        PG_PGVECTOR
    );

    ok_or_error!(
        "checking for pgml PostgreSQL extension",
        {
            let output = psql_output(
                "
                SELECT
                    name
                FROM
                    pg_available_extensions
                WHERE name = 'pgml'
            ",
            )?;
            output.contains("pgml")
        },
        PG_PGML
    );

    print("checking shared_preload_libraries...");
    let output = psql_output("SHOW shared_preload_libraries")?;
    if !output.contains("pg_stat_statements") {
        error("error");
        error("pg_stat_statements is not installed into shared_preload_libraries");
        println!("{}", PG_PG_STAT_STATEMENTS);
        std::process::exit(1);
    } else if !output.contains("pgml") {
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

    print("checking for dashboard database...");
    let output = psql_output(
        "SELECT datname FROM pg_database WHERE datname = 'pgml_dashboard_development'",
    )?;
    if !output.contains("pgml_dashboard_development") {
        warn("missing");
        print("creating pgml_dashboard_development database...");
        unwrap_or_exit!(execute_command(
            Command::new("createdb").arg("pgml_dashboard_development")
        ));
        info("ok");
        print("creating vector extension...");
        unwrap_or_exit!(execute_command(
            Command::new("psql")
                .arg("-c")
                .arg("CREATE EXTENSION IF NOT EXISTS vector")
                .arg("pgml_dashboard_development")
        ));
        info("ok");
        print("creating pgml extension...");
        unwrap_or_exit!(execute_command(
            Command::new("psql")
                .arg("-c")
                .arg("CREATE EXTENSION IF NOT EXISTS pgml")
                .arg("pgml_dashboard_development")
        ));
        info("ok");
    } else {
        info("ok");
    }

    info("all dependencies are installed and working");

    Ok(())
}

pub fn setup() {
    dependencies().unwrap();
}
