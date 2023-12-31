use iglo::{
    chess::{
        board::{ChessBoardState},
        chess_move::Move,
        zobrist_hash::ZHash,
    },
    engine::{
        board_eval::{
            EvaluationFunction, PassedPawnEvaluation, PieceCountEvaluation,
            PieceSquareTableEvaluation,
        },
        opening::opening_book::{OpeningBook, OpeningBookEntry},
        search::Searcher,
    },
};
use iter_progress::ProgressableIter;
use std::{
    collections::{HashMap, HashSet},
    env,
    fs::File,
    io::{BufRead, BufReader, Write},
};

fn process_line(line: &str, openings: &mut HashMap<ZHash, (ChessBoardState, HashSet<Move>)>) {
    let mut board_state = ChessBoardState::starting_state();

    let line_parts: Vec<&str> = line.split(";").collect();
    let _opening_name = line_parts[0];

    let move_strings: Vec<&str> = line_parts[1].trim().split(",").collect();

    for move_str in &move_strings {
        let mv = Move::try_from((*move_str, &board_state));

        if mv.is_err() {
            eprintln!("Error move: {}", move_str);
            return;
        }
        let mv = mv.unwrap();

        // Save the hash before executing the move
        let entry_board_state = board_state.clone();
        board_state = board_state.exec_move(mv);

        if let Some(existing_opening) = openings.get_mut(&entry_board_state.zhash) {
            existing_opening.1.insert(mv);
        } else {
            // Create new entry
            openings.insert(entry_board_state.zhash, (entry_board_state, [mv].into()));
        }
    }
}

fn sort_moves_in_order<const T: usize>(
    searcher: &mut Searcher<T>,
    board_state: &mut ChessBoardState,
    moves: &HashSet<Move>,
) -> Vec<Move> {
    searcher.clear_hash_table();
    let mut move_vec: Vec<Move> = moves.iter().map(|x| x.clone()).collect();
    searcher.minimax_root(board_state, &mut move_vec, 6);
    move_vec
}

fn eval(board_state: &ChessBoardState) -> i32 {
    PieceCountEvaluation::eval(board_state)
        + PieceSquareTableEvaluation::eval(board_state)
        + PassedPawnEvaluation::eval(board_state)
}

const TT_SIZE: usize = 256 * 1024 * 1024;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return;
    }

    let file = File::open(&args[1]).unwrap();
    let reader = BufReader::new(file);

    let mut openings = HashMap::<ZHash, (ChessBoardState, HashSet<Move>)>::new();
    let mut searcher = Searcher::<TT_SIZE>::new(eval);

    for line in reader.lines() {
        let line = line.unwrap();

        process_line(&line, &mut openings);
    }

    // Remove states with only 1 continuation
    openings.retain(|_, (_, move_set)| move_set.len() > 1);

    // Collect into Vec for sorting
    let mut opening_list: Vec<(&ZHash, &(ChessBoardState, HashSet<Move>))> =
        openings.iter().collect();

    // Sort list by hashes in ascending order
    opening_list.sort_by(|a, b| a.0 .0.cmp(&b.0 .0));

    let mut entries = Vec::new();
    for (progress, (hash, (state, move_set))) in opening_list.iter().progress() {
        let mut board = state.clone();
        let entry = OpeningBookEntry {
            position: **hash,
            moves: sort_moves_in_order(&mut searcher, &mut board, move_set),
        };

        progress.do_every_n_sec(1., |progress_state| {
            println!(
                "{}% the way though, and doing {} per sec.",
                progress_state.percent().unwrap(),
                progress_state.rate()
            );
        });

        entries.push(entry);
    }

    let opening_book = OpeningBook { entries };

    let bytes = bincode::serialize(&opening_book).unwrap();

    let mut file = File::create("opening_book.bin").unwrap();
    file.write_all(&bytes).unwrap();

    println!("{:?}", &opening_book.entries[0]);
}
