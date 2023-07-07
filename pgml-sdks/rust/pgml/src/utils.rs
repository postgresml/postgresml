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
