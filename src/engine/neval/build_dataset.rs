use iglo::chess::board::{self, ChessPiece, PieceColor};
use iglo::chess::square::Square;
use iglo::chess::{board::ChessBoardState, chess_move::Move, zobrist_hash::ZHash};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::io::{self, BufRead};
use std::process::{Command, Stdio};
use std::{collections::HashSet, env};

fn get_game_states(moves: Vec<&str>) -> Vec<ChessBoardState> {
    let mut board_states = Vec::new();
    let mut board_state = ChessBoardState::starting_state();
    for move_str in &moves {
        let mv = Move::try_from((*move_str, &board_state));
        if mv.is_err() {
            eprintln!("Error move: {}", move_str);
            return vec![];
        }
        let mv = mv.unwrap();
        board_state = board_state.exec_move(mv);
        board_states.push(board_state.clone());
    }
    board_states
}
fn get_stock_fish_evaluation(
    board_state: &ChessBoardState,
    stock_fish_proc: &mut std::process::Child,
) -> Option<f64> {
    let mut stockfish_stdin = stock_fish_proc
        .stdin
        .as_mut()
        .expect("Failed to open stdin");
    let stockfish_stdout = stock_fish_proc
        .stdout
        .as_mut()
        .expect("Failed to open stdout");

    // Wrap the stdout in a BufReader to enable read_line
    let mut reader = BufReader::new(stockfish_stdout);

    let fen = board_state.to_fen();

    // Send the position with the given moves to Stockfish
    stockfish_stdin
        .write_all(format!("position fen {}\n", fen).as_bytes())
        .expect("Failed to write to Stockfish");
    stockfish_stdin
        .write_all("go depth 12\n".as_bytes())
        .expect("Failed to write to Stockfish");

    let mut evaluation = None;
    let mut line = String::new();

    // Read the output of Stockfish
    loop {
        reader
            .read_line(&mut line)
            .expect("Failed to read from Stockfish");

        // Break if Stockfish sends the "bestmove" line
        if line.starts_with("bestmove") {
            break;
        }

        // Look for "info depth" lines which contain evaluation information
        if line.starts_with("info depth") {
            // Parse the evaluation score from the line
            if let Some(score_pos) = line.find("score cp") {
                // The score value comes right after "score cp" and is followed by the value
                let score_str = &line[score_pos + 9..].trim(); // Skip "score cp" part
                if let Ok(score) = score_str
                    .split_whitespace()
                    .next()
                    .unwrap_or("0")
                    .parse::<f64>()
                {
                    evaluation = Some(score);
                }
            }
        }
        line.clear(); // Clear line buffer for the next read
    }
    evaluation
}

fn main() -> io::Result<()> {
    // Get the file path from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path_to_file>", args[0]);
        std::process::exit(1);
    }
    let file_path = &args[1];
    // Open the file and read lines
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);
    // Create a subprocess of the Stockfish via Command
    let mut stockfish = Command::new("stockfish")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start Stockfish");

    let seen_state_evals = HashSet::<ZHash>::new();

    // Process each line in the file
    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split(';').collect();
        if parts.len() != 2 {
            eprintln!("Invalid line format: {}", line);
            continue;
        }
        let moves_str: Vec<&str> = parts[1].trim().split(',').collect();

        let mut neural_network_input_arr : [f64; 768]= [0.0; 768];

        for game_state in get_game_states(moves_str) {
            if seen_state_evals.contains(&game_state.zhash) {
                continue;
            }
            let evaluation = get_stock_fish_evaluation(&game_state, &mut stockfish);
            if evaluation.is_none() {
                continue;
            }

            game_state.get_neuralnetwork_representation(&mut neural_network_input_arr);
            let neuralnetwork_input_str = neural_network_input_arr
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(";");
            
            println!(
                "{};{};{}",
                game_state.to_fen(),
                neuralnetwork_input_str,
                evaluation.unwrap()
            );
        }
    }

    Ok(())
}
