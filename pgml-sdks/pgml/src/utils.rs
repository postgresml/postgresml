use anyhow::Context;
use indicatif::{ProgressBar, ProgressStyle};
use lopdf::Document;
use std::fs;
use std::path::Path;
use std::time::Duration;

/// A more type flexible version of format!
#[macro_export]
macro_rules! query_builder {
    ($left:expr, $( $x:expr ),* ) => {{
        let re = regex::Regex::new(r"(%s|%d)").unwrap();
        let query = $left.to_string();
        $(
            let captures = re.captures(&query).unwrap();
            let caps = captures.get(0).expect("Your query is missing a %s or %d");
            let query = if caps.as_str() == "%s" {
                let y = $x.to_string().split('.').map(|s| format!("\"{}\"", s)).collect::<Vec<String>>().join(".");
                query.replacen("%s", &y, 1)
            } else {
                let y = $x.to_string();
                query.replacen("%d", &y, 1)
            };
        )*
        query
    }};
}

/// Used to debug sqlx queries
#[macro_export]
macro_rules! debug_sqlx_query {
    ($name:expr, $query:expr) => {{
        let name = stringify!($name);
        let sql = $query.to_string();
        let sql = sea_query::Query::select().expr(sea_query::Expr::cust(sql)).to_string(sea_query::PostgresQueryBuilder);
        let sql = sql.replacen("SELECT", "", 1);
        let span = tracing::span!(tracing::Level::DEBUG, "debug_query");
        tracing::event!(parent: &span, tracing::Level::DEBUG, %name,  %sql);
    }};

     ($name:expr, $query:expr, $( $x:expr ),*) => {{
        let name = stringify!($name);
        let sql = $query.to_string();
        let sql = sea_query::Query::select().expr(sea_query::Expr::cust_with_values(sql, [$(
           sea_query::Value::from($x.clone()),
        )*])).to_string(sea_query::PostgresQueryBuilder);
        let sql = sql.replacen("SELECT", "", 1);
        let span = tracing::span!(tracing::Level::DEBUG, "debug_query");
        tracing::event!(parent: &span, tracing::Level::DEBUG, %name, %sql);
     }};
}

/// Used to debug sea_query queries
#[macro_export]
macro_rules! debug_sea_query {
    ($name:expr, $query:expr, $values:expr) => {{
        let name = stringify!($name);
        let sql = $query.to_string();
        let sql = sea_query::Query::select().expr(sea_query::Expr::cust_with_values(sql, $values.clone().0)).to_string(sea_query::PostgresQueryBuilder);
        let sql = sql.replacen("SELECT", "", 1);
        let span = tracing::span!(tracing::Level::DEBUG, "debug_query");
        tracing::event!(parent: &span, tracing::Level::DEBUG, %name,  %sql);
    }};
}

pub fn default_progress_bar(size: u64) -> ProgressBar {
    let bar = ProgressBar::new(size).with_style(
        ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} ")
            .unwrap(),
    );
    bar.enable_steady_tick(Duration::from_millis(100));
    bar
}

pub fn get_file_contents(path: &Path) -> anyhow::Result<String> {
    let extension = path
        .extension()
        .with_context(|| format!("Error reading file extension: {}", path.display()))?
        .to_str()
        .with_context(|| format!("Extension is not valid UTF-8: {}", path.display()))?;
    Ok(match extension {
        "pdf" => {
            let doc = Document::load(path)
                .with_context(|| format!("Error reading PDF file: {}", path.display()))?;
            doc.get_pages()
                .into_keys()
                .map(|page_number| {
                    doc.extract_text(&[page_number]).with_context(|| {
                        format!("Error extracting content from PDF file: {}", path.display())
                    })
                })
                .collect::<anyhow::Result<Vec<String>>>()?
                .join("\n")
        }
        _ => fs::read_to_string(path)
            .with_context(|| format!("Error reading file: {}", path.display()))?,
    })
}
