/// Module providing various metrics used to rank the algorithms.
use ndarray::{Array2, ArrayView1, ArrayView2};
use std::collections::{BTreeSet, HashMap};

#[derive(Debug)]
pub struct ConfusionMatrixMetrics {
    tp: f32,
    fp: f32,
    fn_: f32,

    #[allow(dead_code)]
    tn: f32,
}

impl ConfusionMatrixMetrics {
    pub fn new(metrics: (f32, f32, f32, f32)) -> ConfusionMatrixMetrics {
        ConfusionMatrixMetrics {
            tp: metrics.0,
            fp: metrics.1,
            fn_: metrics.2,
            tn: metrics.3,
        }
    }
}

pub fn confusion_matrix(
    ground_truth: &ArrayView1<usize>,
    y_hat: &ArrayView1<usize>,
    num_classes: usize,
) -> Array2<f32> {
    assert_eq!(ground_truth.len(), y_hat.len());

    // Distinct classes.
    let mut classes = ground_truth.iter().collect::<BTreeSet<_>>();
    classes.extend(&mut y_hat.iter().collect::<BTreeSet<_>>().into_iter());

    assert_eq!(num_classes, classes.len());

    // Class value = index in the confusion matrix
    // e.g. class value 5 will be index 4 if there are classes 1, 2, 3 and 4 present.
    let indexes = classes
        .iter()
        .enumerate()
        .map(|(a, b)| (**b, a))
        .collect::<HashMap<usize, usize>>();

    let mut matrix = Array2::zeros((num_classes, num_classes));

    for (i, t) in ground_truth.iter().enumerate() {
        let h = y_hat[i];

        matrix[(indexes[t], indexes[&h])] += 1.0;
    }

    matrix
}

/// Return macro-averaged recall for the confusion matrix.
pub fn metrics(matrix: &ArrayView2<f32>, num_classes: usize) -> Vec<ConfusionMatrixMetrics> {
    let mut metrics = Vec::new();

    // Scikit confusion matrix starts from 1 and goes to 0,
    // ours starts from 0 and goes to 1. No big deal,
    // just flip everything lol.
    if num_classes == 2 {
        let tp = matrix[(1, 1)];
        let fp = matrix[(0, 1)];
        let fn_ = matrix[(1, 0)];
        let tn = matrix[(0, 0)];

        metrics.push(ConfusionMatrixMetrics::new((tp, fp, fn_, tn)));
    } else {
        for class in 0..num_classes {
            let tp = matrix[(class, class)];
            let fp = matrix.row(class).sum() - tp;
            let fn_ = matrix.column(class).sum() - tp;
            let tn = matrix.sum() - tp - fp - fn_;

            metrics.push(ConfusionMatrixMetrics::new((tp, fp, fn_, tn)));
        }
    }

    metrics
}

/// Return macro-averaged recall.
pub fn recall(metrics: &Vec<ConfusionMatrixMetrics>) -> f32 {
    let recalls = metrics
        .iter()
        .map(|m| m.tp / (m.tp + m.fn_))
        .collect::<Vec<f32>>();

    recalls.iter().sum::<f32>() / recalls.len() as f32
}

pub fn precision(metrics: &Vec<ConfusionMatrixMetrics>) -> f32 {
    let precisions = metrics
        .iter()
        .map(|m| m.tp / (m.tp + m.fp))
        .collect::<Vec<f32>>();

    precisions.iter().sum::<f32>() / precisions.len() as f32
}

pub fn f1(metrics: &Vec<ConfusionMatrixMetrics>) -> f32 {
    let recalls = metrics
        .iter()
        .map(|m| m.tp / (m.tp + m.fn_))
        .collect::<Vec<f32>>();
    let precisions = metrics
        .iter()
        .map(|m| m.tp / (m.tp + m.fp))
        .collect::<Vec<f32>>();

    let mut f1s = Vec::new();

    for (i, recall) in recalls.iter().enumerate() {
        let precision = precisions[i];
        f1s.push(2. * ((precision * recall) / (precision + recall)));
    }

    f1s.iter().sum::<f32>() / f1s.len() as f32
}

#[cfg(test)]
mod test {
    use super::*;
    use ndarray::array;

    #[test]
    fn test_confusion_matrix_multiclass() {
        let ground_truth = array![1, 2, 3, 4, 4];
        let y_hat = array![1, 2, 3, 4, 4];

        let mat = confusion_matrix(
            &ArrayView1::from(&ground_truth),
            &ArrayView1::from(&y_hat),
            4,
        );
        let metrics = metrics(&ArrayView2::from(&mat), 4);

        assert_eq!(mat[(3, 3)], 2.0);
        assert_eq!(f1(&metrics), 1.0);
    }
}
