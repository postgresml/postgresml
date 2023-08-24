use indicatif::{ProgressBar, ProgressStyle};

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
