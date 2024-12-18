import numpy as np
import tensorflow as tf
from tensorflow.keras.models import Sequential
from tensorflow.keras.layers import Dense, BatchNormalization, Activation
from tensorflow.keras.optimizers import SGD
import torch
import torch.nn as nn
import torch.optim as optim
import sys


# Function to load and process the data
def load_data(filepath):
    inputs, targets = [], []
    with open(filepath, 'r') as f:
        for line in f:
            parts = line.strip().split(';')
            if len(parts) == 770:  # 768 input values + target + ignored first column
                inputs.append([float(x) for x in parts[1:769]])  # 768 inputs
                targets.append(float(parts[-1]))  # Target score

    inputs = np.array(inputs, dtype=np.float32)
    targets = np.array(targets, dtype=np.float32)

    # Normalize target values
    target_mean = np.mean(targets)
    target_std = np.std(targets)
    targets_normalized = (targets - target_mean) / target_std

    print(f"Target mean: {target_mean}, Target std: {target_std}")
    return inputs, targets_normalized, target_mean, target_std

# Main function
def main():
    if len(sys.argv) != 2:
        print("Usage: python train_nn.py <data_file>")
        return

    data_file = sys.argv[1]
    print("Loading data...")
    X, y_normalized, target_mean, target_std = load_data(data_file)
    print(f"Data loaded. {X.shape[0]} samples with {X.shape[1]} features each.")

    # Build Keras model
    keras_model = Sequential()
    keras_model.add(Dense(2048, input_dim=768))
    keras_model.add(BatchNormalization())
    keras_model.add(Activation('elu'))
    keras_model.add(Dense(2048))
    keras_model.add(BatchNormalization())
    keras_model.add(Activation('elu'))
    keras_model.add(Dense(2048))
    keras_model.add(BatchNormalization())
    keras_model.add(Activation('elu'))
    keras_model.add(Dense(1))  # Single output for the centipawn score

    optimizer = SGD(learning_rate=0.001, momentum=0.7, nesterov=True)
    keras_model.compile(optimizer=optimizer, loss='mean_squared_error', metrics=['mae'])

    # Train Keras model
    print("Training model...")
    keras_model.fit(X, y_normalized, batch_size=256, epochs=1, validation_split=0.1)

    # Save Keras model
    keras_model.save("trained_model.h5")

    # Save normalization parameters
    np.save("target_mean_std.npy", np.array([target_mean, target_std]))

    print("Model training complete. Model saved as 'trained_model.h5' and 'trained_model.pt'.")
    print("Target normalization parameters saved as 'target_mean_std.npy'.")

if __name__ == "__main__":
    main()
