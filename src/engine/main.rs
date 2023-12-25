use std::env;

use iglo::engine::bots::nply_bot::NPlyBot;
use iglo::engine::{bots::oneply_bot::OnePlyBot, bots::random_bot::RandomBot, uci::UCIReader};
use iglo::engine::bot::ChessBot;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        UCIReader::<OnePlyBot>::default().run();
    } else {
        match &args[1] as &str {
            "random" => UCIReader::<RandomBot>::default().run(),
            "oneply" => UCIReader::<OnePlyBot>::default().run(),
            "nply" => UCIReader::<NPlyBot>::default().run(),
            _ => return,
        }
    };
}
