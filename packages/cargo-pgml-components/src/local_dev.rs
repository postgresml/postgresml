//! So special, it deserves its own file.
//!
//! Code to handle the setup of our pretty complex local development
//! environment.

use crate::util::{execute_command, info, ok_or_error, print, psql_output, unwrap_or_exit, warn};
use std::process::Command;

#[cfg(target_os = "macos")]
static PG_INSTALL: &str = "
Install PostgreSQL with brew:\n
\tbrew install postgresql@15
";

#[cfg(target_os = "linux")]
static PG_INSTALL: &str = "
Install PostgreSQL with Aptitude:\n
\tsudo apt install postgresql
";

#[cfg(target_os = "macos")]
static BUILD_ESSENTIAL: &str = "
Install build tools with Aptitude:\n
\txcode-select --install
";

#[cfg(target_os = "linux")]
static BUILD_ESSENTIAL: &str = "
Install build tools with Aptitude:\n
\tsudo apt install build-essential
";

#[cfg(target_os = "macos")]
static PG_PG_STAT_STATEMENTS: &str = "
To install pg_stat_statements into your database:

1. Create the extension in PostgreSQL:\n
\tpsql -d postgres -c 'CREATE EXTENSION pg_stat_statements'
2. Add pg_stat_statements into your shared_preload_libraries:\n
\tpsql -c 'ALTER SYSTEM SET shared_preload_libraries TO pgml,pg_stat_statements'
3. Restart PostgreSQL:\n
\tbrew service restart postgresql@15
";

#[cfg(target_os = "linux")]
static PG_PG_STAT_STATEMENTS: &str = "
To install pg_stat_statements into your database:

1. Create the extension in PostgreSQL:\n
\tpsql -d postgres -c 'CREATE EXTENSION pg_stat_statements'
2. Add pg_stat_statements into your shared_preload_libraries:\n
\tpsql -c 'ALTER SYSTEM SET shared_preload_libraries TO pgml,pg_stat_statements'
3. Restart PostgreSQL:\n
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

fn postgres_running() -> String {
    let whoami = unwrap_or_exit!(execute_command(&mut Command::new("whoami")));

    let running = format!(
        "
Could not connect to PostgreSQL database 'postgres' with psql.\n
Is PostgreSQL running and accepting connections?
    "
    );

    #[cfg(target_os = "macos")]
    let start = format!(
        "
To start PostgreSQL, run:\n
\tbrew service start postgresql@15
    "
    );

    #[cfg(target_os = "linux")]
    let start = format!(
        "
To start PostgreSQL, run:\n
\tsudo service postgresql start
    "
    );

    let user = format!(
        "
If PostgreSQL is already running, your current UNIX user is
not allowed to connect to the 'postgres' database with psql
using a UNIX socket.

To make sure your user is allowed to connect:

1. Create the role:\n
\tcreaterole --superuser --login {whoami}

2. Create the user's database:\n
\t createdb {whoami}
    "
    );

    running + &start + &user
}

fn dependencies() -> anyhow::Result<()> {
    ok_or_error!(
        "checking for psql",
        { execute_command(Command::new("which").arg("psql")).is_ok() },
        PG_INSTALL
    );

    ok_or_error!(
        "checking for build tools",
        { execute_command(Command::new("which").arg("gcc")).is_ok() },
        BUILD_ESSENTIAL
    );

    ok_or_error!(
        "checking for PostgreSQL connectivity",
        {
            if let Err(err) = psql_output("SELECT version()") {
                error!("{}", err);
                false
            } else {
                true
            }
        },
        postgres_running()
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
            let output_installed = psql_output(
                "
                SELECT
                    name
                FROM
                    pg_available_extensions
                WHERE name = 'pgml'
            ",
            )?;

            let output_shared = psql_output("SHOW shared_preload_libraries")?;

            output_installed.contains("pgml") && output_shared.contains("pgml")
        },
        PG_PGML
    );

    ok_or_error!(
        "checking for pg_stat_statements PostgreSQL extension",
        {
            let output_installed = psql_output("SHOW shared_preload_libraries")?;
            let output_running = psql_output("SELECT * FROM pg_stat_statements LIMIT 1");
            output_installed.contains("pg_stat_statements") && output_running.is_ok()
        },
        PG_PG_STAT_STATEMENTS
    );

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
        print("creating vector extension in pgml_dashboard_development...");
        unwrap_or_exit!(execute_command(
            Command::new("psql")
                .arg("-c")
                .arg("CREATE EXTENSION IF NOT EXISTS vector")
                .arg("pgml_dashboard_development")
        ));
        info("ok");
        print("creating pgml extension in pgml_dashboard_development...");
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
    unwrap_or_exit!(dependencies())
}
