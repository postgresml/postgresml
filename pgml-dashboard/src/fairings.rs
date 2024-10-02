use std::collections::HashMap;

use once_cell::sync::OnceCell;

use crate::utils::datadog::timing;

/// Times requests and responses for reporting via datadog
struct RequestMonitorStart(std::time::Instant);

#[derive(Default)]
pub struct RequestMonitor;

impl RequestMonitor {
    pub fn new() -> RequestMonitor {
        Self
    }
}

static PATH_IDS: OnceCell<regex::Regex> = OnceCell::new();

#[rocket::async_trait]
impl Fairing for RequestMonitor {
    fn info(&self) -> Info {
        Info {
            name: "Request Monitor",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _data: &mut Data<'_>) {
        let _ = request.local_cache(|| RequestMonitorStart(std::time::Instant::now()));
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        let start = request.local_cache(|| RequestMonitorStart(std::time::Instant::now())).0;
        let elapsed = start.elapsed().as_micros() as f32 / 1000.0;
        let status = response.status().code;
        let method = request.method().as_str();
        let path = match status {
            300..=399 => {
                // don't retain old paths
                "redirect".to_string()
            }
            404 => {
                // don't log high cardinality paths from scrapers
                "not_found".to_string()
            }
            _ => {
                // keep other paths lower cardinality by replacing ids with :id
                let regex = PATH_IDS.get_or_init(|| regex::Regex::new(r"/\d+").unwrap());
                let path = request.uri().path().to_string();
                regex.replace_all(&path, "/id").to_string()
            }
        };
        let tags = HashMap::from([
            ("status".to_string(), status.to_string()),
            ("method".to_string(), method.to_string()),
            ("path".to_string(), path.to_string()),
        ]);
        let metric = "http.request";
        timing(metric, elapsed, Some(&tags)).await;
    }
}
