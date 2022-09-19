use crate::orm::algorithm::Algorithm;
use crate::orm::dataset::Dataset;
use crate::orm::estimator::Estimator;
use crate::orm::task::Task;
use ndarray::{Array1, Array2};

fn smartcore_train(
    task: Task,
    algorithm: Algorithm,
    dataset: &Dataset,
) -> Option<Box<dyn Estimator>> {
    let x_train = Array2::from_shape_vec(
        (dataset.num_train_rows, dataset.num_features),
        dataset.x_train().to_vec(),
    )
    .unwrap();

    let y_train =
        Array1::from_shape_vec(dataset.num_train_rows, dataset.y_train().to_vec()).unwrap();

    match task {
        Task::classification => {
            match algorithm {
                _ => todo!(),
            };
        }

        Task::regression => {
            match algorithm {
                _ => todo!(),
            };
        }
    };

    None
}
