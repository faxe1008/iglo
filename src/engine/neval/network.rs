use tch::{nn, Device, Tensor};

// Clipping values for the CReLU activation function
const CRELU_CLIP_MIN : f64 = 0.0;
const CRELU_CLIP_MAX : f64 = 2.0;

// Clipping values for the evaluation in centipawns
const EVALUTION_CLAMP_MIN : f64 = -20.0;
const EVALUTION_CLAMP_MAX : f64 = 20.0;

// Target value normalization range
const TARGET_MIN : f64 = 0.0;
const TARGET_MAX : f64 = 1.0;

fn squared_crelu(value: &Tensor) -> Tensor {
   value.clamp(CRELU_CLIP_MIN, CRELU_CLIP_MAX).relu()
}

pub fn normalize_target_value(tensor: &Tensor) -> Tensor{
    (tensor / 100.0).clamp(EVALUTION_CLAMP_MIN, EVALUTION_CLAMP_MAX) * (TARGET_MAX - TARGET_MIN) + TARGET_MIN
}

pub fn denormalize_target_value(tensor: &Tensor) -> Tensor {
    (tensor - TARGET_MIN) / (TARGET_MAX - TARGET_MIN) * 100.0
}

pub fn create_network(path: Option<&str>) -> (nn::VarStore, impl nn::Module) {
    let mut vs = nn::VarStore::new(Device::cuda_if_available());
    let net = nn::seq()
        .add(nn::linear(vs.root() / "layer1", 768, 64, Default::default()))
        .add_fn(|xs| squared_crelu(xs))
        .add(nn::linear(vs.root() / "layer2", 64, 32, Default::default()))
        .add_fn(|xs| squared_crelu(xs))
        .add(nn::linear(vs.root() / "output", 32, 1, Default::default()));

    if let Some(path) = path {
        vs.load(path).unwrap();
    }

    (vs, net)
}
