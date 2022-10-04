use parking_lot::Mutex;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use once_cell::sync::Lazy;
use pgx::*;

use crate::bindings::Bindings;

use crate::orm::Algorithm;
use crate::orm::Runtime;

#[allow(clippy::type_complexity)]
static DEPLOYED_ESTIMATORS_BY_MODEL_ID: Lazy<Mutex<HashMap<i64, Arc<Box<dyn Bindings>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Fetch and load the most up-to-date estimator for the given model.
pub fn find_deployed_estimator_by_model_id(model_id: i64) -> Arc<Box<dyn Bindings>> {
    // Get the estimator from process memory, if we already loaded it.
    {
        let estimators = DEPLOYED_ESTIMATORS_BY_MODEL_ID.lock();
        if let Some(estimator) = estimators.get(&model_id) {
            return estimator.clone();
        }
    }

    let (data, runtime, algorithm) = Spi::get_three_with_args::<Vec<u8>, String, String>(
        "SELECT data, runtime::TEXT, algorithm::TEXT FROM pgml.models
        INNER JOIN pgml.files
            ON models.id = files.model_id 
        WHERE models.id = $1
        LIMIT 1",
        vec![(PgBuiltInOids::INT8OID.oid(), model_id.into_datum())],
    );

    let data = data.unwrap_or_else(|| {
        panic!(
            "Project has gone missing for model: {}. Your model store has been corrupted.",
            model_id
        )
    });
    let runtime = Runtime::from_str(&runtime.unwrap()).unwrap();
    let algorithm = Algorithm::from_str(&algorithm.unwrap()).unwrap();

    info!("load {:?} {:?}", runtime, algorithm);
    let bindings: Box<dyn Bindings> = match runtime {
        Runtime::rust => {
            match algorithm {
                Algorithm::xgboost => crate::bindings::xgboost::Estimator::from_bytes(&data),
                Algorithm::lightgbm => crate::bindings::lightgbm::Estimator::from_bytes(&data),
                Algorithm::linear => crate::bindings::linfa::LinearRegression::from_bytes(&data),
                _ => todo!(), //smartcore_load(&data, task, algorithm, &hyperparams),
            }
        }
        Runtime::python => crate::bindings::sklearn::Estimator::from_bytes(&data),
    };

    // Cache the estimator in process memory.
    let mut estimators = DEPLOYED_ESTIMATORS_BY_MODEL_ID.lock();
    estimators.insert(model_id, Arc::new(bindings));
    estimators.get(&model_id).unwrap().clone()
}
