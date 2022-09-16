use pgx::*;
use serde::Deserialize;

#[derive(PostgresEnum, Copy, Clone, Eq, PartialEq, Debug, Deserialize)]
#[allow(non_camel_case_types)]
pub enum Algorithm {
    linear,
    xgboost,
    svm,
    lasso,
    elastic_net,
    ridge,
    kmeans,
    dbscan,
    knn,
    random_forest,
}

impl std::str::FromStr for Algorithm {
    type Err = ();

    fn from_str(input: &str) -> Result<Algorithm, Self::Err> {
        match input {
            "linear" => Ok(Algorithm::linear),
            "xgboost" => Ok(Algorithm::xgboost),
            "svm" => Ok(Algorithm::svm),
            "lasso" => Ok(Algorithm::lasso),
            "elastic_net" => Ok(Algorithm::elastic_net),
            "ridge" => Ok(Algorithm::ridge),
            "kmeans" => Ok(Algorithm::kmeans),
            "dbscan" => Ok(Algorithm::dbscan),
            "knn" => Ok(Algorithm::knn),
            "random_forest" => Ok(Algorithm::random_forest),
            _ => Err(()),
        }
    }
}

impl std::string::ToString for Algorithm {
    fn to_string(&self) -> String {
        match *self {
            Algorithm::linear => "linear".to_string(),
            Algorithm::xgboost => "xgboost".to_string(),
            Algorithm::svm => "svm".to_string(),
            Algorithm::lasso => "lasso".to_string(),
            Algorithm::elastic_net => "elastic_net".to_string(),
            Algorithm::ridge => "ridge".to_string(),
            Algorithm::kmeans => "kmeans".to_string(),
            Algorithm::dbscan => "dbscan".to_string(),
            Algorithm::knn => "knn".to_string(),
            Algorithm::random_forest => "random_forest".to_string(),
        }
    }
}
