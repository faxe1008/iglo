use std::sync::{Arc, atomic::AtomicBool};
use crate::chess::{board::ChessBoardState, chess_move::Move};
use super::board_eval::EvaluationFunction;

#[derive(PartialEq, Debug)]
pub enum TimeControl {
    Infinite,
    FixedDepth(u32)
}

pub trait ChessBot : EvaluationFunction + Default {
    fn search_best_move(&mut self, board_state: &mut ChessBoardState, tc: TimeControl, stop: &Arc<AtomicBool>) -> Move;
    fn set_option(&mut self, name: String, value: String);
    fn get_options() -> &'static str;

    fn append_to_history(&mut self, board_state: &mut ChessBoardState);
    fn clear_history(&mut self);
    fn execute_move_list(&mut self, board_state: &mut ChessBoardState, moves: &Vec<String>) 
    {
        self.clear_history();
        for move_str in moves {
            if let Ok(mv) = Move::try_from(((*move_str).trim(), &*board_state)) {
                eprintln!("Got: '{}', Executed: {:?}", move_str, &mv);
                *board_state = board_state.exec_move(mv);
                self.append_to_history(board_state);
            } else {
                eprintln!("Illegal move: '{}'", move_str);
            }
        }
    }
}