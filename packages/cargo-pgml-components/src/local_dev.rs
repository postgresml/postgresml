//! So special, it deserves its own file.
//!
//! Code to handle the setup of our pretty complex local development
//! environment.

use crate::util::{
    compare_files, error, execute_command, info, ok_or_error, print, psql_output, unwrap_or_exit,
    warn,
};
use std::path::Path;
use std::process::{exit, Command};

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
\tbrew services restart postgresql@15
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
static PG_PGVECTOR: &str = "
\t rm -rf /tmp/pgvector && \\
\tgit clone --branch v0.5.0 https://github.com/pgvector/pgvector /tmp/pgvector && \\
\tcd /tmp/pgvector && \\
\techo \"trusted = true\" >> vector.control && \\
\tmake && \\
\tmake install
";

#[cfg(target_os = "linux")]
static PG_PGVECTOR: &str = "
\t rm -rf /tmp/pgvector && \\
\tgit clone --branch v0.5.0 https://github.com/pgvector/pgvector /tmp/pgvector && \\
\tcd /tmp/pgvector && \\
\techo \"trusted = true\" >> vector.control && \\
\tmake && \\
\tsudo make install
";

#[cfg(target_os = "macos")]
static PG_PGML: &str = "To install PostgresML into your PostgreSQL database,
follow the instructions on:

\thttps://postgresml.org/docs/setup/v2/installation
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
\tbrew services start postgresql@15
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

    #[cfg(target_os = "macos")]
    {
        print("checking for brew...");
        if execute_command(Command::new("which").arg("brew")).is_err() {
            error("missing");
            println!("\nBrew is not installed. Install it from https://brew.sh/\n");
            exit(1);
        } else {
            info("ok");
        }
    }

    #[cfg(target_os = "linux")]
    let postgres_service = "postgresql";

    #[cfg(target_os = "macos")]
    let postgres_service = "postgresql@15";

    print("checking if PostgreSQL is running...");
    if !check_service_running(postgres_service) {
        error("error");

        println!("\nPostgreSQL service is not running. To start PostgreSQL, run:\n");

        #[cfg(target_os = "linux")]
        println!("\tsudo service postgresql start\n");

        #[cfg(target_os = "macos")]
        println!("\tbrew services start postgresql@15\n");

        exit(1);
    } else {
        info("ok");
    }

    print("checking for PostgreSQL connectivity...");
    if let Err(err) = psql_output("SELECT version()") {
        error("error");
        error!("{}", err);
        println!("{}", postgres_running());
    } else {
        info("ok");
    }

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
        print("running quick environment test...");
        unwrap_or_exit!(execute_command(
            Command::new("dropdb")
                .arg("--if-exists")
                .arg("pgml_components_environment_test")
        ));
        unwrap_or_exit!(execute_command(
            Command::new("createdb").arg("pgml_components_environment_test")
        ));
        unwrap_or_exit!(execute_command(
            Command::new("psql")
                .arg("-c")
                .arg("CREATE EXTENSION vector")
                .arg("pgml_components_environment_test")
        ));
        unwrap_or_exit!(execute_command(
            Command::new("psql")
                .arg("-c")
                .arg("CREATE EXTENSION pgml")
                .arg("pgml_components_environment_test")
        ));
        unwrap_or_exit!(execute_command(
            Command::new("dropdb").arg("pgml_components_environment_test")
        ));
        info("ok");
    }

    print("checking .env file...");
    let env = Path::new(".env");
    let env_template = Path::new(".env.development");

    if !env.exists() && env_template.exists() {
        unwrap_or_exit!(execute_command(
            Command::new("cp").arg(".env.development").arg(".env")
        ));
        info("ok");
    } else if env.exists() && env_template.exists() {
        let identical = unwrap_or_exit!(compare_files(&env, &env_template));
        if !identical {
            warn("different");
            warn(".env has been modified");
        } else {
            info("ok");
        }
    } else if !env_template.exists() {
        warn("unknown");
        warn(".env.development not found, can't install or validate .env");
    } else {
        info("ok");
    }

    info("all dependencies are installed and working");

    Ok(())
}

pub fn setup() {
    unwrap_or_exit!(dependencies())
}

pub fn install_pgvector() {
    #[cfg(target_os = "linux")]
    {
        let check_sudo = execute_command(Command::new("sudo").arg("ls"));
        if check_sudo.is_err() {
            println!("Installing pgvector requires sudo permissions.");
            exit(1);
        }
    }

    print("installing pgvector PostgreSQL extension...");

    let result = execute_command(Command::new("bash").arg("-c").arg(PG_PGVECTOR));

    if let Ok(_) = result {
        info("ok");
    } else if let Err(ref err) = result {
        error("error");
        error!("{}", err);
    }
}

fn check_service_running(name: &str) -> bool {
    #[cfg(target_os = "linux")]
    let command = format!("service {} status", name);

    #[cfg(target_os = "macos")]
    let command = format!("brew services list | grep {} | grep started", name);

    execute_command(Command::new("bash").arg("-c").arg(&command)).is_ok()
}
