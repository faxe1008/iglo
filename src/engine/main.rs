use iglo::engine::{uci::UCIReader, bots::random_bot::RandomBot};

fn main () {
    let uci_reader = UCIReader::<RandomBot>::default();
    uci_reader.run();
}