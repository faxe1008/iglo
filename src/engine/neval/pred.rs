use iglo::chess::board::ChessBoardState;
use tch::nn::{self, Module};
use tch::{Device, Tensor};
use std::env;
use std::io::{self, BufRead};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the model path and normalization parameters path from the command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <model_path> <normalization_path>", args[0]);
        return Ok(());
    }
    let model_path = &args[1];
    let normalization_path = &args[2];
    println!("Loading model from: {}", model_path);
    println!("Loading normalization parameters from: {}", normalization_path);

    // Load the PyTorch model
    let mut vs = nn::VarStore::new(Device::Cpu);
    let net = nn::seq()
    .add(nn::linear(vs.root() / "layer1", 768, 512, Default::default()))
    .add_fn(|xs| xs.elu()) // In-place ELU
    .add(nn::linear(vs.root() / "layer2", 512, 256, Default::default()))
    .add_fn(|xs| xs.elu())
    .add(nn::linear(vs.root() / "layer3", 256, 128, Default::default()))
    .add_fn(|xs| xs.elu())
    .add(nn::linear(vs.root() / "output", 128, 1, Default::default()));
    
    vs.load(model_path)?;

    // Load normalization parameters
    let normalization_params = Tensor::load(normalization_path)?;
    let target_min = normalization_params.double_value(&[0]);
    let target_max = normalization_params.double_value(&[1]);

    println!("Model and normalization parameters loaded successfully!");
    println!("Target min: {:.6}, Target max: {:.6}", target_min, target_max);

    // Create a buffer to read FEN positions from standard input
    let stdin = io::stdin();
    let mut buffer = String::new();

    const TARGET_MIN : f64 = 0.0;
    const TARGET_MAX : f64 = 10.0;

    loop {
        // Read a line from standard input
        buffer.clear();
        stdin.read_line(&mut buffer)?;
        let fen = buffer.trim();

        if fen.is_empty() {
            println!("Empty input received. Exiting.");
            break;
        }

        // Convert FEN to neural network input representation
        let input_data = convert_fen_to_nn_input(fen);
        let input_tensor = Tensor::from_slice(&input_data).view([1, 768]);

        // Run the model
        let output = net.forward(&input_tensor);

        // Get the output (denormalize it)
        let normalized_output = output.double_value(&[0]);

        // Reverse the normalization formula to get the original target value
        let denormalized_output = (normalized_output - TARGET_MIN) * (target_max - target_min) / (TARGET_MAX - TARGET_MIN) + target_min;
            
        println!("Output (normalized): {:.6}", normalized_output);
        println!("Output (denormalized): {:.6}", denormalized_output);

    }

    Ok(())
}

// Function to convert FEN to neural network input representation
fn convert_fen_to_nn_input(fen: &str) -> Vec<f32> {
    // Placeholder implementation: Replace with actual conversion logic
    let board = ChessBoardState::from_fen(fen).unwrap();
    let mut repr: [f32; 768] = [0.0; 768];
    board.get_neuralnetwork_representation(&mut repr);
    repr.to_vec()
}
