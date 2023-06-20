/// A more type flexible version of format!
#[macro_export]
macro_rules! query_builder {
    ($left:expr, $( $x:expr ),* ) => {{
        let mut query = $left.to_string();
        $( query = query.replacen("%s", &$x, 1); )*
        query
    }};
}
