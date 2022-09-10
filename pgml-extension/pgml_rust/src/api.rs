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
) {
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

    info!("{:?}", project);
    info!("{:?}", snapshot);
    info!("{:?}", model);

    // TODO move deployment into a struct and only deploy if new model is better than old model
    Spi::get_one_with_args::<i64>(
        "INSERT INTO pgml_rust.deployments (project_id, model_id, strategy) VALUES ($1, $2, $3::pgml_rust.strategy) RETURNING id",
        vec![
            (PgBuiltInOids::INT8OID.oid(), project.id.into_datum()),
            (PgBuiltInOids::INT8OID.oid(), model.id.into_datum()),
            (PgBuiltInOids::TEXTOID.oid(), Strategy::most_recent.to_string().into_datum()),
        ]
    );
}

#[pg_extern]
fn predict(project_name: &str, features: Vec<f32>) -> f32 {
    let estimator = crate::orm::estimator::find_deployed_estimator_by_project_name(project_name);
    estimator.predict_me(features)
}

// #[pg_extern]
// fn return_table_example() -> impl std::Iterator<Item = (name!(id, Option<i64>), name!(title, Option<String>))> {
//     let tuple = Spi::get_two_with_args("SELECT 1 AS id, 2 AS title;", None, None)
//     vec![tuple].into_iter()
// }

#[pg_extern]
fn create_snapshot(
    relation_name: &str,
    y_column_name: &str,
    test_size: f32,
    test_sampling: Sampling,
) -> i64 {
    let snapshot = Snapshot::create(relation_name, y_column_name, test_size, test_sampling);
    info!("{:?}", snapshot);
    snapshot.id
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
