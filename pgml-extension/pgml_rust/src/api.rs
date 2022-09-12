use std::str::FromStr;

use pgx::*;

use crate::orm::Algorithm;
use crate::orm::Model;
use crate::orm::Project;
use crate::orm::Sampling;
use crate::orm::Search;
use crate::orm::Snapshot;
use crate::orm::Strategy;
use crate::orm::Task;

#[pg_extern]
fn train(
    project_name: &str,
    task: Option<default!(Task, "NULL")>,
    relation_name: Option<default!(&str, "NULL")>,
    y_column_name: Option<default!(&str, "NULL")>,
    algorithm: default!(Algorithm, "'linear'"),
    hyperparams: default!(JsonB, "'{}'"),
    search: Option<default!(Search, "NULL")>,
    search_params: default!(JsonB, "'{}'"),
    search_args: default!(JsonB, "'{}'"),
    test_size: default!(f32, 0.25),
    test_sampling: default!(Sampling, "'last'"),
) -> impl std::iter::Iterator<
    Item = (
        name!(project, String),
        name!(task, String),
        name!(algorithm, String),
        name!(deployed, bool),
    ),
> {
    let project = match Project::find_by_name(project_name) {
        Some(project) => project,
        None => Project::create(project_name, task.unwrap()),
    };
    if task.is_some() && task.unwrap() != project.task {
        error!("Project `{:?}` already exists with a different task: `{:?}`. Create a new project instead.", project.name, project.task);
    }
    let snapshot = match relation_name {
        None => project.last_snapshot().expect("You must pass a `relation_name` and `y_column_name` to snapshot the first time you train a model."),
        Some(relation_name) => Snapshot::create(relation_name, y_column_name.expect("You must pass a `y_column_name` when you pass a `relation_name`"), test_size, test_sampling)
    };

    // # Default repeatable random state when possible
    // let algorithm = Model.algorithm_from_name_and_task(algorithm, task);
    // if "random_state" in algorithm().get_params() and "random_state" not in hyperparams:
    //     hyperparams["random_state"] = 0

    let model = Model::create(
        &project,
        &snapshot,
        algorithm,
        hyperparams,
        search,
        search_params,
        search_args,
    );

    let new_metrics: &serde_json::Value = &model.metrics.unwrap().0;
    let new_metrics = new_metrics.as_object().unwrap();

    let deployed_metrics = Spi::get_one_with_args::<JsonB>(
        "
        SELECT models.metrics
        FROM pgml_rust.models
        JOIN pgml_rust.deployments 
            ON deployments.model_id = models.id
        JOIN pgml_rust.projects
            ON projects.id = deployments.project_id
        WHERE projects.name = $1
        ORDER by deployments.created_at DESC
        LIMIT 1;",
        vec![(PgBuiltInOids::TEXTOID.oid(), project_name.into_datum())],
    );

    let mut deploy = false;
    if deployed_metrics.is_none() {
        deploy = true;
    } else {
        let deployed_metrics = deployed_metrics.unwrap().0;
        let deployed_metrics = deployed_metrics.as_object().unwrap();
        if project.task == Task::classification
            && deployed_metrics.get("f1").unwrap().as_f64()
                < new_metrics.get("f1").unwrap().as_f64()
        {
            deploy = true;
        }
        if project.task == Task::regression
            && deployed_metrics.get("r2").unwrap().as_f64()
                < new_metrics.get("r2").unwrap().as_f64()
        {
            deploy = true;
        }
    }

    if deploy {
        Spi::get_one_with_args::<i64>(
            "INSERT INTO pgml_rust.deployments (project_id, model_id, strategy) VALUES ($1, $2, $3::pgml_rust.strategy) RETURNING id",
            vec![
                (PgBuiltInOids::INT8OID.oid(), project.id.into_datum()),
                (PgBuiltInOids::INT8OID.oid(), model.id.into_datum()),
                (PgBuiltInOids::TEXTOID.oid(), Strategy::most_recent.to_string().into_datum()),
            ]
        );
    }

    vec![(
        project.name,
        project.task.to_string(),
        model.algorithm.to_string(),
        deploy,
    )]
    .into_iter()
}

#[pg_extern]
fn deploy(
    project_name: &str,
    strategy: Strategy,
    algorithm: Option<default!(Algorithm, "NULL")>,
) -> impl std::iter::Iterator<
    Item = (
        name!(project, String),
        name!(strategy, String),
        name!(algorithm, String),
    ),
