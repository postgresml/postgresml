/// Module providing various metrics used to rank the algorithms.
use pgrx::*;
use std::collections::{BTreeSet, HashMap};

use ndarray::{Array2, ArrayView1};

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Average {
    Micro,
    Macro,
    Binary,
}

/// Confusion matrix metrics for a class.
#[derive(Debug)]
pub struct ConfusionMatrixMetrics {
    tp: f32,
    fp: f32,
    fn_: f32,

    #[allow(dead_code)]
    tn: f32,
}

impl ConfusionMatrixMetrics {
    /// Args: TP, FP, FN, TN.
    pub fn new(metrics: (f32, f32, f32, f32)) -> ConfusionMatrixMetrics {
        ConfusionMatrixMetrics {
            tp: metrics.0,
            fp: metrics.1,
            fn_: metrics.2,
            tn: metrics.3,
        }
    }
}

pub struct ConfusionMatrix {
    /// The confusion matrix in its raw form.
    matrix: Array2<f32>,

    /// Predicates calculated using the confusion matrix, indexed by class number.
    metrics: Vec<ConfusionMatrixMetrics>,
}

impl ConfusionMatrix {
    /// Construct a new confusion matrix from the ground truth
    /// and the predictions.
    /// `num_classes` is passed it to ensure that all classes
    /// were present in the test set.
    pub fn new(ground_truth: &ArrayView1<usize>, y_hat: &ArrayView1<usize>, num_classes: usize) -> ConfusionMatrix {
        // Distinct classes.
        let mut classes = ground_truth.iter().collect::<BTreeSet<_>>();
        classes.extend(&mut y_hat.iter().collect::<BTreeSet<_>>().into_iter());

        if ground_truth.len() != y_hat.len() {
            error!("Can't compute metrics when the ground truth labels are a different size than the predicted labels. {} != {}", ground_truth.len(), y_hat.len());
        }

        if num_classes != classes.len() {
            error!("Can't compute metrics when the number of classes in the test set is different than the number of classes in the training set. {} != {}", num_classes, classes.len());
        }

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

        ConfusionMatrix { matrix, metrics }
    }

    pub fn accuracy(&self) -> f32 {
        let numerator = self.matrix.diag().sum();
        let denominator = self.matrix.sum();

        numerator / denominator
    }

    /// Average recall.
    pub fn recall(&self) -> f32 {
        let recalls = self.metrics.iter().map(|m| m.tp / (m.tp + m.fn_)).collect::<Vec<f32>>();

        recalls.iter().sum::<f32>() / recalls.len() as f32
    }

    /// Average precision.
    pub fn precision(&self) -> f32 {
        let precisions = self.metrics.iter().map(|m| m.tp / (m.tp + m.fp)).collect::<Vec<f32>>();

        precisions.iter().sum::<f32>() / precisions.len() as f32
    }

    pub fn f1(&self, average: Average) -> f32 {
        match average {
            Average::Macro => self.f1_macro(),
            Average::Micro | Average::Binary => self.f1_micro(), // micro = binary if num_classes = 2
        }
    }

    /// Calculate the f1 using micro metrics, i.e. the sum of predicates.
    /// This evaluates the classifier as a whole instead of evaluating it as a sum of individual parts.
    fn f1_micro(&self) -> f32 {
        let tp = self.metrics.iter().map(|m| m.tp).sum::<f32>();
        let fn_ = self.metrics.iter().map(|m| m.fn_).sum::<f32>();
        let fp = self.metrics.iter().map(|m| m.fp).sum::<f32>();

        let recall = tp / (tp + fn_);
        let precision = tp / (tp + fp);

        // We risk NaN in f1_micro when precision + recall == 0, because that indicates that
        // both precision and recall are terrible, and the model is likely broken, so giving
        // a NaN that is incomparable to other more valid scores will prevent incorrect
        // comparisons across deceptively comparable scores.
        2. * ((precision * recall) / (precision + recall))
    }

    /// Calculate f1 using the average of class f1's.
    /// This gives equal opportunity to each class to impact the overall score.
    fn f1_macro(&self) -> f32 {
        let recalls = self.metrics.iter().map(|m| m.tp / (m.tp + m.fn_)).collect::<Vec<f32>>();
        let precisions = self.metrics.iter().map(|m| m.tp / (m.tp + m.fp)).collect::<Vec<f32>>();

        let mut f1s = Vec::new();

        for (i, recall) in recalls.iter().enumerate() {
            let precision = precisions[i];
            f1s.push(2. * ((precision * recall) / (precision + recall)));
        }

        f1s.iter().sum::<f32>() / f1s.len() as f32
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ndarray::array;

    #[test]
    fn test_confusion_matrix_multiclass() {
        let ground_truth = array![1, 2, 3, 4, 4];
        let y_hat = array![1, 2, 3, 4, 4];

        let mat = ConfusionMatrix::new(&ArrayView1::from(&ground_truth), &ArrayView1::from(&y_hat), 4);

        let f1 = mat.f1(Average::Macro);
        let f1_micro = mat.f1(Average::Micro);

        assert_eq!(mat.matrix[(3, 3)], 2.0);
        assert_eq!(f1, 1.0);
        assert_eq!(f1_micro, 1.0);
    }
}
