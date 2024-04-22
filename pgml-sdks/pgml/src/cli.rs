use clap::{Parser, Subcommand};
use colored::Colorize;
use inquire::Text;
use is_terminal::IsTerminal;
use itertools::Itertools;
#[cfg(feature = "python")]
use pyo3::exceptions::PyRuntimeError;
#[cfg(feature = "python")]
use pyo3::prelude::*;
use sqlx::{Acquire, Executor};
use std::io::Write;

/// PostgresML CLI: configure your PostgresML deployments & create connections to remote data sources.
#[cfg(feature = "python")]
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None, name = "pgml", bin_name = "pgml")]
struct Python {
    /// We're running this as `python -m`, this argument is ignored
    #[arg(short)]
    module: Option<String>,

    #[command(subcommand)]
    subcommand: Subcommands,
}

/// PostgresML CLI
#[cfg(feature = "javascript")]
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None, name = "pgml", bin_name = "pgml")]
struct Javascript {
    /// Ignore this argument, we're running as `node`.
    #[arg(name = "pgmlcli")]
    pgmlcli: Option<String>,

    #[command(subcommand)]
    subcommand: Subcommands,
}

/// PostgresML CLI is Rust by default
#[cfg(all(not(feature = "python"), not(feature = "javascript")))]
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None, name = "pgml", bin_name = "pgml")]
struct Rust {
    /// TODO comment on the necessity of this argument.
    #[arg(name = "pgmlcli")]
    pgmlcli: Option<String>,

    #[command(subcommand)]
    subcommand: Subcommands,
}

#[derive(Subcommand, Debug, Clone)]
enum Subcommands {
    /// Connect your PostgresML database to another PostgreSQL database.
    Connect {
        /// Name for this connection. Allows to configure multiple connections
        /// from PostgresML to any number of databases.
        #[arg(long)]
        name: Option<String>,

        /// Host name or IP address of your database.
        /// The database must be reachable from our cloud via a private link
        /// or the Internet.
        #[arg(long)]
        host: Option<String>,

        /// The port on which the database server is running.
        #[arg(long)]
        port: Option<String>,

        /// A user that has read permissions to your schemas and tables.
        #[arg(long)]
        user: Option<String>,

        /// The password for the user.
        #[arg(long)]
        password: Option<String>,

        /// The name of the Postgres database.
        #[arg(long)]
        database_name: Option<String>,

        /// If you're using another schema that's not public,
        /// you can specify it here.
        #[arg(long)]
        schema: Option<String>,

        /// Don't do anything, just print the commands.
        #[arg(long, default_value = "false")]
        dry_run: bool,

        /// Drop the connection before creating it.
        #[arg(long, default_value = "false")]
        drop: bool,

        /// DATABASE_URL for your PostgresML database.
        #[arg(long)]
        database_url: Option<String>,
    },

    /// Connect your database to PostgresML via dblink.
    Remote {
        /// DATABASE_URL.
        #[arg(long, short)]
        database_url: Option<String>,
    },
}

enum Level {
    Happy,
    Sad,
    #[allow(dead_code)]
    Concerned,
}

#[cfg(feature = "python")]
#[pyfunction]
pub fn cli(py: pyo3::Python) -> pyo3::PyResult<&pyo3::PyAny> {
    ctrlc::set_handler(move || {
        println!("");
        std::process::exit(1);
    })
    .expect("failed to set ctrl-c handler");

    pyo3_asyncio::tokio::future_into_py(py, async move {
        match cli_internal().await {
            Ok(_) => Ok(()),
            Err(err) => Err(PyRuntimeError::new_err(format!("{}", err))),
        }
    })
}

