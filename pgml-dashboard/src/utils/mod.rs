pub mod config;
pub mod datadog;
pub mod markdown;
pub mod secure;

pub fn format_microseconds(microseconds: f64) -> String {
    if microseconds >= 1000000. {
        format!("{}s", microseconds / 1000000.)
    } else if microseconds >= 1000. {
        format!("{}ms", microseconds / 1000.)
    } else {
        format!("{}μs", microseconds)
    }
}
