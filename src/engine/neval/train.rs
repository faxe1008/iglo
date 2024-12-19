use std::fs;
use std::io::Write;
use std::io::{self, BufRead};
use iglo::chess::board::ChessBoardState;
use tch::Kind;
use tch::{nn, nn::Module, nn::OptimizerConfig, Device, Tensor};

fn load_data(filepath: &str) -> (Tensor, Tensor, f64, f64) {
    let file = fs::File::open(filepath).expect("Failed to open data file");
    let reader = io::BufReader::new(file);

    let mut inputs = Vec::new();
    let mut targets = Vec::new();

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let parts: Vec<&str> = line.trim().split(';').collect();
        if parts.len() == 770 {
            let input: Vec<f32> = parts[1..769]
                .iter()
                .map(|x| x.parse::<f32>().expect("Failed to parse input"))
                .collect();
            inputs.push(input);
            targets.push(parts[769].parse::<f32>().expect("Failed to parse target"));
        }
    }

    let inputs_tensor = Tensor::from_slice2(&inputs).to_device(Device::Cpu);
    let targets_tensor = Tensor::from_slice(&targets).to_device(Device::Cpu);

    let target_mean = targets_tensor.mean(Kind::Float);
    let target_std = targets_tensor.std(false);

    let targets_normalized = (targets_tensor - &target_mean) / &target_std;

    (
        inputs_tensor,
        targets_normalized,
        target_mean.double_value(&[]),
        target_std.double_value(&[]),
    )
}

use rand::seq::SliceRandom;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: cargo run <data_file>");
        return;
    }

    let data_file = &args[1];
    println!("Loading data...");
    let (inputs, targets_normalized, target_mean, target_std) = load_data(data_file);
    println!(
        "Data loaded. {} samples with {} features each.",
        inputs.size()[0],
        inputs.size()[1]
    );
    println!("Target mean: {}, Target std: {}", target_mean, target_std);

    // Define training/validation split
    let num_samples = inputs.size()[0];
    let train_size = (0.9 * num_samples as f64).round() as usize;

    // Shuffle indices
    let mut indices: Vec<i64> = (0..num_samples).collect();
    indices.shuffle(&mut rand::thread_rng());

    let train_indices = &indices[..train_size];
    let val_indices = &indices[train_size..];

    let train_inputs = inputs.index_select(0, &Tensor::from_slice(train_indices));
    let train_targets = targets_normalized.index_select(0, &Tensor::from_slice(train_indices));
    let val_inputs = inputs.index_select(0, &Tensor::from_slice(val_indices));
    let val_targets = targets_normalized.index_select(0, &Tensor::from_slice(val_indices));

    println!(
        "Training set: {} samples, Validation set: {} samples",
        train_inputs.size()[0],
        val_inputs.size()[0]
    );

    // Define the neural network structure
    let vs = nn::VarStore::new(Device::cuda_if_available());
    let net = nn::seq()
        .add(nn::linear(
            vs.root() / "layer1",
            768,
            2048,
            Default::default(),
        ))
        .add_fn(|xs| xs.elu())
        .add(nn::linear(
            vs.root() / "layer2",
            2048,
            2048,
            Default::default(),
        ))
        .add_fn(|xs| xs.elu())
        .add(nn::linear(
            vs.root() / "layer3",
            2048,
            2048,
            Default::default(),
        ))
        .add_fn(|xs| xs.elu())
        .add(nn::linear(
            vs.root() / "output",
            2048,
            1,
            Default::default(),
        ));

    // Define optimizer
    let mut opt = nn::Sgd::default().build(&vs, 1e-3).unwrap();
    opt.set_momentum(0.7);

    // Training loop
    println!("Training model...");
    let batch_size = 256;
    let num_train_batches = (train_size as f64 / batch_size as f64).ceil() as i64;

    let num_epochs = 120;

    for epoch in 0..num_epochs {
        // Training phase
        for batch_idx in 0..num_train_batches {
            let start = batch_idx * batch_size;
            let end = ((batch_idx + 1) * batch_size).min(train_size as i64);

            let input_batch = train_inputs.narrow(0, start, end - start);
            let target_batch = train_targets.narrow(0, start, end - start);

            // Ensure target_batch is 2D with shape [batch_size, 1]
            let target_batch = target_batch.view([-1, 1]);

            let predictions = net.forward(&input_batch);
            let loss = predictions.mse_loss(&target_batch, tch::Reduction::Mean);

            opt.backward_step(&loss);

            if batch_idx % 100 == 0 {
                print!(
                    "\rEpoch {}/{} - Batch {}/{} - Loss: {:.6}",
                    epoch + 1,
                    num_epochs,
                    batch_idx + 1,
                    num_train_batches,
                    loss.double_value(&[])
                );
                io::stdout().flush().unwrap();
            }
        }

        // Validation phase
        let num_val_samples = val_inputs.size()[0];
        let num_val_batches = (num_val_samples as f64 / batch_size as f64).ceil() as i64;
        let mut val_loss_sum = 0.0;
        let mut val_total_samples = 0;

        for batch_idx in 0..num_val_batches {
            let start = batch_idx * batch_size;
            let end = ((batch_idx + 1) * batch_size).min(num_val_samples);

            let input_batch = val_inputs.narrow(0, start, end - start);
            let target_batch = val_targets.narrow(0, start, end - start);

            // Ensure target_batch is 2D with shape [batch_size, 1]
            let target_batch = target_batch.view([-1, 1]);

            let predictions = net.forward(&input_batch);
            let batch_loss = predictions.mse_loss(&target_batch, tch::Reduction::Mean);

            val_loss_sum += batch_loss.double_value(&[]) * (end - start) as f64;
            val_total_samples += (end - start) as usize;
        }

        let val_loss = val_loss_sum / val_total_samples as f64;
        let val_accuracy = 1.0 - val_loss; // Placeholder accuracy metric

        // End of epoch, print final validation metrics
        println!(
            " - Validation Loss: {:.6} - Validation Accuracy: {:.6}",
            val_loss, val_accuracy
        );
    }

    // Save model
    println!("Saving model...");
    vs.save("trained_model.ot").expect("Failed to save model");

    // Save normalization parameters
    Tensor::from_slice(&[target_mean as f32, target_std as f32])
        .save("target_mean_std.npy")
        .expect("Failed to save normalization parameters");

    println!("Model training complete. Model saved as 'trained_model.pt'.");
    println!("Target normalization parameters saved as 'target_mean_std.npy'.");
}
