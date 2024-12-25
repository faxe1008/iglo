use ctrlc;
use iglo::chess::{board::ChessBoardState, chess_move::Move};
use num_cpus;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::{env, sync::mpsc, thread};

const NUM_ENTRIES: usize = 1800000;

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
    stdin: &mut std::process::ChildStdin,
    reader: &mut std::io::BufReader<&mut std::process::ChildStdout>,
    stop_flag: &Arc<AtomicBool>,
) -> Option<f64> {
    let fen = board_state.to_fen();

    stdin
        .write_all(format!("position fen {}\ngo depth 12\n", fen).as_bytes())
        .expect("Failed to write to Stockfish");

    let mut evaluation = None;
    let start_time = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(5); // 5-second timeout
    let mut line = String::new();

    loop {
        if stop_flag.load(Ordering::SeqCst) {
            eprintln!("Stopping due to SIGTERM during evaluation.");
            break;
        }

        if start_time.elapsed() > timeout {
            eprintln!("Stockfish evaluation timed out.");
            break;
        }

        if let Ok(bytes_read) = reader.read_line(&mut line) {
            if bytes_read == 0 {
                eprintln!("Unexpected EOF from Stockfish.");
                break;
            }

            if line.starts_with("bestmove") {
                break;
            }

            if line.starts_with("info depth") {
                if let Some(score_pos) = line.find("score cp") {
                    let score_str = &line[score_pos + 9..].trim();
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
            line.clear();
        }
    }
    evaluation
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path_to_file>", args[0]);
        std::process::exit(1);
    }
    let file_path = &args[1];

    let file = File::open(file_path)?;
    let mut content = String::new();
    BufReader::new(file).read_to_string(&mut content)?;

    let lines: Vec<String> = content.lines().map(String::from).collect();
    eprintln!("Loaded {} lines", lines.len());

    let thread_count = num_cpus::get();
    eprintln!("Using {} threads", thread_count);
    let chunk_size = (lines.len() + thread_count - 1) / thread_count;
    let entries_per_thread = NUM_ENTRIES / thread_count;

    let (tx, rx) = mpsc::channel();
    let stop_flag = Arc::new(AtomicBool::new(false));

    ctrlc::set_handler({
        let stop_flag = Arc::clone(&stop_flag);
        move || {
            eprintln!("SIGTERM received, stopping threads...");
            stop_flag.store(true, Ordering::SeqCst);
            eprintln!("After setting stop flag");
        }
    })
    .expect("Error setting Ctrl-C handler");

    let mut handles = Vec::new();

    for (thread_id, chunk) in lines.chunks(chunk_size).enumerate() {
        let chunk = chunk.to_vec();
        let tx = tx.clone();
        let stop_flag = Arc::clone(&stop_flag);

        let handle = thread::spawn(move || {
            let mut stockfish = Command::new("stockfish")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .expect("Failed to start Stockfish");

            let mut stockfish_stdin = stockfish.stdin.as_mut().expect("Failed to open stdin");
            let stockfish_stdout = stockfish.stdout.as_mut().expect("Failed to open stdout");
            let mut stockfish_reader = BufReader::new(stockfish_stdout);

            let mut thread_results = HashMap::<u64, (ChessBoardState, i32)>::new();
            let mut entries_processed = 0;

            for line in chunk {
                if stop_flag.load(Ordering::SeqCst) {
                    eprintln!(
                        "Thread {} received stop signal before processing a new line",
                        thread_id
                    );
                    break;
                }

                if entries_processed >= entries_per_thread {
                    break;
                }

                let parts: Vec<&str> = line.split(';').collect();
                if parts.len() != 2 {
                    eprintln!("Invalid line format: {}", line);
                    continue;
                }
                let moves_str: Vec<&str> = parts[1].trim().split(',').collect();

                for game_state in get_game_states(moves_str) {
                    if stop_flag.load(Ordering::SeqCst) {
                        eprintln!(
                            "Thread {} received stop signal during state processing",
                            thread_id
                        );
                        break;
                    }

                    let zhash = game_state.zhash;

                    if thread_results.contains_key(&zhash.0) {
                        continue;
                    }

                    if let Some(evaluation) = get_stock_fish_evaluation(
                        &game_state,
                        &mut stockfish_stdin,
                        &mut stockfish_reader,
                        &stop_flag,
                    ) {
                        thread_results.insert(zhash.0, (game_state, evaluation as i32));
                        entries_processed += 1;
                    }

                    if entries_processed % 1000 == 0 {
                        eprintln!(
                            "Thread {} processed {} entries",
                            thread_id, entries_processed
                        );
                    }
                }
            }

            tx.send(thread_results).expect("Failed to send results");
        });

        handles.push(handle);
    }

    drop(tx);

    let mut final_results = HashMap::new();
    for thread_result in rx {
        final_results.extend(thread_result);
    }

    for handle in handles {
        let _ = handle.join();
    }

    for (_zhash, (state, evaluation)) in final_results.iter() {
        let mut neural_network_input_arr: [f32; 768] = [0.0; 768];
        state.get_neuralnetwork_representation(&mut neural_network_input_arr);
        let neuralnetwork_input_str = neural_network_input_arr
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
            .join(";");

        println!(
            "{};{};{}",
            state.to_fen(),
            neuralnetwork_input_str,
            evaluation
        );
    }

    Ok(())
}
