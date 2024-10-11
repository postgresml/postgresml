use anyhow::Result;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use once_cell::sync::Lazy;
use pgrx::*;

use crate::bindings::Bindings;

use crate::orm::Algorithm;
use crate::orm::Runtime;
use crate::orm::Task;

#[allow(clippy::type_complexity)]
static DEPLOYED_ESTIMATORS_BY_MODEL_ID: Lazy<Mutex<HashMap<i64, Arc<Box<dyn Bindings>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Fetch and load the most up-to-date estimator for the given model.
pub fn find_deployed_estimator_by_model_id(model_id: i64) -> Result<Arc<Box<dyn Bindings>>> {
    // Get the estimator from process memory, if we already loaded it.
    {
        let estimators = DEPLOYED_ESTIMATORS_BY_MODEL_ID.lock();
        if let Some(estimator) = estimators.get(&model_id) {
            return Ok(estimator.clone());
        }
    }

    let mut data: Option<Vec<u8>> = None;
    let mut runtime: Option<String> = None;
    let mut algorithm: Option<String> = None;
    let mut task: Option<String> = None;
    let mut hyperparams: Option<JsonB> = None;

    Spi::connect(|client| {
        let result = client
            .select(
                "SELECT
                    data,
                    runtime::TEXT,
                    algorithm::TEXT,
                    task::TEXT,
                    hyperparams
                FROM pgml.models
                    INNER JOIN pgml.files
                        ON models.id = files.model_id 
                    INNER JOIN pgml.projects
                        ON models.project_id = projects.id
                    WHERE models.id = $1
                    LIMIT 1
                ",
                Some(1),
                Some(vec![(PgBuiltInOids::INT8OID.oid(), model_id.into_datum())]),
            )
            .unwrap()
            .first();

        if result.is_empty() {
            error!(
                "Model pgml.models.id = {} does not exist, the model store has been corrupted.",
                model_id
            );
        } else {
            data = result
                .get(1)
                .expect("Project has gone missing. Your model store has been corrupted.");
            runtime = result.get(2).expect("Runtime for model is corrupted.");
            algorithm = result.get(3).expect("Algorithm for model is corrupted.");
            task = result.get(4).expect("Task for project is corrupted.");
            hyperparams = result.get(5).expect("Hyperparams for model is corrupted.");
        }
    });

    let (data, runtime, algorithm) = Spi::get_three_with_args::<Vec<u8>, String, String>(
        "SELECT data, runtime::TEXT, algorithm::TEXT FROM pgml.models
        INNER JOIN pgml.files
            ON models.id = files.model_id 
        WHERE models.id = $1
        LIMIT 1",
        vec![(PgBuiltInOids::INT8OID.oid(), model_id.into_datum())],
    )
    .unwrap();

    let data = data.unwrap();
    let runtime = Runtime::from_str(&runtime.unwrap()).unwrap();
    let algorithm = Algorithm::from_str(&algorithm.unwrap()).unwrap();
    let task = Task::from_str(&task.unwrap()).unwrap();
    let hyperparams = hyperparams.unwrap();

    debug1!(
        "runtime = {:?}, algorithm = {:?}, task = {:?}",
        runtime,
        algorithm,
        task
    );

    let bindings: Box<dyn Bindings> = match runtime {
        Runtime::rust => {
            match algorithm {
                Algorithm::xgboost => crate::bindings::xgboost::Estimator::from_bytes(&data, &hyperparams)?,
                Algorithm::lightgbm => crate::bindings::lightgbm::Estimator::from_bytes(&data, &hyperparams)?,
                Algorithm::linear => match task {
                    Task::regression => crate::bindings::linfa::LinearRegression::from_bytes(&data, &hyperparams)?,
                    Task::classification => {
                        crate::bindings::linfa::LogisticRegression::from_bytes(&data, &hyperparams)?
                    }
                    _ => error!("Rust runtime only supports `classification` and `regression` task types for linear algorithms."),
                },
                Algorithm::svm => crate::bindings::linfa::Svm::from_bytes(&data, &hyperparams)?,
                _ => todo!(), //smartcore_load(&data, task, algorithm, &hyperparams),
            }
        }

        #[cfg(feature = "python")]
        Runtime::python => crate::bindings::sklearn::Estimator::from_bytes(&data, &hyperparams)?,

        #[cfg(not(feature = "python"))]
        Runtime::python => {
            anyhow::bail!("Python runtime not supported, recompile with `--features python`")
        }

        Runtime::openai => {
            error!("OpenAI runtime is not supported for training or inference");
        }
    };

    // Cache the estimator in process memory.
    let mut estimators = DEPLOYED_ESTIMATORS_BY_MODEL_ID.lock();
    estimators.insert(model_id, Arc::new(bindings));
    Ok(estimators.get(&model_id).unwrap().clone())
}
