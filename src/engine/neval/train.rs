use iglo::chess::board::ChessBoardState;
use std::fs;
use std::io::Write;
use std::io::{self, BufRead};
use tch::Kind;
use tch::{nn, nn::Module, nn::OptimizerConfig, Device, Tensor};
use rand::seq::SliceRandom;

const TARGET_MIN : f64 = 0.0;
const TARGET_MAX : f64 = 10.0;

const CLIPPING_MIN : f64 = -2000.0;
const CLIPPING_MAX : f64 = 2000.0;

#[inline(always)]
fn load_data(filepath: &str) -> (Tensor, Tensor, f64, f64) {
    let file = fs::File::open(filepath).expect("Failed to open data file");
    let reader = io::BufReader::new(file);

    let mut inputs = Vec::with_capacity(100000); // Pre-allocate memory
    let mut targets = Vec::with_capacity(100000);

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let parts: Vec<&str> = line.trim().split(';').collect();
        if parts.len() == 770 {
            inputs.extend(parts[1..769].iter().map(|x| x.parse::<f32>().unwrap()));
            targets.push(parts[769].parse::<f32>().unwrap());
        }
    }

    let input_tensor = Tensor::from_slice(&inputs).view([-1, 768]);
    let target_tensor = Tensor::from_slice(&targets).clamp(CLIPPING_MIN, CLIPPING_MAX);



    // Calculate the minimum value to shift the data
    let target_min = target_tensor.min().double_value(&[]);
    let target_max = target_tensor.max().double_value(&[]);

    // Normalize the target tensor
    let targets_normalized = (target_tensor - target_min) / (target_max - target_min) * (TARGET_MAX - TARGET_MIN) + TARGET_MIN;

    (
        input_tensor.to_device(Device::Cpu),
        targets_normalized.to_device(Device::Cpu),
        target_min,
        target_max,
    )
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: cargo run <data_file>");
        return;
    }

    let data_file = &args[1];
    println!("Loading data...");
    let (inputs, targets_normalized, target_min, target_max) = load_data(data_file);
    println!(
        "Data loaded. {} samples with {} features each.",
        inputs.size()[0],
        inputs.size()[1]
    );
    println!("Target min: {}, Target max: {}", target_min, target_max);

    let num_samples = inputs.size()[0];
    let train_size = (0.9 * num_samples as f64).round() as i64;

    // Training/validation split
    let train_inputs = inputs.narrow(0, 0, train_size);
    let train_targets = targets_normalized.narrow(0, 0, train_size);
    let val_inputs = inputs.narrow(0, train_size, num_samples - train_size);
    let val_targets = targets_normalized.narrow(0, train_size, num_samples - train_size);

    println!(
        "Training set: {} samples, Validation set: {} samples",
        train_inputs.size()[0],
        val_inputs.size()[0]
    );

    let vs = nn::VarStore::new(Device::cuda_if_available());
    let net = nn::seq()
        .add(nn::linear(vs.root() / "layer1", 768, 256, Default::default()))
        .add_fn(|xs| xs.elu())
        .add(nn::linear(vs.root() / "layer2", 256, 128, Default::default()))
        .add_fn(|xs| xs.elu())
        .add(nn::linear(vs.root() / "output", 128, 1, Default::default()));

    let initial_lr = 1e-3;
    let decay_rate: f64 = 0.90;
    let decay_epochs = 20;
    let mut opt = nn::Sgd::default().build(&vs, initial_lr).unwrap();
    opt.set_momentum(0.7);

    println!("Training model...");
    let batch_size = 512;
    let num_epochs = 60;

    for epoch in 0..num_epochs {
        if epoch > 0 && epoch % decay_epochs == 0 {
            let new_lr = initial_lr * decay_rate.powf((epoch / decay_epochs) as f64);
            opt.set_lr(new_lr);
        }

        // Shuffle training data
        let permutation = Tensor::randperm(train_size, (tch::Kind::Int64, Device::Cpu));
        let shuffled_inputs = train_inputs.index_select(0, &permutation);
        let shuffled_targets = train_targets.index_select(0, &permutation);

        let num_train_batches = (train_size as f64 / batch_size as f64).ceil() as i64;

        for batch_idx in 0..num_train_batches {
            let start = batch_idx * batch_size;
            let end = ((batch_idx + 1) * batch_size).min(train_size);

            let input_batch = shuffled_inputs.narrow(0, start, end - start);
            let target_batch = shuffled_targets.narrow(0, start, end - start).view([-1, 1]);

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
                    loss.double_value(&[]),
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

            let target_batch = target_batch.view([-1, 1]);
            let predictions = net.forward(&input_batch);
            let batch_loss = predictions.mse_loss(&target_batch, tch::Reduction::Mean);

            val_loss_sum += batch_loss.double_value(&[]) * (end - start) as f64;
            val_total_samples += (end - start) as usize;
        }

        let val_loss = val_loss_sum / val_total_samples as f64;
        let val_accuracy = 1.0 - val_loss;

        println!(
            " - Validation Loss: {:.6} - Validation Accuracy: {:.6}",
            val_loss, val_accuracy
        );
    }

    println!("Saving model...");
    vs.save("trained_model.ot").expect("Failed to save model");

    Tensor::from_slice(&[target_min as f32, target_max as f32])
        .save("target_min_max.npy")
        .expect("Failed to save normalization parameters");

    println!("Model training complete. Model saved as 'trained_model.ot'.");
}
