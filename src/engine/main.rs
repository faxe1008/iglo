use iglo::engine::bots::nply_bot::NPlyBot;
use iglo::engine::bots::nplytranspo_bot::NPlyTranspoBot;
use iglo::engine::{bots::oneply_bot::OnePlyBot, bots::random_bot::RandomBot, uci::UCIReader};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        UCIReader::<NPlyTranspoBot>::default().run();
    } else {
        match &args[1] as &str {
            "random" => UCIReader::<RandomBot>::default().run(),
            "oneply" => UCIReader::<OnePlyBot>::default().run(),
            "nply" => UCIReader::<NPlyBot>::default().run(),
            "nplytranspo" => UCIReader::<NPlyTranspoBot>::default().run(),
            _ => return,
        }
    };
}
