use anyhow::Context;
use indicatif::{ProgressBar, ProgressStyle};
use lopdf::Document;
use std::fs;
use std::path::Path;

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

pub fn default_progress_spinner(size: u64) -> ProgressBar {
    ProgressBar::new(size).with_style(
        ProgressStyle::with_template("[{elapsed_precise}] {spinner:0.cyan/blue} {prefix}: {msg}")
            .unwrap(),
    )
}

pub fn default_progress_bar(size: u64) -> ProgressBar {
    ProgressBar::new(size).with_style(
        ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} ")
            .unwrap(),
    )
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
