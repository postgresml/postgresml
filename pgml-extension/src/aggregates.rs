use pgx::*;
use serde::{Deserialize, Serialize};

use once_cell::sync::Lazy;
use parking_lot::Mutex;

#[allow(clippy::type_complexity)]
static PROJECT_NAME: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));

#[pg_extern(strict, name = "predict_agg_st")]
fn predict_agg_st(mut aggregates: Vec<f32>, project_name: &str, aggregate: Vec<f32>) -> Vec<f32> {
    // This is fast, don't worry, I tested
    let mut name = PROJECT_NAME.lock();
    name.clear();
    name.push_str(project_name);

    aggregates.extend(aggregate);
    aggregates
}

#[pg_extern(strict, name = "predict_agg_final")]
fn predict_agg_final(aggregates: Vec<f32>) -> Vec<f32> {
    crate::api::predict_batch(PROJECT_NAME.lock().as_str(), aggregates)
}
