use pgx::*;
/// Let's get lit up.
///
/// libtorch neural nets.
use std::collections::HashMap;
use tch::{kind::Kind, nn, nn::Module, nn::OptimizerConfig, Device, Tensor};

const HIDDEN_NODES: i64 = 128;
const LABELS: i64 = 1;

fn net(vs: &nn::Path, features: i64) -> impl Module {
    nn::seq()
        .add(nn::linear(
            vs / "layer1",
            features,
            HIDDEN_NODES,
            Default::default(),
        ))
        .add_fn(|xs| xs.relu())
        .add(nn::linear(vs, HIDDEN_NODES, 1, Default::default()))
}

pub fn train(
    x_train: Vec<f32>,
    y_train: Vec<f32>,
    x_test: Vec<f32>,
    y_test: Vec<f32>,
    features: i64,
) {
    let vs = nn::VarStore::new(Device::Cpu);
    let net = net(&vs.root(), features);
    let mut opt = nn::Adam::default().build(&vs, 1e-3).unwrap();

    let dtrain = Tensor::of_slice(&x_train)
        .reshape(&[x_train.len() as i64 / features, features])
        .to_kind(Kind::Float);
    let train_labels = Tensor::of_slice(&y_train).unsqueeze(-1);
    let dtest = Tensor::of_slice(&x_test)
        .reshape(&[x_test.len() as i64 / features, features])
        .to_kind(Kind::Float);
    let test_labels = Tensor::of_slice(&y_test).unsqueeze(-1);

    assert_eq!(x_train.len() % features as usize, 0);
    assert_eq!(x_train.len() % y_train.len(), 0);

    for epoch in 1..200 {
        let loss = net
            .forward(&dtrain)
            .mse_loss(&train_labels, tch::Reduction::Mean);

        opt.backward_step(&loss);

        let test_accuracy = net.forward(&dtest).accuracy_for_logits(&test_labels);

        info!(
            "epoch: {:4} train loss: {} test acc: {}%",
            epoch,
            f64::from(&loss),
            f64::from(&test_accuracy),
        )
    }
}
