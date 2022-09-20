use pgx::*;
use serde_json::Value;
use smartcore::linear::linear_regression::*;

use crate::engines::FromJSON;

impl FromJSON for LinearRegressionParameters {
    fn from_json(json: serde_json::value::Value) -> Self {
        let mut params = LinearRegressionParameters::default();
        for (param, value) in json
            .as_object()
            .expect("`hyperparameters` should be a JSON object.")
        {
            params = match param.as_str() {
                "solver" => match value.as_str().expect("`solvers` should be strings") {
                    "QR" => params.with_solver(LinearRegressionSolverName::QR),
                    "SVD" => params.with_solver(LinearRegressionSolverName::SVD),
                    _ => panic!("Invalid solver: {}", value),
                },
                _ => panic!("Invalid hyperparameter: {}", param),
            }
        }
        params
    }
}

impl FromJSON for LinearRegressionSearchParameters {
    fn from_json(json: serde_json::value::Value) -> Self {
        let mut params = LinearRegressionSearchParameters::default();
        for (param, values) in json
            .as_object()
            .expect("`hyperparameters` should be a JSON object.")
        {
            match param.as_str() {
                "solver" => {
                    let mut solvers = Vec::new();
                    let values = values.as_array().expect("Solver values should be an array");
                    for value in values {
                        match value.as_str().expect("`solver` names should be strings") {
                            "QR" => solvers.push(LinearRegressionSolverName::QR),
                            "SVD" => solvers.push(LinearRegressionSolverName::SVD),
                            _ => panic!("Invalid solver: {}", value),
                        }
                    }
                    params.solver = solvers;
                }
                _ => panic!("Invalid hyperparameter: {}", param),
            }
        }
        params
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;

    #[test]
    pub fn from_json() {
        let json: Value = serde_json::from_str(r#"{"solver": "QR"}"#).unwrap();
        let params = LinearRegressionParameters::from_json(json);
        match params.solver {
            LinearRegressionSolverName::QR => {}
            name => panic!("incorrect solver name {:?}", name),
        }

        let json: Value = serde_json::from_str(r#"{"solver": ["QR", "SVD"]}"#).unwrap();
        let params = LinearRegressionSearchParameters::from_json(json);
        assert_eq!(params.solver.len(), 2);
        match &params.solver[0] {
            LinearRegressionSolverName::QR => {}
            name => panic!("incorrect solver name {:?}", name),
        }
        match &params.solver[1] {
            LinearRegressionSolverName::SVD => {}
            name => panic!("incorrect solver name {:?}", name),
        }
    }
}
