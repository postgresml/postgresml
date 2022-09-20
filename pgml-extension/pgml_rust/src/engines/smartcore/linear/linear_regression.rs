use anyhow::{bail, Result};

use serde_json::{Map, Value};
use smartcore::linear::linear_regression::*;

use crate::engines::FromJSON;

impl FromJSON for LinearRegressionParameters {
    fn from_json(json: &Map<std::string::String, Value>) -> Result<Self> {
        let mut params = LinearRegressionParameters::default();
        for (param, value) in json {
            params = match param.as_str() {
                "solver" => match value.as_str() {
                    Some("QR") => params.with_solver(LinearRegressionSolverName::QR),
                    Some("SVD") => params.with_solver(LinearRegressionSolverName::SVD),
                    _ => bail!("Invalid solver: {}", value),
                },
                _ => bail!("Invalid hyperparam: {}", param),
            }
        }
        Ok(params)
    }
}

impl FromJSON for LinearRegressionSearchParameters {
    fn from_json(json: &Map<std::string::String, Value>) -> Result<Self> {
        let mut params = LinearRegressionSearchParameters::default();
        for (param, values) in json {
            match param.as_str() {
                "solver" => {
                    if let Some(values) = values.as_array() {
                        let mut solvers = Vec::new();
                        for value in values {
                            match value.as_str() {
                                Some("QR") => solvers.push(LinearRegressionSolverName::QR),
                                Some("SVD") => solvers.push(LinearRegressionSolverName::SVD),
                                _ => bail!("Invalid solver: {}", value),
                            }
                        }
                        params.solver = solvers;
                    } else {
                        bail!("Invalid list of solvers");
                    }
                }
                _ => bail!("Invalid hyperparameter: {}", param),
            }
        }
        Ok(params)
    }
}

#[cfg(any(test, feature = "pg_test"))]
mod tests {
    use super::*;

    #[test]
    pub fn from_json() {
        let json: Value = serde_json::from_str(r#"{"solver": "BAD VALUE"}"#).unwrap();
        let json: &Map<std::string::String, Value> = json.as_object().unwrap();
        let params = LinearRegressionParameters::from_json(&json);
        assert!(params.is_err());

        let json: Value = serde_json::from_str(r#"{"solver": "QR"}"#).unwrap();
        let json: &Map<std::string::String, Value> = json.as_object().unwrap();
        let params = LinearRegressionParameters::from_json(&json).unwrap();
        match params.solver {
            LinearRegressionSolverName::QR => {}
            name => panic!("incorrect solver name {:?}", name),
        }

        let json: Value = serde_json::from_str(r#"{"solver": ["SVD", "QR"]}"#).unwrap();
        let json: &Map<std::string::String, Value> = json.as_object().unwrap();
        let params = LinearRegressionSearchParameters::from_json(&json).unwrap();
        assert_eq!(params.solver.len(), 2);
        match &params.solver[0] {
            LinearRegressionSolverName::SVD => {}
            name => panic!("incorrect solver name {:?}", name),
        }
        match &params.solver[1] {
            LinearRegressionSolverName::QR => {}
            name => panic!("incorrect solver name {:?}", name),
        }
    }
}
