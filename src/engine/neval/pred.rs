use iglo::chess::board::ChessBoardState;
use tch::nn::Module;
use tch::Tensor;
use std::env;
use std::io::{self};
use iglo::engine::neval::network::{create_network, denormalize_target_value};

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

    let (_, net) = create_network(Some(&model_path));

    println!("Model and normalization parameters loaded successfully!");

    // Create a buffer to read FEN positions from standard input
    let stdin = io::stdin();
    let mut buffer = String::new();

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
        let denormalized_output = denormalize_target_value(&output).double_value(&[0]);
            
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