#[cfg(feature = "javascript")]
pub fn cli(
    mut cx: neon::context::FunctionContext,
) -> neon::result::JsResult<neon::types::JsPromise> {
    ctrlc::set_handler(move || {
        println!("");
        std::process::exit(1);
    })
    .expect("failed to set ctrl-c handler");

    use neon::prelude::*;
    use rust_bridge::javascript::IntoJsResult;
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();
    deferred
        .try_settle_with(&channel, move |mut cx| {
            let runtime = crate::get_or_set_runtime();
            let x = runtime.block_on(cli_internal());
            let x = match x {
                Ok(x) => x,
                Err(e) => {
                    // Node has its own ctrl-c handler, so we need to handle it here.
                    if e.to_string()
                        .contains("Operation was interrupted by the user")
                    {
                        std::process::exit(1);
                    } else {
                        panic!("{e}");
                    }
                }
            };
            x.into_js_result(&mut cx)
        })
        .expect("Error sending js");
    Ok(promise)
}

#[cfg(all(not(feature = "python"), not(feature = "javascript")))]
pub async fn cli() -> anyhow::Result<()> {
    cli_internal().await
}

async fn cli_internal() -> anyhow::Result<()> {
    #[cfg(feature = "python")]
    let subcommand = {
        let args = Python::parse();
        args.subcommand
    };

    #[cfg(feature = "javascript")]
    let subcommand = {
        let args = Javascript::parse();
        args.subcommand
    };

    // Rust by default
    #[cfg(all(not(feature = "python"), not(feature = "javascript")))]
    let subcommand = {
        let args = Rust::parse();
        args.subcommand
    };

    match subcommand {
        Subcommands::Connect {
            name,
            host,
            port,
            user,
            password,
            database_name,
            dry_run,
            schema,
            drop,
            database_url,
        } => {
            connect(
                name,
                host,
                port,
                user,
                password,
                database_name,
                schema,
                dry_run,
                drop,
                database_url,
            )
            .await?;
        }

        Subcommands::Remote { database_url } => {
            remote(database_url).await?;
        }
    };

    Ok(())
}

async fn execute_sql(sql: &str) -> anyhow::Result<()> {
    let pool = crate::get_or_initialize_pool(&None).await?;
    let mut connection = pool.acquire().await?;
    let mut transaction = connection.begin().await?;

    for query in sql.split(";") {
        transaction.execute(query).await?;
    }

    transaction.commit().await?;

    Ok(())
}

async fn connect(
    name: Option<String>,
    host: Option<String>,
    port: Option<String>,
    user: Option<String>,
    password: Option<String>,
    database_name: Option<String>,
    schema: Option<String>,
    dry_run: bool,
    drop: bool,
    database_url: Option<String>,
) -> anyhow::Result<()> {
    println!("");
    println!("The connector will configure a Postgres Foreign Data Wrapper connection");
    println!("from PostgresML to your Postgres database of choice. If we're missing any details,");
    println!("we'll ask for them now.");
    println!("");

    if std::env::var("DATABASE_URL").is_err() && database_url.is_none() {
        println!("Required DATABASE_URL environment variable is not set.");
        println!("We need it to connect to your PostgresML database.");
        println!("");
        let database_url = user_input!(None::<String>, "DATABASE_URL");
        std::env::set_var("DATABASE_URL", database_url);
        println!("");
    } else if let Some(database_url) = database_url {
        std::env::set_var("DATABASE_URL", database_url);
    }

    let name = user_input!(name, "Connection name", Some("production"));
    let host = user_input!(host, "PostgreSQL host");
    let port = user_input!(port, "PostgreSQL port", Some("5432"));
    let user = user_input!(user, "PostgreSQL user", Some("postgres"));
    let password = user_input!(password, "Password");
    let database_name = user_input!(database_name, "PostgreSQL database", Some("postgres"));
    let schema = user_input!(schema, "PostgreSQL schema", Some("public"));

    let sql = include_str!("sql/fdw.sql")
        .replace("{host}", &host)
        .replace("{port}", &port)
        .replace("{user}", &user)
        .replace("{password}", &password)
        .replace("{database_name}", &database_name)
        .replace("{db_name}", &name)
        .replace("{schema}", &schema);
    let drop_sql = include_str!("sql/fdw_drop.sql")
        .replace("{db_name}", &name)
        .replace("{schema}", &schema);

    if dry_run {
        println!("");
        if drop {
            println!("{}", syntax_highlight(&drop_sql));
        }
        println!("{}", syntax_highlight(&sql));
        println!("");
    } else {
        println!("");
        print!("Everything looks good, creating connection...");
        std::io::stdout().flush().unwrap();

        if drop {
            match execute_sql(&drop_sql).await {
                Ok(_) => (),
                Err(err) => {
                    println!("{}", colorize("error", Level::Sad));
                    println!("{}", err);
                    std::process::exit(1);
                }
            };
        }

        match execute_sql(&sql).await {
            Ok(_) => {
                println!("{}", colorize("done", Level::Happy));
                println!("");
                println!("You can now use your PostgreSQL tables inside your PostgresML database.");
                println!("If you connect with psql, you can view your tables by updating your search_path:");
                println!("");
                println!(
                    "{}",
                    syntax_highlight(&format!("SET search_path TO {}_public, public;", name))
                );
                println!("");
            }
            Err(err) => {
                println!("{}", colorize("error", Level::Sad));
                println!("{}", err);
            }
        };
    }

    Ok(())
}

