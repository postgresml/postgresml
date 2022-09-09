pub struct Dataset {
    pub x: Vec<f32>,
    pub y: Vec<f32>,
    pub num_features: usize,
    pub num_labels: usize,
    pub num_rows: usize,
    pub num_train_rows: usize,
    pub num_test_rows: usize,
}

impl Dataset {
    pub fn x_train(&self) -> &[f32] {
        &self.x[..self.num_train_rows * self.num_features]
    }

    pub fn x_test(&self) -> &[f32] {
        &self.x[self.num_train_rows * self.num_features..]
    }

    pub fn y_train(&self) -> &[f32] {
        &self.y[..self.num_train_rows * self.num_labels]
    }

    pub fn y_test(&self) -> &[f32] {
        &self.y[self.num_train_rows * self.num_labels..]
    }
}
