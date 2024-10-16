use log::{error, info};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::io::Result;
use std::string::ToString;
use std::time::Instant;
use tokio::sync::OnceCell;
use zoomies::DatagramFormat;
use zoomies::{Metric, UdsClient};

static CLIENT: OnceCell<Result<UdsClient>> = OnceCell::const_new();
static DEFAULT_TAGS: Lazy<HashMap<String, String>> =
    Lazy::new(|| HashMap::from([("app".to_string(), "pgml".to_string())]));

pub async fn client() -> &'static Result<UdsClient> {
    CLIENT
        .get_or_init(|| async { UdsClient::with_filepath("/var/run/datadog/dsd.socket").await })
        .await
}

async fn send<'a, T: std::fmt::Display + num_traits::Num>(
    metric: Metric<'a, T>,
    tags: Option<&HashMap<String, String>>,
) {
    let mut merged_tags = DEFAULT_TAGS.clone();
    if let Some(tags) = tags {
        merged_tags.extend(tags.clone());
    }

    match client().await {
        Ok(client) => match client.send_with_tags(&metric, &merged_tags).await {
            Ok(_) => (),
            Err(err) => error!("datadog: {err}"),
        },
        Err(_) => info!("datadog: {}{}", metric.format(), merged_tags.format()),
    };
}

pub async fn increment(metric: &str, tags: Option<&HashMap<String, String>>) {
    send(Metric::Inc::<u32>(metric), tags).await;
}

#[allow(dead_code)]
pub async fn decrement(metric: &str, tags: Option<&HashMap<String, String>>) {
    send(Metric::Dec::<u32>(metric), tags).await;
}

#[allow(dead_code)]
pub async fn count(metric: &str, value: f32, tags: Option<&HashMap<String, String>>) {
    send(Metric::Arb::<f32>(metric, value), tags).await;
}

#[allow(dead_code)]
pub async fn gauge(metric: &str, value: f32, tags: Option<&HashMap<String, String>>) {
    send(Metric::Gauge::<f32>(metric, value), tags).await;
}

#[allow(dead_code)]
pub async fn histogram(metric: &str, value: f32, tags: Option<&HashMap<String, String>>) {
    send(Metric::Histogram::<f32>(metric, value), tags).await;
}

#[allow(dead_code)]
pub async fn distribution(metric: &str, value: f32, tags: Option<&HashMap<String, String>>) {
    send(Metric::Distribution::<f32>(metric, value), tags).await;
}

#[allow(dead_code)]
pub async fn set(metric: &str, value: f32, tags: Option<&HashMap<String, String>>) {
    send(Metric::Set::<f32>(metric, value), tags).await;
}

pub async fn timing(metric: &str, millis: f32, tags: Option<&HashMap<String, String>>) {
    send(Metric::Time::<f32>(metric, millis), tags).await;
}

#[allow(dead_code)]
pub async fn time<T>(metric: &str, tags: Option<&HashMap<String, String>>, f: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let result = f();
    send(
        Metric::Time::<f32>(metric, start.elapsed().as_micros() as f32 / 1000.0),
        tags,
    )
    .await;
    result
}

pub async fn time_async<F, Fut, R>(metric: &str, tags: Option<&HashMap<String, String>>, f: F) -> R
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = R>,
{
    let start = Instant::now();
    let result = f().await;
    send(
        Metric::Time::<f32>(metric, start.elapsed().as_micros() as f32 / 1000.0),
        tags,
    )
    .await;
    result
}