> {
    let (project_id, task) = Spi::get_two_with_args::<i64, String>(
        "SELECT id, task::TEXT from pgml_rust.projects WHERE name = $1",
        vec![(PgBuiltInOids::TEXTOID.oid(), project_name.into_datum())],
    );
    let project_id =
        project_id.expect(format!("Project named `{}` does not exist.", project_name).as_str());
    let task = Task::from_str(&task.unwrap()).unwrap();

    let mut sql = "SELECT models.id, models.algorithm::TEXT FROM pgml_rust.models JOIN pgml_rust.projects ON projects.id = models.project_id".to_string();
    let mut predicate = "\nWHERE projects.name = $1".to_string();
    match algorithm {
        Some(algorithm) => {
            predicate += &format!(
                "\nAND algorithm::TEXT = '{}'",
                algorithm.to_string().as_str()
            )
        }
        _ => (),
    }
    match strategy {
        Strategy::best_score => match task {
            Task::regression => {
                sql += &format!("{predicate}\nORDER BY models.metrics->>'r2' DESC NULLS LAST");
            }
            Task::classification => {
                sql += &format!("{predicate}\nORDER BY models.metrics->>'f1' DESC NULLS LAST");
            }
        },
        Strategy::most_recent => {
            sql += &format!("{predicate}\nORDER by models.created_at DESC");
        }
        Strategy::rollback => {
            sql += &format!(
                "
                JOIN pgml_rust.deployments ON deployments.project_id = projects.id
                    AND deployments.model_id = models.id
                    AND models.id != (
                        SELECT models.id
                        FROM pgml_rust.models
                        JOIN pgml_rust.deployments 
                            ON deployments.model_id = models.id
                        JOIN pgml_rust.projects
                            ON projects.id = deployments.project_id
                        WHERE projects.name = $1
                        ORDER by deployments.created_at DESC
                        LIMIT 1
                    )
                {predicate}
                ORDER by deployments.created_at DESC
            "
            );
        }
        _ => error!("invalid stategy"),
    }
    sql += "\nLIMIT 1";
    let (model_id, algorithm_name) = Spi::get_two_with_args::<i64, String>(
        &sql,
        vec![(PgBuiltInOids::TEXTOID.oid(), project_name.into_datum())],
    );
    let model_id = model_id.expect("No qualified models exist for this deployment.");
    let algorithm_name = algorithm_name.expect("No qualified models exist for this deployment.");

    Spi::get_one_with_args::<i64>(
        "INSERT INTO pgml_rust.deployments (project_id, model_id, strategy) VALUES ($1, $2, $3::pgml_rust.strategy) RETURNING id",
        vec![
            (PgBuiltInOids::INT8OID.oid(), project_id.into_datum()),
            (PgBuiltInOids::INT8OID.oid(), model_id.into_datum()),
            (PgBuiltInOids::TEXTOID.oid(), strategy.to_string().into_datum()),
        ]
    );

    vec![(
        project_name.to_string(),
        strategy.to_string(),
        algorithm_name,
    )]
    .into_iter()
}

#[pg_extern]
fn predict(project_name: &str, features: Vec<f32>) -> f32 {
    let estimator = crate::orm::estimator::find_deployed_estimator_by_project_name(project_name);
    estimator.predict(features)
}

#[pg_extern]
fn snapshot(
    relation_name: &str,
    y_column_name: &str,
    test_size: default!(f32, 0.25),
    test_sampling: default!(Sampling, "'last'"),
) -> impl std::iter::Iterator<Item = (name!(relation, String), name!(y_column_name, String))> {
    Snapshot::create(relation_name, y_column_name, test_size, test_sampling);
    vec![(relation_name.to_string(), y_column_name.to_string())].into_iter()
}

#[pg_extern]
fn load_dataset(
    source: &str,
    limit: Option<default!(i64, "NULL")>,
) -> impl std::iter::Iterator<Item = (name!(table_name, String), name!(rows, i64))> {
    // cast limit since pgx doesn't support usize
    let limit: Option<usize> = match limit {
        Some(limit) => Some(limit.try_into().unwrap()),
        None => None,
    };
    let (name, rows) = match source {
        "breast_cancer" => crate::orm::dataset::load_breast_cancer(limit),
        "diabetes" => crate::orm::dataset::load_diabetes(limit),
        "digits" => crate::orm::dataset::load_digits(limit),
        "iris" => crate::orm::dataset::load_iris(limit),
        _ => error!("Unknown source: `{source}`"),
    };

    vec![(name, rows)].into_iter()
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;

    #[pg_test]
    fn test_project_lifecycle() {
        assert_eq!(Project::create("test", Task::regression).id, 1);
        assert_eq!(Project::find(1).id, 1);
    }

    #[pg_test]
    fn test_snapshot_lifecycle() {
        let snapshot = Snapshot::create("test", "column", 0.5, Sampling::last);
        assert_eq!(snapshot.id, 1);
    }
}