async fn remote(database_url: Option<String>) -> anyhow::Result<()> {
    let database_url = user_input!(database_url, "PostgresML DATABASE_URL");
    let database_url = url::Url::parse(&database_url)?;
    let user = database_url.username();
    if user.is_empty() {
        anyhow::bail!("user not found in DATABASE_URL");
    }

    let password = database_url.password();
    let password = if password.is_none() {
        anyhow::bail!("password not found in DATABASE_URL");
    } else {
        password.unwrap()
    };

    let host = database_url.host_str();
    let host = if host.is_none() {
        anyhow::bail!("host not found in DATABASE_URL");
    } else {
        host.unwrap()
    };

    let port = database_url.port();
    let port = if port.is_none() {
        "6432".to_string()
    } else {
        port.unwrap().to_string()
    };

    let database = database_url.path().replace("/", "");

    let sql = include_str!("sql/remote.sql")
        .replace("{user}", user)
        .replace("{password}", password)
        .replace("{host}", host)
        .replace("{db_name}", "postgresml")
        .replace("{database_name}", &database)
        .replace("{port}", &port);

    println!("{}", syntax_highlight(&sql));
    Ok(())
}

fn syntax_highlight(text: &str) -> String {
    if !std::io::stdout().is_terminal() {
        return text.to_owned();
    }

    text.split(" ")
        .into_iter()
        .map(|word| {
            let uppercase = word.chars().all(|c| c.is_ascii_uppercase());

            if uppercase {
                word.cyan().to_string()
            } else {
                word.to_owned()
            }
        })
        .join(" ")
}

fn colorize(text: &str, level: Level) -> String {
    if !std::io::stdout().is_terminal() {
        return text.to_owned();
    }

    match level {
        Level::Happy => text.green().to_string(),
        Level::Sad => text.red().to_string(),
        Level::Concerned => text.yellow().to_string(),
    }
}

macro_rules! user_input {
    ($var:expr, $prompt:expr, $default:expr) => {{
        if $var.is_none() {
            let prompt = format!("{}:", $prompt);
            let prompt = if let Some(default) = $default {
                Text::new(&prompt).with_default(default).prompt()?
            } else {
                Text::new(&prompt).prompt()?
            };
            prompt.to_string()
        } else {
            $var.unwrap()
        }
    }};

    ($var:expr, $prompt:expr) => {{
        user_input!($var, $prompt, None)
    }};

    ($var:expr) => {{
        user_input!($var, strginfy!($var))
    }};
}

use user_input;
