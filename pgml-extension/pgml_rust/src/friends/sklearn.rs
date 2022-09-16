use pgx::*;
use pyo3::prelude::*;

use crate::orm::algorithm::Algorithm;
use crate::orm::dataset::Dataset;
use crate::orm::task::Task;

#[pg_extern]
fn sklearn_version() -> String {
    let mut version = String::new();

    Python::with_gil(|py| {
        let sklearn = py.import("sklearn").unwrap();
        version = sklearn.getattr("__version__").unwrap().extract().unwrap();
    });

    version
}

pub fn sklearn_train(task: Task, algorithm: Algorithm, dataset: &Dataset) {
    match algorithm {
        Algorithm::linear => {
            match task {
                Task::regression => todo!(),
                Task::classification => {
                    Python::with_gil(|py| {
                        let linear_model = py.import("sklearn.linear_model").unwrap();
                        let logistic_regression =
                            linear_model.getattr("LogisticRegression").unwrap();
                        let classifier = logistic_regression.call0().unwrap();

                        let (x_train, y_train) = (dataset.x_train(), dataset.y_train());

                        let fit = classifier.getattr("fit").unwrap();

                        // fit.call1((x_train, y_train)).unwrap();

                        panic!("Bye"); // Don't let it get stuck in the smartcore classifier
                    });
                }
            };
        }

        _ => todo!(),
    };
}
