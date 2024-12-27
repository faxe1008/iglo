use tch::{nn, Device, Tensor};

// Clipping values for the CReLU activation function
const CRELU_CLIP_MIN : f64 = 0.0;
const CRELU_CLIP_MAX : f64 = 2.0;

const SIGMOID_SCALING_FACTOR : f64 = 400.0;

fn squared_crelu(value: &Tensor) -> Tensor {
   value.clamp(CRELU_CLIP_MIN, CRELU_CLIP_MAX).relu()
}

pub fn normalize_target_value(tensor: &Tensor) -> Tensor{
    (tensor / SIGMOID_SCALING_FACTOR).sigmoid()
}

pub fn denormalize_target_value(tensor: &Tensor) -> Tensor {
    // Add type annotation to the tensor division operation
    let tensor : Tensor = tensor / (1.0 - tensor);
    tensor.log() * SIGMOID_SCALING_FACTOR
}

pub fn create_network(path: Option<&str>) -> (nn::VarStore, impl nn::Module) {
    let mut vs = nn::VarStore::new(Device::cuda_if_available());
    let net = nn::seq()
        .add(nn::linear(vs.root() / "layer1", 768, 256, Default::default()))
        .add_fn(|xs| squared_crelu(xs))
        .add(nn::linear(vs.root() / "layer2", 256, 32, Default::default()))
        .add_fn(|xs| squared_crelu(xs))
        .add(nn::linear(vs.root() / "output", 32, 1, Default::default()));

    if let Some(path) = path {
        vs.load(path).unwrap();
    }

    (vs, net)
}
